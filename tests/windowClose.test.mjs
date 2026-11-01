import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { mockIPC, mockWindows, clearMocks } from '@tauri-apps/api/mocks';
import { createCloseHandler } from '../src/lib/windowClose.ts';
import { createSettingsWriter } from '../src/lib/settingsWriter.ts';

const capability = JSON.parse(readFileSync(new URL('../src-tauri/capabilities/default.json', import.meta.url)));

// Run the installed Tauri SDK's actual close-request dispatch, replacing only
// the native transport and event registration. Browser mocks do not enforce ACLs.
async function nativeWindow(save) {
  globalThis.window = {};
  mockWindows('main');
  const calls = [];
  let listener;
  mockIPC(async (cmd, args) => {
    assert.equal(args.label, 'main');
    assert.ok(capability.windows.includes(args.label));
    assert.ok(capability.permissions.includes(`core:window:allow-${cmd.split('|')[1]}`), `Missing permission for ${cmd}`);
    calls.push(cmd);
    if (cmd === 'plugin:window|close') {
      await listener({ event: 'tauri://close-requested', id: 1, payload: null });
    }
  });
  const win = getCurrentWindow();
  win.listen = async (event, callback) => {
    assert.equal(event, 'tauri://close-requested');
    listener = callback;
    return () => {};
  };
  await win.onCloseRequested(createCloseHandler(save));
  return { win, calls };
}

test('X closes a clean window through the SDK with the configured native permissions', async (t) => {
  t.after(clearMocks);
  const { win, calls } = await nativeWindow(async () => {});
  await win.close();
  assert.deepEqual(calls, ['plugin:window|close', 'plugin:window|destroy']);
});

test('closing waits for pending settings and destroys the window once, without recursive close requests', async (t) => {
  t.after(clearMocks);
  let release;
  let persisted = false;
  const gate = new Promise((resolve) => { release = resolve; });
  const writer = createSettingsWriter({
    async save(value) { await gate; persisted = true; return value; },
    onSaved: () => {}, onError: assert.fail,
  });
  writer.schedule({ palette: 'iris' });
  const { win, calls } = await nativeWindow(writer.flush);
  const closing = win.close();
  assert.equal(persisted, false);
  assert.deepEqual(calls, ['plugin:window|close']);
  release();
  await closing;
  assert.equal(persisted, true);
  assert.deepEqual(calls, ['plugin:window|close', 'plugin:window|destroy']);
});

test('a save error preserves the open window and the next close retries successfully', async (t) => {
  t.after(clearMocks);
  let fail = true;
  let errors = 0;
  const writer = createSettingsWriter({
    async save(value) { if (fail) throw Error('disk full'); return value; },
    onSaved: () => {}, onError: () => errors++,
  });
  writer.schedule({ palette: 'ocean' });
  const { win, calls } = await nativeWindow(writer.flush);
  await win.close();
  assert.deepEqual(calls, ['plugin:window|close']);
  assert.equal(errors, 1);
  assert.equal(writer.hasPending(), true);
  fail = false;
  await win.close();
  assert.deepEqual(calls, ['plugin:window|close', 'plugin:window|close', 'plugin:window|destroy']);
  assert.equal(writer.hasPending(), false);
});

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { adoptSaved, createSettingsWriter } from '../src/lib/settingsWriter.ts';

const deferred = () => {
  let resolve;
  const promise = new Promise((r) => { resolve = r; });
  return { promise, resolve };
};

test('serializes writes and keeps the newest edit while an older save is in flight', async () => {
  const gate = deferred();
  const writes = [];
  const applied = [];
  const writer = createSettingsWriter({
    async save(value) {
      writes.push(value);
      if (writes.length === 1) await gate.promise;
      return value;
    },
    onSaved: (value) => applied.push(value), onError: assert.fail,
  });
  const draft = { memory: 2048 };
  writer.schedule(draft);
  draft.memory = 9000;
  const saving = writer.flush();
  writer.schedule({ memory: 4096 });
  writer.schedule({ memory: 6144 });
  assert.equal(writes.length, 1);
  assert.strictEqual(writer.flush(), saving);
  gate.resolve();
  await saving;
  assert.deepEqual(writes, [{ memory: 2048 }, { memory: 6144 }]);
  assert.deepEqual(applied, writes);
  assert.equal(writer.hasPending(), false);
});

test('retains a failed change for an explicit retry', async () => {
  let fail = true;
  let errors = 0;
  let saved;
  const writer = createSettingsWriter({
    async save(value) { if (fail) throw Error('disk full'); return value; },
    onSaved: (value) => { saved = value; }, onError: () => errors++,
  });
  writer.schedule({ theme: 'iris' });
  await assert.rejects(writer.flush(), /disk full/);
  assert.equal(writer.hasPending(), true);
  assert.equal(errors, 1);
  fail = false;
  await writer.flush();
  assert.deepEqual(saved, { theme: 'iris' });
});

test('a failed old write never replaces a newer pending snapshot', async () => {
  const gate = deferred();
  let first = true;
  const writes = [];
  const writer = createSettingsWriter({
    async save(value) {
      writes.push(value);
      if (first) { first = false; await gate.promise; throw Error('offline'); }
      return value;
    }, onSaved: () => {}, onError: () => {},
  });
  writer.schedule({ theme: 'ocean' });
  const saving = writer.flush();
  writer.schedule({ theme: 'forest' });
  gate.resolve();
  await assert.rejects(saving);
  await writer.flush();
  assert.deepEqual(writes, [{ theme: 'ocean' }, { theme: 'forest' }]);
});

test('a slow save never overwrites a field edited after it was submitted', () => {
  const submitted = { uiScale: 3, accent: '#111111', engine: { hdr: null } };
  const saved = { uiScale: 1.25, accent: '#111111', engine: { hdr: null } };
  const current = { uiScale: 3, accent: '#222222', engine: { hdr: true } };
  assert.deepEqual(adoptSaved(current, saved, submitted), { uiScale: 1.25 });
});

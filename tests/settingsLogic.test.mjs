import { test } from 'node:test';
import assert from 'node:assert/strict';
import { accentForeground, contrast, parseHex, usableAccent } from '../src/lib/color.ts';
import { parseWhole } from '../src/lib/numbers.ts';
import { updateAction } from '../src/lib/updatePolicy.ts';

const light = '#f2f3f3';
const dark = '#17191a';

test('a pale accent stays visible on a light page and keeps its use on a dark one', () => {
  const shifted = usableAccent('#e4e4e7', light);
  assert.notEqual(shifted, '#e4e4e7');
  assert.ok(contrast(parseHex(shifted), parseHex(light)) >= 3);
  assert.equal(usableAccent('#e4e4e7', dark), '#e4e4e7');
});

test('a dark accent is lifted on a dark page; junk falls back', () => {
  assert.ok(contrast(parseHex(usableAccent('#101010', dark)), parseHex(dark)) >= 3);
  assert.equal(usableAccent('not-a-colour', dark), '#3fb8a0');
});

test('text on the accent stays readable', () => {
  assert.equal(accentForeground('#e4e4e7'), '#101214');
  assert.equal(accentForeground('#1d3b6f'), '#f5f6f6');
});

test('number fields keep the old value on junk and clamp the rest', () => {
  assert.equal(parseWhole('', 320, 15360, 1280), 1280);
  assert.equal(parseWhole('abc', 320, 15360, 1280), 1280);
  assert.equal(parseWhole('10', 320, 15360, 1280), 320);
  assert.equal(parseWhole('99999', 320, 15360, 1280), 15360);
  assert.equal(parseWhole(' 1920.4 ', 320, 15360, 1280), 1920);
});

const update = { critical: false, installable: true, version: '1.0.0', dismissed: '', gameBusy: false };

test('normal updates ask, and Later hides that version only', () => {
  assert.equal(updateAction(update), 'prompt');
  assert.equal(updateAction({ ...update, gameBusy: true }), 'prompt');
  assert.equal(updateAction({ ...update, dismissed: '1.0.0' }), 'none');
  assert.equal(updateAction({ ...update, dismissed: '0.9.0' }), 'prompt');
});

test('critical updates never ask, but wait for a running game', () => {
  assert.equal(updateAction({ ...update, critical: true }), 'install');
  assert.equal(updateAction({ ...update, critical: true, dismissed: '1.0.0' }), 'install');
  assert.equal(updateAction({ ...update, critical: true, gameBusy: true }), 'wait');
});

test('a build that cannot install itself only offers the link', () => {
  assert.equal(updateAction({ ...update, critical: true, installable: false }), 'prompt');
});

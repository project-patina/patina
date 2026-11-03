import { test } from 'node:test';
import assert from 'node:assert/strict';
import { en } from '../src/lib/locales/en.ts';
import { pl } from '../src/lib/locales/pl.ts';
import { format, resolveLanguage } from '../src/lib/locales/resolve.ts';

test('both dictionaries hold the same keys and placeholders', () => {
  assert.deepEqual(Object.keys(en).sort(), Object.keys(pl).sort());
  const names = (text) => [...new Set(text.match(/\{\w+\}/g) ?? [])].sort();
  for (const key of Object.keys(pl)) assert.deepEqual(names(en[key]), names(pl[key]), key);
});

test('plural keys have the forms their language needs', () => {
  for (const key of Object.keys(pl)) {
    if (!pl[key].includes('|')) continue;
    assert.equal(pl[key].split('|').length, 3, key);
    assert.equal(en[key].split('|').length, 2, key);
  }
});

test('the saved language wins, otherwise the system decides', () => {
  assert.equal(resolveLanguage('en', 'pl-PL'), 'en');
  assert.equal(resolveLanguage('pl', 'en-US'), 'pl');
  assert.equal(resolveLanguage('', 'pl-PL'), 'pl');
  assert.equal(resolveLanguage(undefined, 'de-DE'), 'en');
});

test('plurals follow each language', () => {
  const polish = '{n} obraz|{n} obrazy|{n} obrazów';
  assert.equal(format(polish, 'pl', { n: 1 }), '1 obraz');
  assert.equal(format(polish, 'pl', { n: 3 }), '3 obrazy');
  assert.equal(format(polish, 'pl', { n: 5 }), '5 obrazów');
  assert.equal(format(polish, 'pl', { n: 22 }), '22 obrazy');
  assert.equal(format('{n} image|{n} images', 'en', { n: 1 }), '1 image');
  assert.equal(format('{n} image|{n} images', 'en', { n: 2 }), '2 images');
});

test('unknown placeholders stay visible', () => {
  assert.equal(format('Hi {name} {rest}', 'en', { name: 'Ana' }), 'Hi Ana {rest}');
});

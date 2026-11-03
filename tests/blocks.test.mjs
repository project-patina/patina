import { test } from 'node:test';
import assert from 'node:assert/strict';
import { blockTracks, css, multiply, translation } from '../src/lib/blocks.ts';

const { duration, tracks } = blockTracks();
const position = (matrix) => matrix.slice(12, 15);
const near = (a, b) => a.every((value, i) => Math.abs(value - b[i]) < 1e-6);

test('eight blocks finish as one 2x2x2 cube with nothing overlapping', () => {
  assert.equal(tracks.length, 8);
  const homes = tracks.map((track) => position(track.home).map((v) => v.toFixed(3)).join());
  assert.equal(new Set(homes).size, 8);
  for (const [x, y, z] of tracks.map((track) => position(track.home))) {
    assert.ok([0.5, 1.5].includes(+x.toFixed(6)));
    assert.ok([-0.5, -1.5].includes(+y.toFixed(6)));
    assert.ok([0.5, 1.5].includes(+z.toFixed(6)));
  }
});

test('blocks stay rigid: every sampled frame is a pure rotation and translation', () => {
  for (const { frames } of tracks) {
    for (const { matrix: m } of frames) {
      const rows = [[m[0], m[1], m[2]], [m[4], m[5], m[6]], [m[8], m[9], m[10]]];
      for (const row of rows) assert.ok(Math.abs(Math.hypot(...row) - 1) < 1e-6);
    }
  }
});

test('the loop closes and offsets only go forward', () => {
  for (const { frames } of tracks) {
    assert.equal(frames[0].offset, 0);
    assert.equal(frames.at(-1).offset, 1);
    assert.ok(near(frames[0].matrix, frames.at(-1).matrix));
    for (let i = 1; i < frames.length; i++) assert.ok(frames[i].offset >= frames[i - 1].offset);
  }
  assert.ok(duration > 3000 && duration < 8000);
});

test('blocks start on the floor and no two share a cell at any moment', () => {
  for (const { frames } of tracks) assert.equal(+position(frames[0].matrix)[1].toFixed(6), -0.5);
  for (let time = 0; time <= 1; time += 0.01) {
    const at = tracks.map(({ frames }) => {
      const frame = frames.findLast((f) => f.offset <= time) ?? frames[0];
      return position(frame.matrix);
    });
    for (let a = 0; a < at.length; a++) {
      for (let b = a + 1; b < at.length; b++) assert.ok(Math.hypot(at[a][0] - at[b][0], at[a][1] - at[b][1], at[a][2] - at[b][2]) > 0.95, `blocks ${a}, ${b} at ${time}`);
    }
  }
});

test('each block ends with its faces lit like the logo', () => {
  for (const { faces } of tracks) {
    const shades = faces.map((face) => face.shade).sort();
    assert.deepEqual(shades, ['hidden', 'hidden', 'hidden', 'left', 'right', 'top']);
  }
});

test('css scales only the translation', () => {
  assert.equal(css(multiply(translation(1, 0, 0), translation(0, 2, 0)), 10), 'matrix3d(1,0,0,0,0,1,0,0,0,0,1,0,10,20,0,1)');
});

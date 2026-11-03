// Geometry of the update animation: eight unit blocks roll across a floor grid and build one 2x2x2 cube.
// Space is CSS space in block units: x right, y down, z towards the viewer. The floor is y = 0.
export type Matrix = number[];
export type Direction = "+x" | "-x" | "+z" | "-z";
export interface Frame { offset: number; matrix: Matrix }
export interface Face { name: string; shade: "top" | "left" | "right" | "hidden" }
export interface BlockTrack { layer: 0 | 1; frames: Frame[]; home: Matrix; faces: Face[] }

const ROLL_MS = 300;
const CLIMB_MS = 460;
const SAMPLES = 8;
const FLOOR_STAGGER_MS = 130;
const TOP_DELAY_MS = 880;
const HOLD_MS = 800;
const REST_MS = 260;

export function multiply(a: Matrix, b: Matrix): Matrix {
  const out = new Array<number>(16).fill(0);
  for (let column = 0; column < 4; column++) {
    for (let row = 0; row < 4; row++) {
      for (let k = 0; k < 4; k++) out[column * 4 + row] += a[k * 4 + row] * b[column * 4 + k];
    }
  }
  return out;
}

export function translation(x: number, y: number, z: number): Matrix {
  return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, x, y, z, 1];
}

function rotation(axis: "x" | "z", angle: number): Matrix {
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return axis === "z" ? [c, s, 0, 0, -s, c, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] : [1, 0, 0, 0, 0, c, s, 0, 0, -s, c, 0, 0, 0, 0, 1];
}

/** Maps a block's world matrix through a rotation about a floor-parallel edge. */
function aboutEdge(pivot: [number, number, number], axis: "x" | "z", angle: number): Matrix {
  return multiply(translation(...pivot), multiply(rotation(axis, angle), translation(-pivot[0], -pivot[1], -pivot[2])));
}

const STEP: Record<Direction, { axis: "x" | "z"; sign: number; dx: number; dz: number }> = {
  "+x": { axis: "z", sign: 1, dx: 1, dz: 0 },
  "-x": { axis: "z", sign: -1, dx: -1, dz: 0 },
  "+z": { axis: "x", sign: -1, dx: 0, dz: 1 },
  "-z": { axis: "x", sign: 1, dx: 0, dz: -1 },
};

/** A block's route: its home cell, the way it travels there and whether it climbs onto the first layer. */
interface Route { cell: [number, number]; heading: Direction; layer: 0 | 1; start: number }

const ease = (t: number) => (t < 0.5 ? 4 * t * t * t : 1 - (-2 * t + 2) ** 3 / 2);

// Blocks arrive as a pinwheel: no two routes share a lane, so none passes through another.
const ROUTES: Route[] = [
  { cell: [0, 0], heading: "+x", layer: 0, start: 0 },
  { cell: [1, 0], heading: "+z", layer: 0, start: FLOOR_STAGGER_MS },
  { cell: [1, 1], heading: "-x", layer: 0, start: FLOOR_STAGGER_MS * 2 },
  { cell: [0, 1], heading: "-z", layer: 0, start: FLOOR_STAGGER_MS * 3 },
  { cell: [0, 0], heading: "+z", layer: 1, start: TOP_DELAY_MS },
  { cell: [1, 0], heading: "-x", layer: 1, start: TOP_DELAY_MS + FLOOR_STAGGER_MS },
  { cell: [1, 1], heading: "-z", layer: 1, start: TOP_DELAY_MS + FLOOR_STAGGER_MS * 2 },
  { cell: [0, 1], heading: "+x", layer: 1, start: TOP_DELAY_MS + FLOOR_STAGGER_MS * 3 },
];

interface Sample { time: number; matrix: Matrix }

function walk({ cell, heading, layer, start }: Route): { samples: Sample[]; end: number; orientation: Matrix } {
  const { axis, sign, dx, dz } = STEP[heading];
  // The lane starts two cells back from home.
  let x = cell[0] - dx * 2;
  let z = cell[1] - dz * 2;
  let world = translation(x + 0.5, -0.5, z + 0.5);
  const samples: Sample[] = [{ time: start, matrix: world }];
  let time = start;

  const roll = (pivot: [number, number, number], angle: number, ms: number) => {
    const before = world;
    for (let i = 1; i <= SAMPLES; i++) {
      const t = i / SAMPLES;
      samples.push({ time: time + ms * t, matrix: multiply(aboutEdge(pivot, axis, angle * ease(t)), before) });
    }
    world = multiply(aboutEdge(pivot, axis, angle), before);
    time += ms;
  };
  // The edge at the block's leading side, at the given height.
  const edge = (height: number): [number, number, number] => [x + 0.5 + dx / 2, -height, z + 0.5 + dz / 2];

  roll(edge(0), sign * Math.PI / 2, ROLL_MS);
  x += dx; z += dz;
  if (layer === 0) {
    roll(edge(0), sign * Math.PI / 2, ROLL_MS);
  } else {
    // Over the top edge of the finished layer: half a turn lands the block one cell on, one up.
    roll(edge(1), sign * Math.PI, CLIMB_MS);
  }
  return { samples, end: time, orientation: world };
}

const FACE_NORMALS: { name: string; normal: [number, number, number] }[] = [
  { name: "front", normal: [0, 0, 1] }, { name: "back", normal: [0, 0, -1] },
  { name: "right", normal: [1, 0, 0] }, { name: "left", normal: [-1, 0, 0] },
  { name: "top", normal: [0, -1, 0] }, { name: "bottom", normal: [0, 1, 0] },
];

/** Paints each face for the side it ends up on, so the finished cube is lit like the logo. */
function facesFor(home: Matrix): Face[] {
  return FACE_NORMALS.map(({ name, normal: [nx, ny, nz] }) => {
    const wx = home[0] * nx + home[4] * ny + home[8] * nz;
    const wy = home[1] * nx + home[5] * ny + home[9] * nz;
    const wz = home[2] * nx + home[6] * ny + home[10] * nz;
    return { name, shade: wy < -0.5 ? "top" : wx < -0.5 ? "left" : wz > 0.5 ? "right" : "hidden" };
  });
}

/** One full loop: assemble, hold, then the same path in reverse. Offsets are 0..1 of `duration`. */
export function blockTracks(): { duration: number; tracks: BlockTrack[] } {
  const walks = ROUTES.map(walk);
  const built = Math.max(...walks.map((w) => w.end));
  const duration = built * 2 + HOLD_MS + REST_MS;
  const tracks = ROUTES.map((route, i) => {
    const { samples, orientation } = walks[i];
    const home = orientation;
    const out = [...samples].reverse().map(({ time, matrix }) => ({ time: built * 2 + HOLD_MS - time, matrix }));
    const frames = [{ time: 0, matrix: samples[0].matrix }, ...samples, { time: built + HOLD_MS, matrix: home }, ...out, { time: duration, matrix: samples[0].matrix }]
      .map(({ time, matrix }) => ({ offset: time / duration, matrix }));
    return { layer: route.layer, frames, home, faces: facesFor(home) };
  });
  return { duration, tracks };
}

export function css(matrix: Matrix, unit: number): string {
  const scaled = matrix.map((value, i) => (i >= 12 && i < 15 ? value * unit : value));
  return `matrix3d(${scaled.map((value) => +value.toFixed(5)).join(",")})`;
}

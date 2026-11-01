export const PARTS = {
  rightArm: "Prawa ręka",
  leftArm: "Lewa ręka",
  sword: "Miecz",
  shield: "Tarcza",
  head: "Głowa",
  body: "Tułów",
  rightLeg: "Prawa noga",
  leftLeg: "Lewa noga",
} as const;

export type PartKey = keyof typeof PARTS;

/** Rotation in degrees and position; `size` in % applies to the items only. */
export interface PartPose {
  rx: number;
  ry: number;
  rz: number;
  x: number;
  y: number;
  z: number;
  size?: number;
}

export type Pose = Record<PartKey, PartPose> & { sink: number; yaw: number };

const part = (rx: number, ry: number, rz: number, x: number, y: number, z: number, size?: number): PartPose => ({ rx, ry, rz, x, y, z, size });

/** The stance set in the pose editor. Positions are absolute in skinview3d's model space; items sit in the hand. */
export const DEFAULT_POSE: Pose = {
  rightArm: part(-14, -3.5, -14, -5.2, -3.05, 3.8),
  leftArm: part(-15, -3.5, 8, 5.8, -2.9, 3.95),
  sword: part(110.5, 80.5, -25.5, -1.4, -9.3, 0.25, 74),
  shield: part(4.5, 82.5, 69, 2.95, -8.1, 0.1, 74),
  head: part(6, 6.5, 0, 0.15, 0.35, 3.75),
  body: part(0, 0, 0.5, 0.4, -6.65, 4),
  rightLeg: part(7, -13.5, -3, -1.7, -13.4, 4.05),
  leftLeg: part(-3, 2, 5.5, 2.9, -13.35, 4.6),
  sink: -0.65,
  yaw: -26,
};

export function parsePose(saved: string | undefined): Pose {
  const pose = structuredClone(DEFAULT_POSE);
  if (!saved) return pose;
  try {
    const parsed = JSON.parse(saved);
    for (const key of Object.keys(PARTS) as PartKey[]) Object.assign(pose[key], parsed[key]);
    if (typeof parsed.sink === "number") pose.sink = parsed.sink;
    if (typeof parsed.yaw === "number") pose.yaw = parsed.yaw;
  } catch {
    // A damaged value falls back to the default stance.
  }
  return pose;
}

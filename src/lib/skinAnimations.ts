import type { Object3D } from "three";
import { loadGear, type Gear } from "./playerGear";
import type { PartPose, Pose } from "./pose";
import { FunctionAnimation, type PlayerAnimation, type PlayerObject } from "skinview3d";

/** Pointer position relative to the model's canvas, roughly -1…1 on each axis. */
export interface LookTarget {
  x: number;
  y: number;
  /** performance.now() of the last pointer move. */
  at: number;
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;
const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

/** Offsets of the raised parts, fixed by skinview3d's model. */
const BODY_Y = -6;
const ARM_Y = -2;

const ease = (x: number) => {
  const k = clamp(x, 0, 1);
  return k * k * (3 - 2 * k);
};

/** Head: follows the pointer for a few seconds after it moves, else looks around. */
function headTracker(look: LookTarget, bodyYaw: () => number, restYaw: number, restPitch: number, wander: number) {
  let yaw = restYaw;
  let pitch = restPitch;
  return (head: PlayerObject["skin"]["head"], t: number, dt: number) => {
    let ty: number, tp: number;
    if (performance.now() - look.at < 3500) {
      ty = clamp(look.x * 0.9 - bodyYaw(), -0.95, 0.95);
      tp = clamp(look.y * 0.55, -0.45, 0.5);
    } else {
      ty = restYaw + (Math.sin(t * 0.31) * 0.34 + Math.sin(t * 0.13) * 0.16) * wander - bodyYaw() * 0.4;
      tp = restPitch + Math.sin(t * 0.23) * 0.07 * wander;
    }
    const k = 1 - Math.pow(0.02, dt);
    yaw = lerp(yaw, ty, k);
    pitch = lerp(pitch, tp, k);
    head.rotation.y = yaw;
    head.rotation.x = pitch;
  };
}

const DEG = Math.PI / 180;

function place(part: Object3D, p: PartPose) {
  part.rotation.set(p.rx * DEG, p.ry * DEG, p.rz * DEG);
  part.position.set(p.x, p.y, p.z);
}

/**
 * The stance from `pose`, with the sword in the right hand and the shield in
 * the left. Unless `still()`, it comes alive: the player breathes, shifts
 * weight and twitches the sword now and then. `strike()` swings the sword.
 */
export function combatAnimation(look: LookTarget, bodyYaw: () => number, pose: () => Pose, still: () => boolean) {
  let gear: Gear | null = null;
  let disposed = false;
  let attached = false;
  let swing = -1;
  let last = -1;
  void loadGear()
    .then((g) => {
      if (disposed) g?.dispose();
      else gear = g;
    })
    .catch(() => {});
  const lookAt = headTracker(look, bodyYaw, 0, 0, 0.35);

  const anim = new FunctionAnimation((p: PlayerObject, t: number) => {
    const dt = last < 0 ? 0 : clamp(t - last, 0, 0.1);
    last = t;
    const s = p.skin;
    if (gear && !attached) {
      s.rightArm.add(gear.sword);
      s.leftArm.add(gear.shield);
      attached = true;
    }

    const stance = pose();
    p.position.y = stance.sink;
    p.rotation.y = stance.yaw * DEG;
    place(s.body, stance.body);
    place(s.head, stance.head);
    place(s.rightArm, stance.rightArm);
    place(s.leftArm, stance.leftArm);
    place(s.rightLeg, stance.rightLeg);
    place(s.leftLeg, stance.leftLeg);
    if (gear) {
      place(gear.sword, stance.sword);
      place(gear.shield, stance.shield);
      gear.sword.scale.setScalar((stance.sword.size ?? 74) / 100);
      gear.shield.scale.setScalar((stance.shield.size ?? 74) / 100);
      if (s.modelType === "slim") {
        gear.sword.position.x += 0.5;
        gear.shield.position.x -= 0.5;
      }
    }
    if (still()) {
      s.body.scale.set(1, 1, 1);
      return;
    }

    const breath = (1 - Math.cos(t * 1.7)) / 2;
    const sway = Math.sin(t * 0.8);
    const drift = Math.sin(t * 1.27 + 1);

    // A short twitch of the blade every 6 s or so.
    const beat = t % 6.3;
    const feint = ease(beat / 0.22) * (1 - ease((beat - 0.22) / 0.5));

    // A click: draw back, cut across, settle.
    let windup = 0;
    let cut = 0;
    if (swing >= 0) {
      const w = t - swing;
      if (w > 0.8) swing = -1;
      else {
        const strike = ease((w - 0.16) / 0.14);
        windup = ease(w / 0.16) * (1 - strike);
        cut = strike * (1 - ease((w - 0.36) / 0.44));
      }
    }

    const rise = breath * 0.24 + sway * 0.06;
    s.body.position.x += sway * 0.12;
    s.body.position.y += rise;
    s.body.rotation.x += breath * 0.012 + cut * 0.14 - windup * 0.05;
    s.body.rotation.y += sway * 0.05 - windup * 0.16 + cut * 0.75;
    s.body.rotation.z += sway * 0.025;
    s.body.scale.set(1 + breath * 0.012, 1, 1 + breath * 0.03);
    s.head.position.x += sway * 0.12;
    s.head.position.y += rise * 1.2;
    const baseYaw = s.head.rotation.y;
    const basePitch = s.head.rotation.x;
    lookAt(s.head, t, dt);
    s.head.rotation.y += baseYaw;
    s.head.rotation.x += basePitch;

    for (const arm of [s.rightArm, s.leftArm]) {
      arm.position.x += sway * 0.12;
      arm.position.y += rise * 1.3;
    }
    s.rightArm.rotation.x += -breath * 0.06 - drift * 0.03 + feint * 0.2 - windup * 0.8 + cut * 0.95;
    s.rightArm.rotation.z += -feint * 0.08 + cut * 0.35;
    s.leftArm.rotation.x += -breath * 0.05 + drift * 0.02 - cut * 0.2;
    if (gear) gear.sword.rotation.x += drift * 0.03;

    p.cape.rotation.x = 0.3 + breath * 0.03 + Math.abs(sway) * 0.03;
  });

  return Object.assign(anim, {
    strike() {
      if (swing < 0) swing = Math.max(last, 0);
    },
    gear: () => gear,
    dispose() {
      disposed = true;
      gear?.dispose();
    },
  });
}

/** Standing at ease: both feet planted, each breath lifting the chest, shoulders and head. */
export function aliveAnimation(look: LookTarget, bodyYaw: () => number) {
  let last = -1;
  const lookAt = headTracker(look, bodyYaw, 0, 0.04, 1);

  return new FunctionAnimation((p: PlayerObject, t: number) => {
    const dt = last < 0 ? 0 : clamp(t - last, 0, 0.1);
    last = t;
    const s = p.skin;
    const breath = (1 - Math.cos(t * 1.25)) / 2;
    const sway = Math.sin(t * 0.38);
    const drift = Math.sin(t * 0.61 + 1);

    s.body.position.y = BODY_Y + breath * 0.2;
    s.body.scale.set(1 + breath * 0.012, 1, 1 + breath * 0.03);
    s.body.rotation.set(-breath * 0.012, sway * 0.035, sway * 0.012);
    s.head.position.y = breath * 0.26;

    const shoulderL = (1 - Math.cos(t * 1.25 - 0.25)) / 2;
    s.rightArm.position.y = ARM_Y + breath * 0.3;
    s.leftArm.position.y = ARM_Y + shoulderL * 0.3;
    s.rightLeg.rotation.z = -0.05 + sway * 0.006;
    s.leftLeg.rotation.z = 0.05 + sway * 0.006;

    const armLift = breath * 0.05 + drift * 0.018;
    s.rightArm.rotation.set(-armLift, 0, -0.1 - breath * 0.03);
    s.leftArm.rotation.set(armLift, 0, 0.1 + shoulderL * 0.03);

    lookAt(s.head, t, dt);
    p.cape.rotation.x = 0.1 + breath * 0.03 + Math.abs(sway) * 0.015;
  });
}

export type AnyAnimation = PlayerAnimation & { strike?: () => void; dispose?: () => void; gear?: () => Gear | null };

/** What the armed stance needs; without it the player stands at ease. */
export interface Armed {
  pose: () => Pose;
  still: () => boolean;
}

export function makeAnimation(look: LookTarget, bodyYaw: () => number, armed: Armed | null): AnyAnimation {
  return armed ? combatAnimation(look, bodyYaw, armed.pose, armed.still) : aliveAnimation(look, bodyYaw);
}

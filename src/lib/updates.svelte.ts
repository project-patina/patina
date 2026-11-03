import { api, errText, events } from "./api";
import { PATINA_DEV } from "./flavour";
import { app, flushSettings } from "./state.svelte";
import { updateAction } from "./updatePolicy";
import type { UpdateInfo, UpdateProgress } from "./types";

const CHECK_EVERY_MS = 5 * 60 * 1000;
const RELEASES_URL = "https://github.com/project-patina/patina-updates/releases";

export { RELEASES_URL };

export const updates = $state({
  info: null as UpdateInfo | null,
  /** Installing covers the window; `failed` keeps it up with the error until the user answers. */
  phase: "idle" as "idle" | "installing" | "restarting" | "failed",
  progress: null as UpdateProgress | null,
  error: "",
  checking: false,
  checkError: "",
  checked: false,
  lastCheck: null as number | null,
  /** Version hidden with "Later" for this session. */
  dismissed: "",
  /** A critical update waiting for the game to exit. */
  waiting: false,
});

export const gameBusy = () => Object.keys(app.running).length > 0;

function act(info: UpdateInfo) {
  const action = updateAction({ ...info, dismissed: updates.dismissed, gameBusy: gameBusy() });
  updates.waiting = action === "wait";
  if (action === "install") void install();
}

async function check() {
  if (updates.checking || updates.phase !== "idle") return;
  updates.checking = true;
  updates.checkError = "";
  try {
    const info = await api.updateCheck();
    updates.info = info;
    updates.checked = true;
    updates.lastCheck = Math.floor(Date.now() / 1000);
    if (info) act(info);
    else updates.waiting = false;
  } finally {
    updates.checking = false;
  }
}

/** The settings button: failures are shown, and a version hidden with "Later" comes back. */
export async function checkNow() {
  updates.dismissed = "";
  try {
    await check();
  } catch (error) {
    updates.checkError = errText(error);
  }
}

export async function install() {
  if (updates.phase === "installing" || updates.phase === "restarting") return;
  updates.phase = "installing";
  updates.error = "";
  updates.progress = null;
  const stop = await events.updateProgress((p) => (updates.progress = p));
  try {
    await flushSettings();
    await api.updateInstall();
    updates.phase = "restarting";
    await api.updateRestart();
  } catch (error) {
    updates.phase = "failed";
    updates.error = errText(error);
  } finally {
    stop();
  }
}

/** Back into the launcher after a failure; a critical update is tried again on the next start. */
export function continueWithoutUpdate() {
  updates.phase = "idle";
  updates.error = "";
  updates.progress = null;
  updates.waiting = false;
  if (updates.info) updates.dismissed = updates.info.version;
}

export function later() {
  if (updates.info) updates.dismissed = updates.info.version;
}

/** Called when the game stops: a waiting critical update goes in now. */
export function gameFinished() {
  if (updates.waiting && updates.info && !gameBusy()) act(updates.info);
}

/** Checks after boot and then periodically. Dev builds stay quiet unless the mock backend answers. */
export function startUpdates(): () => void {
  if (PATINA_DEV || (import.meta.env.DEV && !(window as { __devmock?: boolean }).__devmock)) return () => {};
  const quietly = () => { void check().catch(() => {}); };
  quietly();
  const timer = setInterval(quietly, CHECK_EVERY_MS);
  return () => clearInterval(timer);
}

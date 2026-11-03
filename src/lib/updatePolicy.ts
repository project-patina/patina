export type UpdateAction = "none" | "prompt" | "install" | "wait";

export interface PolicyInput {
  critical: boolean;
  installable: boolean;
  version: string;
  dismissed: string;
  gameBusy: boolean;
}

/** What to do with a found update. Critical ones install on their own as soon as no game is running. */
export function updateAction({ critical, installable, version, dismissed, gameBusy }: PolicyInput): UpdateAction {
  if (critical && installable) return gameBusy ? "wait" : "install";
  return dismissed === version ? "none" : "prompt";
}

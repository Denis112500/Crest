import { invoke } from "@tauri-apps/api/core";

import { PERFORM_ACTIVITY_ACTION_COMMAND } from "./ipcChannelNames";

// Hands a button press to the Rust source that owns `activityKind`. Resolves once the
// request is queued; the visible result arrives later as a normal presentation update.
export async function requestActivityAction(activityKind: string, activityAction: string): Promise<void> {
  await invoke(PERFORM_ACTIVITY_ACTION_COMMAND, { activityKind, activityAction });
}

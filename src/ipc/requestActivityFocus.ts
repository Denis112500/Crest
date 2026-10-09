import { invoke } from "@tauri-apps/api/core";

import { FOCUS_ACTIVITY_COMMAND } from "./ipcChannelNames";

// The companion segment was clicked: Rust makes an ongoing activity the main one, or marks a
// lingering one's news as seen. The visible result arrives as a normal arrangement update.
export async function requestActivityFocus(activityKind: string): Promise<void> {
  await invoke(FOCUS_ACTIVITY_COMMAND, { activityKind });
}

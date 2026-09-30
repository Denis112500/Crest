import { invoke } from "@tauri-apps/api/core";

import { SET_PILL_INTERACTIVE_AREA_COMMAND } from "./ipcChannelNames";

// Only this rectangle of the window (CSS pixels, relative to the window) takes the mouse;
// everywhere else, clicks go to the app below.
export async function requestPillInteractiveArea(
  logicalLeft: number,
  logicalTop: number,
  logicalWidth: number,
  logicalHeight: number,
): Promise<void> {
  await invoke(SET_PILL_INTERACTIVE_AREA_COMMAND, { logicalLeft, logicalTop, logicalWidth, logicalHeight });
}

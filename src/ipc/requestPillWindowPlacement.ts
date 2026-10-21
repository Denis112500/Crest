// Asks Rust to size the window and put it at the top of the chosen monitor.

import { invoke } from "@tauri-apps/api/core";

import { PLACE_PILL_WINDOW_COMMAND } from "./ipcChannelNames";

// Tauri converts these camelCase keys to the snake_case Rust parameters (`logical_width`, ...).
// `topEdgeTriggerLogicalWidth`: how wide, centred, the screen's top edge opens the hidden pill.
export async function requestPillWindowPlacement(
  logicalWidth: number,
  logicalHeight: number,
  topEdgeTriggerLogicalWidth: number,
): Promise<void> {
  await invoke(PLACE_PILL_WINDOW_COMMAND, { logicalWidth, logicalHeight, topEdgeTriggerLogicalWidth });
}

import { invoke } from "@tauri-apps/api/core";

import { PLACE_PILL_WINDOW_COMMAND } from "./ipcChannelNames";

// Tauri converts these camelCase keys to the snake_case Rust parameters
// (`logical_width`, `logical_height`).
export async function requestPillWindowPlacement(
  logicalWidth: number,
  logicalHeight: number,
): Promise<void> {
  await invoke(PLACE_PILL_WINDOW_COMMAND, { logicalWidth, logicalHeight });
}

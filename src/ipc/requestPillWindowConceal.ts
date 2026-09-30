import { invoke } from "@tauri-apps/api/core";

import { CONCEAL_PILL_WINDOW_COMMAND } from "./ipcChannelNames";

// Hides the native window once the pill's hide animation has finished.
export async function requestPillWindowConceal(): Promise<void> {
  await invoke(CONCEAL_PILL_WINDOW_COMMAND);
}

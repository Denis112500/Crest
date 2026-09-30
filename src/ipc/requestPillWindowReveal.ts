import { invoke } from "@tauri-apps/api/core";

import { REVEAL_PILL_WINDOW_COMMAND } from "./ipcChannelNames";

export async function requestPillWindowReveal(): Promise<void> {
  await invoke(REVEAL_PILL_WINDOW_COMMAND);
}

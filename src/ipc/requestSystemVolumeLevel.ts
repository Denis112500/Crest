import { invoke } from "@tauri-apps/api/core";

import { SET_SYSTEM_VOLUME_LEVEL_COMMAND } from "./ipcChannelNames";

// Asks Rust to set the system volume (0..1) and to unmute, like Windows' own slider. Resolves once
// the request is queued. Rust doesn't announce this change back: the slider that asked shows it.
export async function requestSystemVolumeLevel(level: number): Promise<void> {
  await invoke(SET_SYSTEM_VOLUME_LEVEL_COMMAND, { level });
}

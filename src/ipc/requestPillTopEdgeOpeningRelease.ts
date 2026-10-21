import { invoke } from "@tauri-apps/api/core";

import { RELEASE_PILL_TOP_EDGE_OPENING_COMMAND } from "./ipcChannelNames";

// The pill brought down from the top edge closed again; Rust hides it unless something is on.
export async function requestPillTopEdgeOpeningRelease(): Promise<void> {
  await invoke(RELEASE_PILL_TOP_EDGE_OPENING_COMMAND);
}

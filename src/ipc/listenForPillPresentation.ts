import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { PillPresentation } from "../activities/pillPresentationTypes";
import { GET_CURRENT_PILL_PRESENTATION_COMMAND, PILL_PRESENTATION_CHANGED_EVENT } from "./ipcChannelNames";

// Starts listening *before* asking for the current state, so no change can slip into
// the gap. If an event arrives before the answer, the answer may already be outdated
// and is ignored.
export async function listenForPillPresentation(
  onPillPresentation: (pillPresentation: PillPresentation | null) => void,
): Promise<void> {
  let hasReceivedPresentationEvent = false;
  await listen<PillPresentation | null>(PILL_PRESENTATION_CHANGED_EVENT, (presentationEvent) => {
    hasReceivedPresentationEvent = true;
    onPillPresentation(presentationEvent.payload);
  });
  const currentPillPresentation = await invoke<PillPresentation | null>(GET_CURRENT_PILL_PRESENTATION_COMMAND);
  if (!hasReceivedPresentationEvent) {
    onPillPresentation(currentPillPresentation);
  }
}

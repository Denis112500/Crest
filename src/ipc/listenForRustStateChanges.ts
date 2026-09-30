import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// For state Rust owns and announces with an event (what to show, whether to show it).
// Events sent before the page listened are lost, so: start listening first, then ask for
// the current value once. If an event arrives before the answer, the answer may already
// be outdated and is ignored.
export async function listenForRustStateChanges<RustState>(
  stateChangedEventName: string,
  getCurrentStateCommandName: string,
  onRustState: (rustState: RustState) => void,
): Promise<void> {
  let hasReceivedStateEvent = false;
  await listen<RustState>(stateChangedEventName, (stateChangedEvent) => {
    hasReceivedStateEvent = true;
    onRustState(stateChangedEvent.payload);
  });
  const currentRustState = await invoke<RustState>(getCurrentStateCommandName);
  if (!hasReceivedStateEvent) {
    onRustState(currentRustState);
  }
}

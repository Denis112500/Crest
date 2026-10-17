import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// For state Rust owns and announces with an event (what to show, whether to show it).
// Events sent before the page listened are lost, so: start listening first, then ask for
// the current value once. If an event arrives before the answer, the answer may already
// be outdated and is ignored. Resolves with the way to stop listening (for state the page only
// follows while it's seen); if asking fails, listening stops again.
export async function listenForRustStateChanges<RustState>(
  stateChangedEventName: string,
  getCurrentStateCommandName: string,
  onRustState: (rustState: RustState) => void,
): Promise<UnlistenFn> {
  let hasReceivedStateEvent = false;
  const stopListening = await listen<RustState>(stateChangedEventName, (stateChangedEvent) => {
    hasReceivedStateEvent = true;
    onRustState(stateChangedEvent.payload);
  });
  try {
    const currentRustState = await invoke<RustState>(getCurrentStateCommandName);
    if (!hasReceivedStateEvent) {
      onRustState(currentRustState);
    }
  } catch (askError) {
    stopListening();
    throw askError;
  }
  return stopListening;
}

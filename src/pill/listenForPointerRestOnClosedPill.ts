// The closed pill lets every click through (fix phase P.2), so the page never sees it hovered;
// Rust watches the pointer and reports when it rests on the closed pill.

import { listen } from "@tauri-apps/api/event";

import { PILL_POINTER_RESTED_EVENT } from "../ipc/ipcChannelNames";
import type { PillShellElements } from "./pillShellElements";
import type { PillStateMachine } from "./pillStateMachine";

// Mirrors `PillPointerRestPoint` in src-tauri/src/pointer_watching/shared_pill_pointer_zones.rs.
interface PillPointerRestPoint {
  logicalX: number;
  logicalY: number;
}

// A rest on the companion's mark does what a click on it did: that activity comes forward, then
// the pill opens.
export async function listenForPointerRestOnClosedPill(
  pillShellElements: PillShellElements,
  pillStateMachine: PillStateMachine,
  onCompanionRestedOn: () => void,
): Promise<void> {
  await listen<PillPointerRestPoint>(PILL_POINTER_RESTED_EVENT, (restEvent) => {
    const restedOnElement = document.elementFromPoint(restEvent.payload.logicalX, restEvent.payload.logicalY);
    if (restedOnElement !== null && pillShellElements.companionMarkElement.contains(restedOnElement)) {
      onCompanionRestedOn();
    }
    pillStateMachine.handlePointerRestedOnPill();
  });
}

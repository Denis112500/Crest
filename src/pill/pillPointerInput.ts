// Turns the mouse events on the pill into state-machine calls.

import type { PillShellElements } from "./pillShellElements";
import type { PillStateMachine } from "./pillStateMachine";

// The window's interactive area matches the open pill, so ordinary DOM mouse events are
// enough there. The closed pill lets clicks through and gets none of these events: Rust reports
// the pointer resting on it instead (listenForPointerRestOnClosedPill.ts). A click on the
// companion's mark (while the pill closes, its area still takes clicks) first runs
// `onCompanionClicked` (which picks what the pill opens on), then bubbles to the pill's own
// click, which opens it.
export function connectPillPointerInput(
  pillShellElements: PillShellElements,
  pillStateMachine: PillStateMachine,
  onCompanionClicked: () => void,
): void {
  const { pillShellElement, companionMarkElement } = pillShellElements;
  pillShellElement.addEventListener("mouseenter", () => pillStateMachine.handlePointerEntered());
  pillShellElement.addEventListener("mouseleave", () => pillStateMachine.handlePointerLeft());
  pillShellElement.addEventListener("click", () => pillStateMachine.handlePointerClicked());
  companionMarkElement.addEventListener("mouseenter", () => pillStateMachine.handlePointerEnteredCompanion());
  companionMarkElement.addEventListener("mouseleave", () => pillStateMachine.handlePointerLeftCompanion());
  companionMarkElement.addEventListener("click", onCompanionClicked);
}

// Turns the mouse events on the pill into state-machine calls.

import type { PillShellElements } from "./pillShellElements";
import type { PillStateMachine } from "./pillStateMachine";

// The window's interactive area matches the pill, so ordinary DOM mouse events are
// enough: no cursor polling is needed. A click on the companion segment first runs
// `onCompanionClicked` (which picks what the pill opens on), then bubbles to the pill's own
// click, which opens it.
export function connectPillPointerInput(
  pillShellElements: PillShellElements,
  pillStateMachine: PillStateMachine,
  onCompanionClicked: () => void,
): void {
  const { pillShellElement, companionSegmentElement } = pillShellElements;
  pillShellElement.addEventListener("mouseenter", () => pillStateMachine.handlePointerEntered());
  pillShellElement.addEventListener("mouseleave", () => pillStateMachine.handlePointerLeft());
  pillShellElement.addEventListener("click", () => pillStateMachine.handlePointerClicked());
  companionSegmentElement.addEventListener("mouseenter", () => pillStateMachine.handlePointerEnteredCompanion());
  companionSegmentElement.addEventListener("mouseleave", () => pillStateMachine.handlePointerLeftCompanion());
  companionSegmentElement.addEventListener("click", onCompanionClicked);
}

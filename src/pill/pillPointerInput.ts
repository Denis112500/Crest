// Turns the mouse events on the pill into state-machine calls.

import type { PillStateMachine } from "./pillStateMachine";

// The window's interactive area matches the pill, so ordinary DOM mouse events are
// enough: no cursor polling is needed.
export function connectPillPointerInput(pillShellElement: HTMLElement, pillStateMachine: PillStateMachine): void {
  pillShellElement.addEventListener("mouseenter", () => pillStateMachine.handlePointerEntered());
  pillShellElement.addEventListener("mouseleave", () => pillStateMachine.handlePointerLeft());
  pillShellElement.addEventListener("click", () => pillStateMachine.handlePointerClicked());
}

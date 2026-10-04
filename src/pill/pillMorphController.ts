import {
  PILL_COMPACT_LOGICAL_HEIGHT,
  PILL_COMPACT_LOGICAL_WIDTH,
  PILL_MORPH_DURATION_MILLISECONDS,
  PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS,
  PILL_NOTCH_SHOULDER_LOGICAL_RADIUS,
  PILL_WINDOW_LOGICAL_HEIGHT,
  PILL_WINDOW_LOGICAL_WIDTH,
} from "../frontendConstants";
import { requestPillInteractiveArea } from "../ipc/requestPillInteractiveArea";
import type { PillShellElements } from "./pillShellElements";
import type { PillExpansionState } from "./pillStateMachine";

const EXPANDED_SHELL_CLASS = "is-expanded";

// Carries out a state change in the right order. Expanding: first make the whole window
// interactive, then animate, so the growing pill is never clipped. Collapsing: animate
// first, and only shrink the interactive area once the pill is small again.
export class PillMorphController {
  private collapseFinishFallbackTimer: number | undefined;

  constructor(
    private readonly pillShellElements: PillShellElements,
    private readonly onExpandedContentVisibilityChange: (isExpandedContentVisible: boolean) => void,
  ) {
    pillShellElements.pillShellElement.addEventListener("transitionend", (transitionEndEvent) => {
      if (transitionEndEvent.target === pillShellElements.pillShellElement && transitionEndEvent.propertyName === "height") {
        this.finishCollapseIfStillCompact();
      }
    });
  }

  applyCompactInteractiveArea(): Promise<void> {
    // The interactive area also clips what is drawn, so it includes the shoulders.
    const compactNotchLogicalWidth = PILL_COMPACT_LOGICAL_WIDTH + 2 * PILL_NOTCH_SHOULDER_LOGICAL_RADIUS;
    return requestPillInteractiveArea(
      (PILL_WINDOW_LOGICAL_WIDTH - compactNotchLogicalWidth) / 2,
      0,
      compactNotchLogicalWidth,
      PILL_COMPACT_LOGICAL_HEIGHT,
    );
  }

  showExpansionState(expansionState: PillExpansionState): void {
    window.clearTimeout(this.collapseFinishFallbackTimer);
    const pillShellClassList = this.pillShellElements.pillShellElement.classList;
    if (expansionState === "expanded") {
      requestPillInteractiveArea(0, 0, PILL_WINDOW_LOGICAL_WIDTH, PILL_WINDOW_LOGICAL_HEIGHT).catch(
        reportInteractiveAreaFailure,
      );
      pillShellClassList.add(EXPANDED_SHELL_CLASS);
      this.onExpandedContentVisibilityChange(true);
      return;
    }
    pillShellClassList.remove(EXPANDED_SHELL_CLASS);
    this.collapseFinishFallbackTimer = window.setTimeout(
      () => this.finishCollapseIfStillCompact(),
      PILL_MORPH_DURATION_MILLISECONDS + PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS,
    );
  }

  // Also reached by the fallback timer; the class check makes a late call after a
  // re-expand harmless.
  private finishCollapseIfStillCompact(): void {
    if (this.pillShellElements.pillShellElement.classList.contains(EXPANDED_SHELL_CLASS)) {
      return;
    }
    window.clearTimeout(this.collapseFinishFallbackTimer);
    this.applyCompactInteractiveArea().catch(reportInteractiveAreaFailure);
    this.onExpandedContentVisibilityChange(false);
  }
}

function reportInteractiveAreaFailure(interactiveAreaError: unknown): void {
  console.error("Crest could not update the pill's interactive area:", interactiveAreaError);
}

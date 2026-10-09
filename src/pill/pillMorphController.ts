import {
  PILL_EXPANDED_MAX_LOGICAL_HEIGHT,
  PILL_MORPH_DURATION_MILLISECONDS,
  PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS,
} from "../frontendConstants";
import { requestPillInteractiveArea } from "../ipc/requestPillInteractiveArea";
import { calculatePillInteractiveArea } from "./calculatePillInteractiveArea";
import type { PillShellElements } from "./pillShellElements";
import type { PillExpansionState } from "./pillStateMachine";

const EXPANDED_SHELL_CLASS = "is-expanded";
const WITH_COMPANION_SHELL_CLASS = "has-companion";
const EXPANDED_HEIGHT_CSS_VARIABLE = "--pill-expanded-height";

/** What the content needs from the pill's shape (reported by PillContentPresenter). */
export interface PillContentLayout {
  expandedLogicalHeight: number;
  isCompanionSegmentShown: boolean;
}

// Carries out a change of state or size in the right order. Growing (opening, taller content
// while open, a companion appearing while compact): first enlarge the interactive area, then
// animate, so the growing pill is never clipped. Shrinking: animate first, and only shrink the
// interactive area once the pill has settled.
export class PillMorphController {
  private settleFallbackTimer: number | undefined;
  // Until the first activity reports its own height; the pill starts compact, so it isn't seen.
  private expandedLogicalHeight = PILL_EXPANDED_MAX_LOGICAL_HEIGHT;
  private isCompanionSegmentShown = false;

  constructor(
    private readonly pillShellElements: PillShellElements,
    private readonly onExpandedContentVisibilityChange: (isExpandedContentVisible: boolean) => void,
  ) {
    this.applyExpandedHeightCssVariable();
    pillShellElements.pillShellElement.addEventListener("transitionend", (transitionEndEvent) => {
      const isSizeTransition = transitionEndEvent.propertyName === "height" || transitionEndEvent.propertyName === "width";
      if (transitionEndEvent.target === pillShellElements.pillShellElement && isSizeTransition) {
        this.settleInteractiveArea();
      }
    });
  }

  // After the window was placed again (another monitor, another scale), the interactive
  // area must be sent again for whatever shape the pill has right now.
  applyCurrentInteractiveArea(): Promise<void> {
    return this.applyInteractiveArea(this.isExpanded());
  }

  showContentLayout(pillContentLayout: PillContentLayout): void {
    this.showExpandedContentHeight(pillContentLayout.expandedLogicalHeight);
    this.showCompanionSegment(pillContentLayout.isCompanionSegmentShown);
  }

  showExpansionState(expansionState: PillExpansionState): void {
    window.clearTimeout(this.settleFallbackTimer);
    const pillShellClassList = this.pillShellElements.pillShellElement.classList;
    if (expansionState === "expanded") {
      this.applyInteractiveArea(true).catch(reportInteractiveAreaFailure);
      pillShellClassList.add(EXPANDED_SHELL_CLASS);
      this.onExpandedContentVisibilityChange(true);
      return;
    }
    pillShellClassList.remove(EXPANDED_SHELL_CLASS);
    this.settleAfterMorph();
  }

  // The open height only shows while open; while compact, only the CSS variable changes.
  private showExpandedContentHeight(requestedLogicalHeight: number): void {
    const nextLogicalHeight = Math.min(requestedLogicalHeight, PILL_EXPANDED_MAX_LOGICAL_HEIGHT);
    if (nextLogicalHeight === this.expandedLogicalHeight) {
      return;
    }
    const isGrowing = nextLogicalHeight > this.expandedLogicalHeight;
    const isExpanded = this.isExpanded();
    this.expandedLogicalHeight = nextLogicalHeight;
    if (isExpanded && isGrowing) {
      this.applyInteractiveArea(true).catch(reportInteractiveAreaFailure);
    }
    this.applyExpandedHeightCssVariable();
    if (isExpanded && !isGrowing) {
      this.settleAfterMorph();
    }
  }

  // The companion segment only widens the compact pill; while open, only the class changes.
  private showCompanionSegment(isCompanionSegmentShown: boolean): void {
    if (isCompanionSegmentShown === this.isCompanionSegmentShown) {
      return;
    }
    const isCompact = !this.isExpanded();
    this.isCompanionSegmentShown = isCompanionSegmentShown;
    if (isCompact && isCompanionSegmentShown) {
      this.applyInteractiveArea(false).catch(reportInteractiveAreaFailure);
    }
    this.pillShellElements.pillShellElement.classList.toggle(WITH_COMPANION_SHELL_CLASS, isCompanionSegmentShown);
    if (isCompact && !isCompanionSegmentShown) {
      this.settleAfterMorph();
    }
  }

  // If the browser skips the transitionend event, a timer settles the pill instead.
  private settleAfterMorph(): void {
    window.clearTimeout(this.settleFallbackTimer);
    this.settleFallbackTimer = window.setTimeout(
      () => this.settleInteractiveArea(),
      PILL_MORPH_DURATION_MILLISECONDS + PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS,
    );
  }

  // Reached when a size animation ends or by the fallback timer. It sends the area for the
  // shape the pill has now, so a late call after the pill changed its mind again is harmless.
  private settleInteractiveArea(): void {
    window.clearTimeout(this.settleFallbackTimer);
    const isExpanded = this.isExpanded();
    this.applyInteractiveArea(isExpanded).catch(reportInteractiveAreaFailure);
    if (!isExpanded) {
      this.onExpandedContentVisibilityChange(false);
    }
  }

  private applyInteractiveArea(isExpanded: boolean): Promise<void> {
    const interactiveArea = calculatePillInteractiveArea(
      isExpanded
        ? { expansionState: "expanded", expandedLogicalHeight: this.expandedLogicalHeight }
        : { expansionState: "compact", isCompanionSegmentShown: this.isCompanionSegmentShown },
    );
    return requestPillInteractiveArea(
      interactiveArea.logicalLeft,
      interactiveArea.logicalTop,
      interactiveArea.logicalWidth,
      interactiveArea.logicalHeight,
    );
  }

  private applyExpandedHeightCssVariable(): void {
    this.pillShellElements.pillShellElement.style.setProperty(
      EXPANDED_HEIGHT_CSS_VARIABLE,
      `${this.expandedLogicalHeight}px`,
    );
  }

  private isExpanded(): boolean {
    return this.pillShellElements.pillShellElement.classList.contains(EXPANDED_SHELL_CLASS);
  }
}

function reportInteractiveAreaFailure(interactiveAreaError: unknown): void {
  console.error("Crest could not update the pill's interactive area:", interactiveAreaError);
}

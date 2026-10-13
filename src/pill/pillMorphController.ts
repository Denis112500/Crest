import { PILL_DASHBOARD_TILE_LOGICAL_WIDTH, PILL_EXPANDED_MAX_LOGICAL_HEIGHT } from "../frontendConstants";
import { requestPillInteractiveArea } from "../ipc/requestPillInteractiveArea";
import { calculatePillInteractiveArea, type PillShape } from "./calculatePillInteractiveArea";
import { PillMorphSettleDetector } from "./pillMorphSettleDetector";
import type { PillShellElements } from "./pillShellElements";
import type { PillExpansionState } from "./pillStateMachine";

const EXPANDED_SHELL_CLASS = "is-expanded";
const WITH_COMPANION_SHELL_CLASS = "has-companion";
const EXPANDED_WIDTH_CSS_VARIABLE = "--pill-expanded-width";
const EXPANDED_HEIGHT_CSS_VARIABLE = "--pill-expanded-height";

/** What the content needs from the pill's shape (reported by PillContentPresenter). */
export interface PillContentLayout {
  expandedLogicalWidth: number;
  expandedLogicalHeight: number;
  isCompanionMarkShown: boolean;
}

// Carries out a change of state or size in the right order. Growing (opening, more tiles or
// taller content while open, a companion appearing while compact): first enlarge the
// interactive area, then animate, so the growing pill is never clipped. Shrinking: animate
// first, and only shrink the interactive area once the pill has settled.
export class PillMorphController {
  private readonly morphSettleDetector: PillMorphSettleDetector;
  // Until the first activity reports its own size; the pill starts compact, so it isn't seen.
  private expandedLogicalWidth = PILL_DASHBOARD_TILE_LOGICAL_WIDTH;
  private expandedLogicalHeight = PILL_EXPANDED_MAX_LOGICAL_HEIGHT;
  private isCompanionMarkShown = false;

  constructor(
    private readonly pillShellElements: PillShellElements,
    private readonly onExpandedContentVisibilityChange: (isExpandedContentVisible: boolean) => void,
  ) {
    this.applyExpandedSizeCssVariables();
    this.morphSettleDetector = new PillMorphSettleDetector(pillShellElements.pillShellElement, () =>
      this.settleInteractiveArea(),
    );
  }

  // After the window was placed again (another monitor, another scale), the interactive
  // area must be sent again for whatever shape the pill has right now.
  applyCurrentInteractiveArea(): Promise<void> {
    return this.applyInteractiveArea(this.isExpanded());
  }

  showContentLayout(pillContentLayout: PillContentLayout): void {
    this.showExpandedContentSize(pillContentLayout.expandedLogicalWidth, pillContentLayout.expandedLogicalHeight);
    this.showCompanionMark(pillContentLayout.isCompanionMarkShown);
  }

  showExpansionState(expansionState: PillExpansionState): void {
    const pillShellClassList = this.pillShellElements.pillShellElement.classList;
    if (expansionState === "expanded") {
      this.morphSettleDetector.stopWaiting();
      this.applyInteractiveArea(true).catch(reportInteractiveAreaFailure);
      pillShellClassList.add(EXPANDED_SHELL_CLASS);
      this.onExpandedContentVisibilityChange(true);
      return;
    }
    pillShellClassList.remove(EXPANDED_SHELL_CLASS);
    this.morphSettleDetector.waitForSettle();
  }

  // The open size only shows while open; while compact, only the CSS variables change. One
  // side can grow while the other shrinks (a tall alert gives way to a wide, lower dashboard):
  // the area then covers both sizes until the pill settles.
  private showExpandedContentSize(nextLogicalWidth: number, requestedLogicalHeight: number): void {
    const nextLogicalHeight = Math.min(requestedLogicalHeight, PILL_EXPANDED_MAX_LOGICAL_HEIGHT);
    if (nextLogicalWidth === this.expandedLogicalWidth && nextLogicalHeight === this.expandedLogicalHeight) {
      return;
    }
    const isGrowing = nextLogicalWidth > this.expandedLogicalWidth || nextLogicalHeight > this.expandedLogicalHeight;
    const isShrinking = nextLogicalWidth < this.expandedLogicalWidth || nextLogicalHeight < this.expandedLogicalHeight;
    const isExpanded = this.isExpanded();
    if (isExpanded && isGrowing) {
      const shapeCoveringBothSizes: PillShape = {
        expansionState: "expanded",
        expandedLogicalWidth: Math.max(nextLogicalWidth, this.expandedLogicalWidth),
        expandedLogicalHeight: Math.max(nextLogicalHeight, this.expandedLogicalHeight),
      };
      this.sendInteractiveArea(shapeCoveringBothSizes).catch(reportInteractiveAreaFailure);
    }
    this.expandedLogicalWidth = nextLogicalWidth;
    this.expandedLogicalHeight = nextLogicalHeight;
    this.applyExpandedSizeCssVariables();
    if (isExpanded && isShrinking) {
      this.morphSettleDetector.waitForSettle();
    }
  }

  // The companion's mark only widens the compact pill; while open, only the class changes.
  private showCompanionMark(isCompanionMarkShown: boolean): void {
    if (isCompanionMarkShown === this.isCompanionMarkShown) {
      return;
    }
    const isCompact = !this.isExpanded();
    this.isCompanionMarkShown = isCompanionMarkShown;
    if (isCompact && isCompanionMarkShown) {
      this.applyInteractiveArea(false).catch(reportInteractiveAreaFailure);
    }
    this.pillShellElements.pillShellElement.classList.toggle(WITH_COMPANION_SHELL_CLASS, isCompanionMarkShown);
    if (isCompact && !isCompanionMarkShown) {
      this.morphSettleDetector.waitForSettle();
    }
  }

  // Reached when a size animation ends (PillMorphSettleDetector). It sends the area for the
  // shape the pill has now, so a late call after the pill changed its mind again is harmless.
  private settleInteractiveArea(): void {
    const isExpanded = this.isExpanded();
    this.applyInteractiveArea(isExpanded).catch(reportInteractiveAreaFailure);
    if (!isExpanded) {
      this.onExpandedContentVisibilityChange(false);
    }
  }

  private applyInteractiveArea(isExpanded: boolean): Promise<void> {
    return this.sendInteractiveArea(
      isExpanded
        ? {
            expansionState: "expanded",
            expandedLogicalWidth: this.expandedLogicalWidth,
            expandedLogicalHeight: this.expandedLogicalHeight,
          }
        : { expansionState: "compact", isCompanionMarkShown: this.isCompanionMarkShown },
    );
  }

  private sendInteractiveArea(pillShape: PillShape): Promise<void> {
    const interactiveArea = calculatePillInteractiveArea(pillShape);
    return requestPillInteractiveArea(
      interactiveArea.logicalLeft,
      interactiveArea.logicalTop,
      interactiveArea.logicalWidth,
      interactiveArea.logicalHeight,
    );
  }

  // Set on the shell, so the expanded layer inside inherits them.
  private applyExpandedSizeCssVariables(): void {
    const pillShellStyle = this.pillShellElements.pillShellElement.style;
    pillShellStyle.setProperty(EXPANDED_WIDTH_CSS_VARIABLE, `${this.expandedLogicalWidth}px`);
    pillShellStyle.setProperty(EXPANDED_HEIGHT_CSS_VARIABLE, `${this.expandedLogicalHeight}px`);
  }

  private isExpanded(): boolean {
    return this.pillShellElements.pillShellElement.classList.contains(EXPANDED_SHELL_CLASS);
  }
}

function reportInteractiveAreaFailure(interactiveAreaError: unknown): void {
  console.error("Crest could not update the pill's interactive area:", interactiveAreaError);
}

import {
  PILL_ATTENTION_PEEK_DURATION_MILLISECONDS,
  PILL_COLLAPSE_DELAY_AFTER_POINTER_LEAVES_MILLISECONDS,
  PILL_HOVER_EXPAND_DELAY_MILLISECONDS,
} from "../frontendConstants";

export type PillExpansionState = "compact" | "expanded";

// Decides when the pill is compact or expanded. It only reports transitions and knows
// nothing about the DOM or Tauri, so the rules are all in one readable place.
export class PillStateMachine {
  private currentExpansionState: PillExpansionState = "compact";
  private isPointerCurrentlyOverPill = false;
  private pendingHoverExpandTimer: number | undefined;
  private pendingCollapseTimer: number | undefined;
  private attentionPeekEndTimer: number | undefined;

  constructor(private readonly onExpansionStateChange: (expansionState: PillExpansionState) => void) {}

  get isPointerOverPill(): boolean {
    return this.isPointerCurrentlyOverPill;
  }

  handlePointerEntered(): void {
    this.isPointerCurrentlyOverPill = true;
    window.clearTimeout(this.pendingCollapseTimer);
    // While hovered, a running peek must not close the pill under the pointer.
    window.clearTimeout(this.attentionPeekEndTimer);
    if (this.currentExpansionState === "compact") {
      window.clearTimeout(this.pendingHoverExpandTimer);
      this.pendingHoverExpandTimer = window.setTimeout(
        () => this.transitionTo("expanded"),
        PILL_HOVER_EXPAND_DELAY_MILLISECONDS,
      );
    }
  }

  handlePointerLeft(): void {
    this.isPointerCurrentlyOverPill = false;
    window.clearTimeout(this.pendingHoverExpandTimer);
    if (this.currentExpansionState === "expanded") {
      window.clearTimeout(this.pendingCollapseTimer);
      this.pendingCollapseTimer = window.setTimeout(
        () => this.transitionTo("compact"),
        PILL_COLLAPSE_DELAY_AFTER_POINTER_LEAVES_MILLISECONDS,
      );
    }
  }

  handlePointerClicked(): void {
    window.clearTimeout(this.pendingHoverExpandTimer);
    this.transitionTo("expanded");
  }

  /** Something new (a track change): open for a few seconds, then close unless hovered. */
  handleAttentionRequested(): void {
    window.clearTimeout(this.pendingCollapseTimer);
    window.clearTimeout(this.attentionPeekEndTimer);
    this.transitionTo("expanded");
    this.attentionPeekEndTimer = window.setTimeout(() => {
      if (!this.isPointerCurrentlyOverPill) {
        this.transitionTo("compact");
      }
    }, PILL_ATTENTION_PEEK_DURATION_MILLISECONDS);
  }

  /** The window was hidden: no pointer can be over it, and it must come back compact. */
  handlePillConcealed(): void {
    window.clearTimeout(this.pendingHoverExpandTimer);
    window.clearTimeout(this.pendingCollapseTimer);
    window.clearTimeout(this.attentionPeekEndTimer);
    this.isPointerCurrentlyOverPill = false;
    this.transitionTo("compact");
  }

  private transitionTo(nextExpansionState: PillExpansionState): void {
    if (nextExpansionState === this.currentExpansionState) {
      return;
    }
    this.currentExpansionState = nextExpansionState;
    this.onExpansionStateChange(nextExpansionState);
  }
}

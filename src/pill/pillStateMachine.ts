import { PILL_COLLAPSE_DELAY_AFTER_POINTER_LEAVES_MILLISECONDS, PILL_HOVER_EXPAND_DELAY_MILLISECONDS } from "../frontendConstants";

export type PillExpansionState = "compact" | "expanded";

// Decides when the pill is compact or expanded. It only reports transitions and knows
// nothing about the DOM or Tauri, so the rules are all in one readable place.
export class PillStateMachine {
  private currentExpansionState: PillExpansionState = "compact";
  private isPointerCurrentlyOverPill = false;
  private pendingHoverExpandTimer: number | undefined;
  private pendingCollapseTimer: number | undefined;
  private attentionPeekEndTimer: number | undefined;
  private isHeldOpenByAlert = false;

  constructor(private readonly onExpansionStateChange: (expansionState: PillExpansionState) => void) {}

  get isPointerOverPill(): boolean {
    return this.isPointerCurrentlyOverPill;
  }

  handlePointerEntered(): void {
    this.isPointerCurrentlyOverPill = true;
    window.clearTimeout(this.pendingCollapseTimer);
    // While hovered, a running peek must not close the pill under the pointer.
    this.stopAttentionPeek();
    this.startHoverExpandCountdownIfCompact();
  }

  /** Over the companion's mark the pill doesn't open by itself: hover and a click there would
      fight (it would open before most clicks land). A click on it opens the pill instead. */
  handlePointerEnteredCompanion(): void {
    window.clearTimeout(this.pendingHoverExpandTimer);
  }

  /** Back over the main activity (or leaving the pill, which handlePointerLeft settles). */
  handlePointerLeftCompanion(): void {
    if (this.isPointerCurrentlyOverPill) {
      this.startHoverExpandCountdownIfCompact();
    }
  }

  private startHoverExpandCountdownIfCompact(): void {
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
    if (this.currentExpansionState === "expanded" && !this.isHeldOpenByAlert) {
      window.clearTimeout(this.pendingCollapseTimer);
      this.pendingCollapseTimer = window.setTimeout(
        () => this.transitionTo("compact"),
        PILL_COLLAPSE_DELAY_AFTER_POINTER_LEAVES_MILLISECONDS,
      );
    }
  }

  /** The closed pill lets clicks through and a hidden one isn't there, so neither gets a
      mouseenter; Rust reports the pointer resting on the closed pill, or against the top edge
      where the hidden one hangs. The pointer is over the pill now, which the page couldn't see. */
  handlePointerRestedOnPill(): void {
    this.isPointerCurrentlyOverPill = true;
    window.clearTimeout(this.pendingCollapseTimer);
    this.stopAttentionPeek();
    this.transitionTo("expanded");
  }

  handlePointerClicked(): void {
    window.clearTimeout(this.pendingHoverExpandTimer);
    this.transitionTo("expanded");
  }

  /** Something new (a track change, Claude Code's news): open for `peekDurationMilliseconds`,
      then close unless hovered or an alert holds the pill open. */
  handleAttentionRequested(peekDurationMilliseconds: number): void {
    window.clearTimeout(this.pendingCollapseTimer);
    this.stopAttentionPeek();
    this.transitionTo("expanded");
    this.attentionPeekEndTimer = window.setTimeout(() => {
      this.attentionPeekEndTimer = undefined;
      if (!this.isPointerCurrentlyOverPill && !this.isHeldOpenByAlert) {
        this.transitionTo("compact");
      }
    }, peekDurationMilliseconds);
  }

  private stopAttentionPeek(): void {
    window.clearTimeout(this.attentionPeekEndTimer);
    this.attentionPeekEndTimer = undefined;
  }

  /** An alert (a permission request) leads: the pill opens and stays open until it ends, so the
      buttons can't close under the user. When it ends, the pill closes unless hovered, or unless
      the answer itself was news (a Deny), whose peek was requested just before. */
  setHeldOpenByAlert(isHeldOpenByAlert: boolean): void {
    if (isHeldOpenByAlert === this.isHeldOpenByAlert) {
      return;
    }
    this.isHeldOpenByAlert = isHeldOpenByAlert;
    window.clearTimeout(this.pendingCollapseTimer);
    if (isHeldOpenByAlert) {
      this.stopAttentionPeek();
      window.clearTimeout(this.pendingHoverExpandTimer);
      this.transitionTo("expanded");
    } else if (!this.isPointerCurrentlyOverPill && this.attentionPeekEndTimer === undefined) {
      this.transitionTo("compact");
    }
  }

  /** Shown again (after a fullscreen app): an alert that is still waiting opens the pill again. */
  handlePillRevealed(): void {
    if (this.isHeldOpenByAlert) {
      this.transitionTo("expanded");
    }
  }

  /** The window was hidden: no pointer can be over it, and it must come back compact. */
  handlePillConcealed(): void {
    window.clearTimeout(this.pendingHoverExpandTimer);
    window.clearTimeout(this.pendingCollapseTimer);
    this.stopAttentionPeek();
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

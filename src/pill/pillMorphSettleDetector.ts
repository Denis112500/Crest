import { PILL_MORPH_DURATION_MILLISECONDS, PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS } from "../frontendConstants";

// Tells when the pill's size animation has ended: the capsule's own width or height
// transitionend, or a timer shortly after the animation should have ended, in case the browser
// skips the event (e.g. nothing actually changed). Split from PillMorphController (code review
// before 0.4.0), which decides what to do once the pill has settled.
export class PillMorphSettleDetector {
  private settleFallbackTimer: number | undefined;

  constructor(
    pillShellElement: HTMLElement,
    private readonly onMorphSettled: () => void,
  ) {
    pillShellElement.addEventListener("transitionend", (transitionEndEvent) => {
      const isSizeTransition = transitionEndEvent.propertyName === "height" || transitionEndEvent.propertyName === "width";
      if (transitionEndEvent.target === pillShellElement && isSizeTransition) {
        this.reportSettled();
      }
    });
  }

  /** An animation that ends in a smaller pill has started: report when it ends. */
  waitForSettle(): void {
    window.clearTimeout(this.settleFallbackTimer);
    this.settleFallbackTimer = window.setTimeout(
      () => this.reportSettled(),
      PILL_MORPH_DURATION_MILLISECONDS + PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS,
    );
  }

  /** The pill is growing again: nothing to wait for. */
  stopWaiting(): void {
    window.clearTimeout(this.settleFallbackTimer);
  }

  private reportSettled(): void {
    window.clearTimeout(this.settleFallbackTimer);
    this.onMorphSettled();
  }
}

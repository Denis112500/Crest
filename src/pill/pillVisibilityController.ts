import { PILL_CONCEAL_DURATION_MILLISECONDS, PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS } from "../frontendConstants";
import { requestPillWindowConceal } from "../ipc/requestPillWindowConceal";
import { requestPillWindowReveal } from "../ipc/requestPillWindowReveal";
import type { PillStateMachine } from "./pillStateMachine";
import type { PillVisibility } from "./pillVisibilityTypes";
import { waitUntilNextFrameIsPainted } from "./waitUntilNextFrameIsPainted";

const CONCEALED_SHELL_CLASS = "is-concealed";
const INSTANTLY_CONCEALED_SHELL_CLASS = "is-concealed-instantly";

// Carries out Rust's show/hide decisions with an animation. Showing: show the window
// first, then let the pill grow in. Hiding: let the pill fade out first, then hide the
// window. A hide never happens under the user's pointer; it waits until they leave.
// The exception is a fullscreen app in front: then the pill vanishes at once.
export class PillVisibilityController {
  private isNativeWindowShown = false;
  private latestRequestedVisibility = false;
  private isHideWaitingForPointerToLeave = false;
  private concealFinishFallbackTimer: number | undefined;

  constructor(
    private readonly pillShellElement: HTMLElement,
    private readonly pillStateMachine: PillStateMachine,
    private readonly onPillOnScreenChange: (isPillOnScreen: boolean) => void,
  ) {
    pillShellElement.classList.add(CONCEALED_SHELL_CLASS);
    // Registered after the state machine's own listener, so it already knows the pointer left.
    pillShellElement.addEventListener("mouseleave", () => {
      if (this.isHideWaitingForPointerToLeave) {
        this.concealPill();
      }
    });
    pillShellElement.addEventListener("transitionend", (transitionEndEvent) => {
      if (transitionEndEvent.target === pillShellElement && transitionEndEvent.propertyName === "opacity") {
        this.finishConcealIfStillConcealed();
      }
    });
  }

  showPillVisibility(pillVisibility: PillVisibility): void {
    this.latestRequestedVisibility = pillVisibility.isPillVisible;
    if (pillVisibility.isPillVisible) {
      this.revealPill().catch(reportVisibilityFailure);
    } else if (pillVisibility.isFullscreenAppInFront) {
      this.concealPillInstantly();
    } else {
      this.concealPill();
    }
  }

  private async revealPill(): Promise<void> {
    this.isHideWaitingForPointerToLeave = false;
    window.clearTimeout(this.concealFinishFallbackTimer);
    // Back to animated changes; removed while the pill is still concealed, so the grow-in
    // below is animated again.
    this.pillShellElement.classList.remove(INSTANTLY_CONCEALED_SHELL_CLASS);
    if (!this.isNativeWindowShown) {
      this.isNativeWindowShown = true;
      await requestPillWindowReveal();
      this.onPillOnScreenChange(true);
      await waitUntilNextFrameIsPainted();
    }
    // A hide may have been requested while the window was being shown.
    if (this.latestRequestedVisibility) {
      this.pillShellElement.classList.remove(CONCEALED_SHELL_CLASS);
    }
  }

  private concealPill(): void {
    if (this.pillStateMachine.isPointerOverPill) {
      this.isHideWaitingForPointerToLeave = true;
      return;
    }
    this.isHideWaitingForPointerToLeave = false;
    if (!this.isNativeWindowShown) {
      return;
    }
    this.pillShellElement.classList.add(CONCEALED_SHELL_CLASS);
    window.clearTimeout(this.concealFinishFallbackTimer);
    this.concealFinishFallbackTimer = window.setTimeout(
      () => this.finishConcealIfStillConcealed(),
      PILL_CONCEAL_DURATION_MILLISECONDS + PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS,
    );
  }

  // No fade over the game, and no waiting for the pointer to leave: a game captures the
  // pointer, so the pill might never hear that it left.
  private concealPillInstantly(): void {
    this.isHideWaitingForPointerToLeave = false;
    this.pillShellElement.classList.add(INSTANTLY_CONCEALED_SHELL_CLASS, CONCEALED_SHELL_CLASS);
    this.finishConcealIfStillConcealed();
  }

  private finishConcealIfStillConcealed(): void {
    const isStillConcealed = this.pillShellElement.classList.contains(CONCEALED_SHELL_CLASS);
    if (!isStillConcealed || !this.isNativeWindowShown || this.latestRequestedVisibility) {
      return;
    }
    window.clearTimeout(this.concealFinishFallbackTimer);
    this.isNativeWindowShown = false;
    this.pillStateMachine.handlePillConcealed();
    this.onPillOnScreenChange(false);
    requestPillWindowConceal().catch(reportVisibilityFailure);
  }
}

function reportVisibilityFailure(visibilityError: unknown): void {
  console.error("Crest could not change the pill's visibility:", visibilityError);
}

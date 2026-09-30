import { PILL_CONCEAL_DURATION_MILLISECONDS, PILL_MORPH_END_FALLBACK_SLACK_MILLISECONDS } from "../frontendConstants";
import { requestPillWindowConceal } from "../ipc/requestPillWindowConceal";
import { requestPillWindowReveal } from "../ipc/requestPillWindowReveal";
import type { PillStateMachine } from "./pillStateMachine";
import { waitUntilNextFrameIsPainted } from "./waitUntilNextFrameIsPainted";

const CONCEALED_SHELL_CLASS = "is-concealed";

// Carries out Rust's show/hide decisions with an animation. Showing: show the window
// first, then let the pill grow in. Hiding: let the pill fade out first, then hide the
// window. A hide never happens under the user's pointer; it waits until they leave.
export class PillVisibilityController {
  private isNativeWindowShown = false;
  private latestRequestedVisibility = false;
  private isHideWaitingForPointerToLeave = false;
  private concealFinishFallbackTimer: number | undefined;

  constructor(
    private readonly pillShellElement: HTMLElement,
    private readonly pillStateMachine: PillStateMachine,
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

  showPillVisibility(isPillVisible: boolean): void {
    this.latestRequestedVisibility = isPillVisible;
    if (isPillVisible) {
      this.revealPill().catch(reportVisibilityFailure);
    } else {
      this.concealPill();
    }
  }

  private async revealPill(): Promise<void> {
    this.isHideWaitingForPointerToLeave = false;
    window.clearTimeout(this.concealFinishFallbackTimer);
    if (!this.isNativeWindowShown) {
      this.isNativeWindowShown = true;
      await requestPillWindowReveal();
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

  private finishConcealIfStillConcealed(): void {
    const isStillConcealed = this.pillShellElement.classList.contains(CONCEALED_SHELL_CLASS);
    if (!isStillConcealed || !this.isNativeWindowShown || this.latestRequestedVisibility) {
      return;
    }
    window.clearTimeout(this.concealFinishFallbackTimer);
    this.isNativeWindowShown = false;
    this.pillStateMachine.handlePillConcealed();
    requestPillWindowConceal().catch(reportVisibilityFailure);
  }
}

function reportVisibilityFailure(visibilityError: unknown): void {
  console.error("Crest could not change the pill's visibility:", visibilityError);
}

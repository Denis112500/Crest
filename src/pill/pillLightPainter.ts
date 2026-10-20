/** One light for the pill: three colours along the edge, left to right, and how strongly they shine. */
export interface PillLightColors {
  startColor: string;
  middleColor: string;
  endColor: string;
  strength: number;
}

const SWEEPING_LIGHT_CLASS = "is-sweeping-light";
const COMMITTING_LIGHT_CLASS = "is-committing-light";
const LIGHT_SWEEP_ANIMATION_NAME = "pill-light-sweep";

// Puts a light on the notch's CSS variables (pillLight.css draws it). A change of colour is the
// sweep: the new colours travel along the edge behind a bright front and replace the old ones;
// when it ends they become the current colours. A change of strength alone (play / pause) fades.
// While the pill is off screen, changes apply at once.
export class PillLightPainter {
  private shownLightColors: PillLightColors | null = null;
  private sweepingLightColors: PillLightColors | null = null;
  private isPillOnScreen = false;

  constructor(private readonly pillNotchElement: HTMLElement) {
    pillNotchElement.addEventListener("animationend", (animationEndEvent) => {
      if (animationEndEvent.target === pillNotchElement && animationEndEvent.animationName === LIGHT_SWEEP_ANIMATION_NAME) {
        this.finishSweep();
      }
    });
  }

  showLight(nextLightColors: PillLightColors): void {
    const targetLightColors = this.sweepingLightColors ?? this.shownLightColors;
    if (targetLightColors && areSameLightColors(targetLightColors, nextLightColors)) {
      return;
    }
    this.finishSweep();
    if (!this.shownLightColors || !this.isPillOnScreen) {
      this.applyShownLightColors(nextLightColors, false);
    } else if (areSameHues(this.shownLightColors, nextLightColors)) {
      this.applyShownLightColors(nextLightColors, true);
    } else {
      this.startSweep(nextLightColors);
    }
  }

  /** Something new to see in the same colours (a new track, a new status line): one pass of the
      bright front over the current light. Skipped while a sweep runs or the pill is off screen. */
  sweepShownLight(): void {
    if (this.sweepingLightColors || !this.shownLightColors || !this.isPillOnScreen) {
      return;
    }
    this.startSweep(this.shownLightColors);
  }

  setPillOnScreen(isPillOnScreen: boolean): void {
    this.isPillOnScreen = isPillOnScreen;
    if (!isPillOnScreen) {
      this.finishSweep();
    }
  }

  private startSweep(incomingLightColors: PillLightColors): void {
    const notchStyle = this.pillNotchElement.style;
    notchStyle.setProperty("--pill-light-incoming-start-color", incomingLightColors.startColor);
    notchStyle.setProperty("--pill-light-incoming-middle-color", incomingLightColors.middleColor);
    notchStyle.setProperty("--pill-light-incoming-end-color", incomingLightColors.endColor);
    notchStyle.setProperty("--pill-light-incoming-strength", String(incomingLightColors.strength));
    this.sweepingLightColors = incomingLightColors;
    this.pillNotchElement.classList.add(SWEEPING_LIGHT_CLASS);
  }

  // The incoming colours have covered the whole edge (or the sweep is cut short): they become the
  // current ones at once, without the fade, so the old colours can't flash back for a moment.
  private finishSweep(): void {
    if (!this.sweepingLightColors) {
      return;
    }
    const sweptLightColors = this.sweepingLightColors;
    this.sweepingLightColors = null;
    this.applyShownLightColors(sweptLightColors, false);
    this.pillNotchElement.classList.remove(SWEEPING_LIGHT_CLASS);
  }

  private applyShownLightColors(lightColors: PillLightColors, isFaded: boolean): void {
    if (!isFaded) {
      this.pillNotchElement.classList.add(COMMITTING_LIGHT_CLASS);
    }
    const notchStyle = this.pillNotchElement.style;
    notchStyle.setProperty("--pill-light-start-color", lightColors.startColor);
    notchStyle.setProperty("--pill-light-middle-color", lightColors.middleColor);
    notchStyle.setProperty("--pill-light-end-color", lightColors.endColor);
    notchStyle.setProperty("--pill-light-strength", String(lightColors.strength));
    if (!isFaded) {
      // Lets the new values take effect while transitions are off, before turning them back on.
      void this.pillNotchElement.offsetWidth;
      this.pillNotchElement.classList.remove(COMMITTING_LIGHT_CLASS);
    }
    this.shownLightColors = lightColors;
  }
}

function areSameHues(firstLightColors: PillLightColors, secondLightColors: PillLightColors): boolean {
  return (
    firstLightColors.startColor === secondLightColors.startColor &&
    firstLightColors.middleColor === secondLightColors.middleColor &&
    firstLightColors.endColor === secondLightColors.endColor
  );
}

function areSameLightColors(firstLightColors: PillLightColors, secondLightColors: PillLightColors): boolean {
  return areSameHues(firstLightColors, secondLightColors) && firstLightColors.strength === secondLightColors.strength;
}

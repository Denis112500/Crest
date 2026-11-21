import {
  SYSTEM_VOLUME_WHEEL_NOTCH_LINES,
  SYSTEM_VOLUME_WHEEL_NOTCH_PAGES,
  SYSTEM_VOLUME_WHEEL_NOTCH_PIXELS,
  SYSTEM_VOLUME_WHEEL_STEP_PERCENT,
} from "../../frontendConstants";

const PERCENT_PER_FULL_LEVEL = 100;
const MUTED_CLASS = "is-system-volume-muted";
const LEVEL_CSS_VARIABLE = "--system-volume-level";

export interface SystemVolumeSlider {
  /** The line with its knob, and the level as a number after it. */
  sliderElement: HTMLElement;
  showLevel(level: number, isMuted: boolean): void;
  isBeingDragged(): boolean;
}

// A real <input type="range"> (0–100 in whole steps), so screen readers find a slider; its own
// track is invisible and the line under it is Crest's (the cover ring's look). Each step goes to
// `onLevelChosen` (`isFinalStep` once the drag or click is over); the wheel moves it too.
export function createSystemVolumeSlider(onLevelChosen: (level: number, isFinalStep: boolean) => void): SystemVolumeSlider {
  const sliderElement = document.createElement("div");
  sliderElement.className = "system-volume-slider";
  const sliderTrackElement = document.createElement("div");
  sliderTrackElement.className = "system-volume-slider-track";
  const sliderFillElement = document.createElement("div");
  sliderFillElement.className = "system-volume-slider-fill";
  const sliderInputElement = document.createElement("input");
  sliderInputElement.type = "range";
  sliderInputElement.className = "system-volume-slider-input";
  sliderInputElement.min = "0";
  sliderInputElement.max = String(PERCENT_PER_FULL_LEVEL);
  sliderInputElement.step = "1";
  sliderInputElement.setAttribute("aria-label", "System volume");
  const levelReadoutElement = document.createElement("span");
  levelReadoutElement.className = "system-volume-readout";
  levelReadoutElement.setAttribute("aria-hidden", "true");
  sliderTrackElement.append(sliderFillElement, sliderInputElement);
  sliderElement.append(sliderTrackElement, levelReadoutElement);

  let isPointerDragging = false;
  // A notched wheel sends one big delta per notch, a touchpad or smooth wheel many small ones per
  // gesture. Stepping on every event made those race (Reddit feedback, 2026-11-18), so small deltas
  // add up here until they make a whole notch.
  // Kept in the event's own units (usually whole pixels): summing notch fractions instead drifts
  // (ten 0.1s make 0.999…) and loses a notch.
  let pendingWheelDelta = 0;

  const countWheelNotches = (wheelEvent: WheelEvent): number => {
    const wheelDeltaOfOneNotch = wheelDeltaPerNotch(wheelEvent.deltaMode);
    if (Math.abs(wheelEvent.deltaY) >= wheelDeltaOfOneNotch) {
      pendingWheelDelta = 0;
      return Math.round(wheelEvent.deltaY / wheelDeltaOfOneNotch);
    }
    // Turning back starts afresh, so the first notch the other way isn't eaten by the leftover.
    if (wheelEvent.deltaY * pendingWheelDelta < 0) {
      pendingWheelDelta = 0;
    }
    pendingWheelDelta += wheelEvent.deltaY;
    const wholeWheelNotches = Math.trunc(pendingWheelDelta / wheelDeltaOfOneNotch);
    pendingWheelDelta -= wholeWheelNotches * wheelDeltaOfOneNotch;
    return wholeWheelNotches;
  };

  const paintLevel = (levelPercent: number, isMuted: boolean): void => {
    sliderElement.style.setProperty(LEVEL_CSS_VARIABLE, String(levelPercent / PERCENT_PER_FULL_LEVEL));
    sliderElement.classList.toggle(MUTED_CLASS, isMuted);
    // Muted keeps the level (it comes back on unmute) but shows a dash, as the mockup did.
    levelReadoutElement.textContent = isMuted ? "—" : String(levelPercent);
    sliderInputElement.setAttribute("aria-valuetext", isMuted ? `Muted, ${levelPercent} %` : `${levelPercent} %`);
  };
  const chooseLevelPercent = (levelPercent: number, isFinalStep: boolean): void => {
    paintLevel(levelPercent, false);
    onLevelChosen(levelPercent / PERCENT_PER_FULL_LEVEL, isFinalStep);
  };

  sliderInputElement.addEventListener("input", () => chooseLevelPercent(sliderInputElement.valueAsNumber, false));
  sliderInputElement.addEventListener("change", () => chooseLevelPercent(sliderInputElement.valueAsNumber, true));
  // Captured, the drag keeps the pointer "on" the slider even when it wanders off the pill, so
  // the pill doesn't start closing under a drag.
  sliderInputElement.addEventListener("pointerdown", (pointerDownEvent) => {
    isPointerDragging = true;
    sliderInputElement.setPointerCapture(pointerDownEvent.pointerId);
  });
  sliderInputElement.addEventListener("lostpointercapture", () => {
    isPointerDragging = false;
  });
  sliderElement.addEventListener(
    "wheel",
    (wheelEvent) => {
      wheelEvent.preventDefault();
      const wheelNotches = countWheelNotches(wheelEvent);
      if (wheelNotches === 0) {
        return;
      }
      // Scrolling down (positive deltaY) turns the volume down.
      const wheelStep = -wheelNotches * SYSTEM_VOLUME_WHEEL_STEP_PERCENT;
      const steppedLevelPercent = Math.min(Math.max(sliderInputElement.valueAsNumber + wheelStep, 0), PERCENT_PER_FULL_LEVEL);
      if (steppedLevelPercent !== sliderInputElement.valueAsNumber) {
        sliderInputElement.valueAsNumber = steppedLevelPercent;
        chooseLevelPercent(steppedLevelPercent, false);
      }
    },
    // Not passive: the wheel turns the volume, nothing else.
    { passive: false },
  );

  return {
    sliderElement,
    showLevel(level, isMuted) {
      const levelPercent = Math.round(level * PERCENT_PER_FULL_LEVEL);
      sliderInputElement.valueAsNumber = levelPercent;
      paintLevel(levelPercent, isMuted);
    },
    isBeingDragged: () => isPointerDragging,
  };
}

function wheelDeltaPerNotch(wheelDeltaMode: number): number {
  if (wheelDeltaMode === WheelEvent.DOM_DELTA_LINE) {
    return SYSTEM_VOLUME_WHEEL_NOTCH_LINES;
  }
  if (wheelDeltaMode === WheelEvent.DOM_DELTA_PAGE) {
    return SYSTEM_VOLUME_WHEEL_NOTCH_PAGES;
  }
  return SYSTEM_VOLUME_WHEEL_NOTCH_PIXELS;
}

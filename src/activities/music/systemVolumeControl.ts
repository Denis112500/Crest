// Its speaker is a `music-control-button`, also in the idle tile, where no music view loads it.
import "../../styles/musicControlButtons.css";
import "../../styles/systemVolumeControl.css";

import {
  SYSTEM_VOLUME_SET_MIN_INTERVAL_MILLISECONDS,
  SYSTEM_VOLUME_SLIDER_ARMING_DELAY_MILLISECONDS,
  SYSTEM_VOLUME_SLIDER_AUTO_CLOSE_MILLISECONDS,
  SYSTEM_VOLUME_SPEAKER_TWO_WAVES_FROM_LEVEL,
} from "../../frontendConstants";
import { GET_CURRENT_SYSTEM_VOLUME_COMMAND, SYSTEM_VOLUME_CHANGED_EVENT } from "../../ipc/ipcChannelNames";
import { createRustStateFollower } from "../../ipc/createRustStateFollower";
import { requestSystemVolumeLevel } from "../../ipc/requestSystemVolumeLevel";
import { createLatestValueThrottle } from "./createLatestValueThrottle";
import { createSystemVolumeSlider } from "./systemVolumeSlider";
import { createSystemVolumeSpeakerIcons, type SpeakerIconLevel } from "./systemVolumeSpeakerIcons";
import type { SystemVolume, SystemVolumeSliderMode } from "./systemVolumeTypes";

const SLIDER_OPEN_CLASS = "is-system-volume-open";
const SLIDER_ARMING_CLASS = "is-system-volume-arming";
const SPEAKER_INDICATOR_CLASS = "is-speaker-indicator";

export interface SystemVolumeControl {
  speakerButtonElement: HTMLButtonElement;
  sliderElement: HTMLElement;
  /** Follows Rust's system volume only while the open tile is seen; unseen, the slider closes. */
  setSeen(isSeen: boolean): void;
}

// The speaker button in the music tile's control row and the slider it slides out (layout C,
// decided 2026-10-17): open, the row's other controls give way to the slider. It closes on a
// second click, or by itself a few seconds after it was last touched (never while the pointer
// is on it). Nothing runs while it's closed and nobody sees the tile. "alwaysOpen" (the idle tile)
// keeps the slider out whenever there is an output device, and the speaker only shows the level.
export function createSystemVolumeControl(
  controlRowElement: HTMLElement,
  elementsGivingWayToSlider: HTMLElement[],
  onSliderOpenChanged: (isSliderOpen: boolean) => void,
  sliderMode: SystemVolumeSliderMode,
): SystemVolumeControl {
  const isSliderAlwaysOpen = sliderMode === "alwaysOpen";
  const speakerButtonElement = document.createElement("button");
  speakerButtonElement.type = "button";
  speakerButtonElement.className = "music-control-button system-volume-speaker-button";
  speakerButtonElement.setAttribute("aria-label", "Volume");
  speakerButtonElement.append(...createSystemVolumeSpeakerIcons());
  if (isSliderAlwaysOpen) {
    // Nothing to click: the slider next to it is the control.
    speakerButtonElement.classList.add(SPEAKER_INDICATOR_CLASS);
    speakerButtonElement.tabIndex = -1;
    speakerButtonElement.setAttribute("aria-hidden", "true");
  }

  // `null` = no output device (also until Rust first answers): the speaker is greyed out.
  let shownSystemVolume: SystemVolume | null = null;
  let isSliderOpen = false;
  let isPointerOnSlider = false;
  let autoCloseTimer: number | undefined;
  let armingTimer: number | undefined;

  const levelRequestThrottle = createLatestValueThrottle((level: number) => {
    requestSystemVolumeLevel(level).catch((requestError: unknown) => {
      console.error("Crest could not set the system volume:", requestError);
    });
  }, SYSTEM_VOLUME_SET_MIN_INTERVAL_MILLISECONDS);
  const systemVolumeSlider = createSystemVolumeSlider((level, isFinalStep) => {
    // Setting a level unmutes (Rust does the same), and Rust won't send this change back.
    shownSystemVolume = { level, isMuted: false };
    showSpeakerIcon();
    levelRequestThrottle.offer(level);
    if (isFinalStep) {
      levelRequestThrottle.flush();
    }
    restartAutoClose();
  });
  const { sliderElement } = systemVolumeSlider;

  const showSpeakerIcon = (): void => {
    speakerButtonElement.dataset.speakerIconLevel = chooseSpeakerIconLevel(shownSystemVolume);
  };
  const showSystemVolume = (systemVolume: SystemVolume | null): void => {
    // Mid-drag the slider is the truth: its last position is sent when the drag ends.
    if (systemVolume !== null && systemVolumeSlider.isBeingDragged()) {
      return;
    }
    shownSystemVolume = systemVolume;
    showSpeakerIcon();
    speakerButtonElement.disabled = systemVolume === null;
    if (systemVolume === null) {
      setSliderOpen(false);
    } else {
      systemVolumeSlider.showLevel(systemVolume.level, systemVolume.isMuted);
      if (isSliderAlwaysOpen && !isSliderOpen) {
        setSliderOpen(true);
      }
    }
  };
  const restartAutoClose = (): void => {
    window.clearTimeout(autoCloseTimer);
    autoCloseTimer = undefined;
    if (!isSliderAlwaysOpen && isSliderOpen && !isPointerOnSlider && !systemVolumeSlider.isBeingDragged()) {
      autoCloseTimer = window.setTimeout(() => setSliderOpen(false), SYSTEM_VOLUME_SLIDER_AUTO_CLOSE_MILLISECONDS);
    }
  };
  const setSliderOpen = (isOpen: boolean): void => {
    const hasOpenChanged = isOpen !== isSliderOpen;
    isSliderOpen = isOpen;
    controlRowElement.classList.toggle(SLIDER_OPEN_CLASS, isOpen);
    speakerButtonElement.setAttribute("aria-expanded", String(isOpen));
    // Hidden controls can't be reached by a screen reader or a stray click while they're away.
    sliderElement.inert = !isOpen;
    for (const elementGivingWay of elementsGivingWayToSlider) {
      elementGivingWay.inert = isOpen;
    }
    // The speaker slides away and leaves the line under the pointer, so a quick second click
    // (meant to close) would set the volume there; the line ignores clicks for a moment.
    window.clearTimeout(armingTimer);
    sliderElement.classList.toggle(SLIDER_ARMING_CLASS, isOpen);
    if (isOpen) {
      armingTimer = window.setTimeout(
        () => sliderElement.classList.remove(SLIDER_ARMING_CLASS),
        SYSTEM_VOLUME_SLIDER_ARMING_DELAY_MILLISECONDS,
      );
    }
    restartAutoClose();
    if (hasOpenChanged) {
      onSliderOpenChanged(isOpen);
    }
  };

  if (!isSliderAlwaysOpen) {
    speakerButtonElement.addEventListener("click", () => setSliderOpen(!isSliderOpen));
  }
  sliderElement.addEventListener("pointerenter", () => {
    isPointerOnSlider = true;
    restartAutoClose();
  });
  sliderElement.addEventListener("pointerleave", () => {
    isPointerOnSlider = false;
    restartAutoClose();
  });
  sliderElement.addEventListener("lostpointercapture", restartAutoClose);
  setSliderOpen(false);
  showSystemVolume(null);

  const systemVolumeFollower = createRustStateFollower<SystemVolume | null>(
    SYSTEM_VOLUME_CHANGED_EVENT,
    GET_CURRENT_SYSTEM_VOLUME_COMMAND,
    showSystemVolume,
  );

  return {
    speakerButtonElement,
    sliderElement,
    setSeen(isSeen) {
      // Reported again with every arrangement; the follower ignores repeats.
      systemVolumeFollower.setFollowing(isSeen);
      if (!isSeen && isSliderOpen && !isSliderAlwaysOpen) {
        setSliderOpen(false);
      }
    },
  };
}

function chooseSpeakerIconLevel(systemVolume: SystemVolume | null): SpeakerIconLevel {
  if (systemVolume === null || systemVolume.isMuted || systemVolume.level === 0) {
    return "muted";
  }
  return systemVolume.level < SYSTEM_VOLUME_SPEAKER_TWO_WAVES_FROM_LEVEL ? "low" : "high";
}

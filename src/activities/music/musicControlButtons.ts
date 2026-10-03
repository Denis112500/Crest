import "../../styles/musicControlButtons.css";

import {
  MUSIC_ACTIVITY_KIND,
  MUSIC_NEXT_TRACK_ACTION,
  MUSIC_PREVIOUS_TRACK_ACTION,
  MUSIC_TOGGLE_PLAY_PAUSE_ACTION,
} from "../../ipc/ipcChannelNames";
import { requestActivityAction } from "../../ipc/requestActivityAction";
import { createSvgIconElement } from "../createSvgIconElement";
import type { NowPlayingControlAvailability } from "./nowPlayingTypes";

// Simple shapes on a 24×24 grid: a bar plus a triangle for skipping, a triangle for play,
// two bars for pause.
const PREVIOUS_TRACK_ICON_PATH = "M5.5 5H8v14H5.5zM19 5v14l-9.5-7z";
const NEXT_TRACK_ICON_PATH = "M16 5h2.5v14H16zM5 5v14l9.5-7z";
const PLAY_ICON_PATH = "M8 5v14l11-7z";
const PAUSE_ICON_PATH = "M7 5h3.5v14H7zM13.5 5H17v14h-3.5z";
const PLAYING_CLASS = "is-playing";

export interface MusicControlButtons {
  controlButtonsElement: HTMLElement;
  showIsPlaying(isPlaying: boolean): void;
  showAvailableControls(availableControls: NowPlayingControlAvailability): void;
}

// Previous / play-pause / next. Each press becomes an action for the Rust music source;
// the icons change only when the player confirms, through the next presentation update.
// A button the player doesn't accept is disabled: greyed out and unclickable.
export function createMusicControlButtons(): MusicControlButtons {
  const controlButtonsElement = document.createElement("div");
  controlButtonsElement.className = "music-control-buttons";

  const createControlButton = (
    accessibleLabel: string,
    musicAction: string,
    ...iconPaths: string[]
  ): HTMLButtonElement => {
    const controlButtonElement = document.createElement("button");
    controlButtonElement.type = "button";
    controlButtonElement.className = "music-control-button";
    controlButtonElement.setAttribute("aria-label", accessibleLabel);
    controlButtonElement.append(...iconPaths.map((iconPath) => createSvgIconElement(iconPath)));
    controlButtonElement.addEventListener("click", () => {
      requestActivityAction(MUSIC_ACTIVITY_KIND, musicAction).catch((actionError: unknown) => {
        console.error(`Crest could not send "${musicAction}" to the player:`, actionError);
      });
    });
    return controlButtonElement;
  };
  const previousTrackButton = createControlButton("Previous track", MUSIC_PREVIOUS_TRACK_ACTION, PREVIOUS_TRACK_ICON_PATH);
  // Both icons live in the button; CSS shows the one matching the playback state.
  const playPauseButton = createControlButton(
    "Play or pause",
    MUSIC_TOGGLE_PLAY_PAUSE_ACTION,
    PLAY_ICON_PATH,
    PAUSE_ICON_PATH,
  );
  playPauseButton.classList.add("music-play-pause-button");
  const nextTrackButton = createControlButton("Next track", MUSIC_NEXT_TRACK_ACTION, NEXT_TRACK_ICON_PATH);
  controlButtonsElement.append(previousTrackButton, playPauseButton, nextTrackButton);

  return {
    controlButtonsElement,
    showIsPlaying(isPlaying) {
      playPauseButton.classList.toggle(PLAYING_CLASS, isPlaying);
    },
    showAvailableControls(availableControls) {
      previousTrackButton.disabled = !availableControls.canSkipToPreviousTrack;
      playPauseButton.disabled = !availableControls.canTogglePlayPause;
      nextTrackButton.disabled = !availableControls.canSkipToNextTrack;
    },
  };
}

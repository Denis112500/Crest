import "../../styles/musicControlButtons.css";

import { createSvgIconElement } from "../createSvgIconElement";

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
}

// Previous / play-pause / next. Milestone (f) connects them to the player.
export function createMusicControlButtons(): MusicControlButtons {
  const controlButtonsElement = document.createElement("div");
  controlButtonsElement.className = "music-control-buttons";

  const createControlButton = (accessibleLabel: string, ...iconPaths: string[]): HTMLButtonElement => {
    const controlButtonElement = document.createElement("button");
    controlButtonElement.type = "button";
    controlButtonElement.className = "music-control-button";
    controlButtonElement.setAttribute("aria-label", accessibleLabel);
    controlButtonElement.append(...iconPaths.map((iconPath) => createSvgIconElement(iconPath)));
    return controlButtonElement;
  };
  const previousTrackButton = createControlButton("Previous track", PREVIOUS_TRACK_ICON_PATH);
  // Both icons live in the button; CSS shows the one matching the playback state.
  const playPauseButton = createControlButton("Play or pause", PLAY_ICON_PATH, PAUSE_ICON_PATH);
  playPauseButton.classList.add("music-play-pause-button");
  const nextTrackButton = createControlButton("Next track", NEXT_TRACK_ICON_PATH);
  controlButtonsElement.append(previousTrackButton, playPauseButton, nextTrackButton);

  return {
    controlButtonsElement,
    showIsPlaying(isPlaying) {
      playPauseButton.classList.toggle(PLAYING_CLASS, isPlaying);
    },
  };
}

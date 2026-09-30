import "../../styles/playbackBarsIndicator.css";

const PLAYBACK_BAR_COUNT = 4;
const PLAYING_CLASS = "is-playing";

export interface PlaybackBarsIndicator {
  barsIndicatorElement: HTMLElement;
  showIsPlaying(isPlaying: boolean): void;
}

// Bouncing bars while music plays, resting bars while paused. The animation is pure CSS
// (transform only), so the browser runs it without any JavaScript per frame.
export function createPlaybackBarsIndicator(): PlaybackBarsIndicator {
  const barsIndicatorElement = document.createElement("div");
  barsIndicatorElement.className = "playback-bars";
  barsIndicatorElement.setAttribute("aria-hidden", "true");
  for (let barIndex = 0; barIndex < PLAYBACK_BAR_COUNT; barIndex++) {
    const playbackBarElement = document.createElement("span");
    playbackBarElement.className = "playback-bar";
    barsIndicatorElement.append(playbackBarElement);
  }
  return {
    barsIndicatorElement,
    showIsPlaying(isPlaying) {
      barsIndicatorElement.classList.toggle(PLAYING_CLASS, isPlaying);
    },
  };
}

import "../../styles/playbackBarsIndicator.css";

import {
  PLAYBACK_BAR_BOUNCE_PERIODS_MILLISECONDS,
  PLAYBACK_BAR_RESTING_SCALE,
  PLAYBACK_BARS_FRAMES_PER_SECOND,
} from "../../frontendConstants";

const MILLISECONDS_PER_SECOND = 1000;

export interface PlaybackBarsIndicator {
  barsIndicatorElement: HTMLElement;
  showIsPlaying(isPlaying: boolean): void;
}

// Bouncing bars while music plays, resting bars while paused. A timer at a low fixed rate
// moves them instead of a CSS animation, which would redraw the window at the monitor's
// full refresh rate for as long as the music plays. Paused bars cost nothing.
export function createPlaybackBarsIndicator(): PlaybackBarsIndicator {
  const barsIndicatorElement = document.createElement("div");
  barsIndicatorElement.className = "playback-bars";
  barsIndicatorElement.setAttribute("aria-hidden", "true");
  const playbackBarElements = PLAYBACK_BAR_BOUNCE_PERIODS_MILLISECONDS.map(() => {
    const playbackBarElement = document.createElement("span");
    playbackBarElement.className = "playback-bar";
    playbackBarElement.style.transform = `scaleY(${PLAYBACK_BAR_RESTING_SCALE})`;
    return playbackBarElement;
  });
  barsIndicatorElement.append(...playbackBarElements);

  let bounceTimer: number | undefined;
  const drawBarHeights = (): void => {
    const now = performance.now();
    playbackBarElements.forEach((playbackBarElement, barIndex) => {
      const bouncePhase = (now / PLAYBACK_BAR_BOUNCE_PERIODS_MILLISECONDS[barIndex]) * 2 * Math.PI;
      // A cosine wave from 0 (rest) to 1 (full height), eased at both ends like a bounce.
      const bounceHeight = 0.5 - 0.5 * Math.cos(bouncePhase);
      const barScale = PLAYBACK_BAR_RESTING_SCALE + (1 - PLAYBACK_BAR_RESTING_SCALE) * bounceHeight;
      playbackBarElement.style.transform = `scaleY(${barScale})`;
    });
  };
  const restAllBars = (): void => {
    for (const playbackBarElement of playbackBarElements) {
      playbackBarElement.style.transform = `scaleY(${PLAYBACK_BAR_RESTING_SCALE})`;
    }
  };

  return {
    barsIndicatorElement,
    showIsPlaying(isPlaying) {
      if (isPlaying && bounceTimer === undefined) {
        bounceTimer = window.setInterval(drawBarHeights, MILLISECONDS_PER_SECOND / PLAYBACK_BARS_FRAMES_PER_SECOND);
      } else if (!isPlaying && bounceTimer !== undefined) {
        window.clearInterval(bounceTimer);
        bounceTimer = undefined;
        restAllBars();
      }
    },
  };
}

import type { NowPlayingTimeline } from "./nowPlayingTypes";

const MILLISECONDS_PER_SECOND = 1000;

/** Where the song is: position and duration in milliseconds, or null when the player has no
    timeline (a live stream). */
export type PlaybackPosition = { positionMilliseconds: number; durationMilliseconds: number } | null;

export interface PlaybackPositionTicker {
  showTimeline(timeline: NowPlayingTimeline | null, isPlaying: boolean): void;
  /** How often to redraw while playing; 0 while nobody sees it (no timer runs then). */
  setRedrawsPerSecond(redrawsPerSecond: number): void;
}

// Works out the song's position between Rust's reports (it reports a position and when it was
// taken, not every second) and hands it to a drawing function: once on every change, and on a
// slow timer only while the song plays and someone sees it. A slow timer, not
// requestAnimationFrame: rAF runs at the monitor's refresh rate (239 Hz here), far more often
// than a ring moving a fraction of a pixel per second needs.
export function createPlaybackPositionTicker(drawPlaybackPosition: (playbackPosition: PlaybackPosition) => void): PlaybackPositionTicker {
  let currentTimeline: NowPlayingTimeline | null = null;
  let isTrackPlaying = false;
  let currentRedrawsPerSecond = 0;
  let redrawTimer: number | undefined;

  const drawCurrentPosition = (): void => {
    if (!currentTimeline) {
      drawPlaybackPosition(null);
      return;
    }
    const millisecondsSinceReport = isTrackPlaying ? Date.now() - currentTimeline.positionReportedAtUnixMilliseconds : 0;
    const extrapolatedPosition = currentTimeline.reportedPositionMilliseconds + millisecondsSinceReport;
    drawPlaybackPosition({
      positionMilliseconds: Math.min(Math.max(extrapolatedPosition, 0), currentTimeline.trackDurationMilliseconds),
      durationMilliseconds: currentTimeline.trackDurationMilliseconds,
    });
  };
  const restartRedrawing = (): void => {
    window.clearInterval(redrawTimer);
    redrawTimer = undefined;
    drawCurrentPosition();
    if (currentRedrawsPerSecond > 0 && isTrackPlaying && currentTimeline) {
      redrawTimer = window.setInterval(drawCurrentPosition, MILLISECONDS_PER_SECOND / currentRedrawsPerSecond);
    }
  };

  return {
    showTimeline(timeline, isPlaying) {
      currentTimeline = timeline;
      isTrackPlaying = isPlaying;
      restartRedrawing();
    },
    setRedrawsPerSecond(redrawsPerSecond) {
      if (redrawsPerSecond !== currentRedrawsPerSecond) {
        currentRedrawsPerSecond = redrawsPerSecond;
        restartRedrawing();
      }
    },
  };
}

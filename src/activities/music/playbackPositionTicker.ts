import { PLAYBACK_POSITION_REDRAW_AFTER_WHOLE_SECOND_MILLISECONDS } from "../../frontendConstants";
import type { NowPlayingTimeline } from "./nowPlayingTypes";

const MILLISECONDS_PER_SECOND = 1000;

/** Where the song is: position and duration in milliseconds, or null when the player has no
    timeline (a live stream). */
export type PlaybackPosition = { positionMilliseconds: number; durationMilliseconds: number } | null;

export interface PlaybackPositionTicker {
  showTimeline(timeline: NowPlayingTimeline | null, isPlaying: boolean): void;
  /** Whether anyone sees the position; the timer runs only while seen and playing. */
  setPositionSeen(isPositionSeen: boolean): void;
  /** Draws the current position once, outside the timer (something that was hidden shows again). */
  redrawNow(): void;
}

// Works out the song's position between Rust's reports (it reports a position and when it was
// taken, not every second) and hands it to a drawing function: once on every change, and on each
// whole second of the song while it plays and someone sees it. Every redraw makes WebView2 draw
// a whole frame (paint, raster, composite, present), whatever little changed, so the ring and
// the clock move together in one frame a second: ticking 4 times a second cost the open pill
// about 1.8 % of a core (notes/pill-window-and-layout.md, "Open pill's CPU").
export function createPlaybackPositionTicker(drawPlaybackPosition: (playbackPosition: PlaybackPosition) => void): PlaybackPositionTicker {
  let currentTimeline: NowPlayingTimeline | null = null;
  let isTrackPlaying = false;
  let isPositionSeenNow = false;
  let redrawTimer: number | undefined;

  const extrapolatePositionMilliseconds = (timeline: NowPlayingTimeline): number => {
    const millisecondsSinceReport = isTrackPlaying ? Date.now() - timeline.positionReportedAtUnixMilliseconds : 0;
    const extrapolatedPosition = timeline.reportedPositionMilliseconds + millisecondsSinceReport;
    return Math.min(Math.max(extrapolatedPosition, 0), timeline.trackDurationMilliseconds);
  };
  const drawCurrentPosition = (): void => {
    if (!currentTimeline) {
      drawPlaybackPosition(null);
      return;
    }
    drawPlaybackPosition({
      positionMilliseconds: extrapolatePositionMilliseconds(currentTimeline),
      durationMilliseconds: currentTimeline.trackDurationMilliseconds,
    });
  };
  // Waits for the song's next whole second, so the clock flips on time and never skips a second.
  const scheduleRedrawAtNextWholeSecond = (): void => {
    if (!isPositionSeenNow || !isTrackPlaying || !currentTimeline) {
      return;
    }
    const millisecondsIntoCurrentSecond = extrapolatePositionMilliseconds(currentTimeline) % MILLISECONDS_PER_SECOND;
    redrawTimer = window.setTimeout(() => {
      drawCurrentPosition();
      scheduleRedrawAtNextWholeSecond();
    }, MILLISECONDS_PER_SECOND - millisecondsIntoCurrentSecond + PLAYBACK_POSITION_REDRAW_AFTER_WHOLE_SECOND_MILLISECONDS);
  };
  const restartRedrawing = (): void => {
    window.clearTimeout(redrawTimer);
    redrawTimer = undefined;
    drawCurrentPosition();
    scheduleRedrawAtNextWholeSecond();
  };

  return {
    showTimeline(timeline, isPlaying) {
      currentTimeline = timeline;
      isTrackPlaying = isPlaying;
      restartRedrawing();
    },
    redrawNow: drawCurrentPosition,
    setPositionSeen(isPositionSeen) {
      if (isPositionSeen !== isPositionSeenNow) {
        isPositionSeenNow = isPositionSeen;
        restartRedrawing();
      }
    },
  };
}

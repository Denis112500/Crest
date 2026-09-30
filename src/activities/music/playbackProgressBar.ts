import "../../styles/playbackProgressBar.css";

import { PLAYBACK_PROGRESS_REDRAWS_PER_SECOND } from "../../frontendConstants";
import type { NowPlayingTimeline } from "./nowPlayingTypes";

const MILLISECONDS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
const WITHOUT_TIMELINE_CLASS = "is-without-timeline";

export interface PlaybackProgressBar {
  progressBarElement: HTMLElement;
  showTimeline(timeline: NowPlayingTimeline | null, isPlaying: boolean): void;
  /** Animate only while someone can see it: the expanded view is open. */
  setAnimationActive(isAnimationActive: boolean): void;
}

// Players report the position only when something happens (play, pause, seek), so the
// bar moves by computing: last reported position + time passed since then. That works
// only while playing; a paused position doesn't move, however old its timestamp is.
export function createPlaybackProgressBar(): PlaybackProgressBar {
  const progressBarElement = document.createElement("div");
  progressBarElement.classList.add("playback-progress", WITHOUT_TIMELINE_CLASS);
  const elapsedLabelElement = document.createElement("span");
  elapsedLabelElement.className = "playback-progress-label";
  const progressTrackElement = document.createElement("div");
  progressTrackElement.className = "playback-progress-track";
  const progressFillElement = document.createElement("div");
  progressFillElement.className = "playback-progress-fill";
  progressTrackElement.append(progressFillElement);
  const remainingLabelElement = document.createElement("span");
  remainingLabelElement.className = "playback-progress-label";
  progressBarElement.append(elapsedLabelElement, progressTrackElement, remainingLabelElement);

  let currentTimeline: NowPlayingTimeline | null = null;
  let isTrackPlaying = false;
  let isProgressAnimationActive = false;
  // A slow timer, not requestAnimationFrame: rAF runs at the monitor's refresh rate
  // (239 Hz here), far more often than a bar moving 2 px per second needs.
  let progressRedrawTimer: number | undefined;

  const calculateCurrentPositionMilliseconds = (timeline: NowPlayingTimeline): number => {
    const millisecondsSinceReport = isTrackPlaying ? Date.now() - timeline.positionReportedAtUnixMilliseconds : 0;
    const extrapolatedPosition = timeline.reportedPositionMilliseconds + millisecondsSinceReport;
    return Math.min(Math.max(extrapolatedPosition, 0), timeline.trackDurationMilliseconds);
  };
  const drawProgress = (): void => {
    if (!currentTimeline) {
      return;
    }
    const currentPosition = calculateCurrentPositionMilliseconds(currentTimeline);
    const progressFraction = currentPosition / currentTimeline.trackDurationMilliseconds;
    // A transform animates on the GPU without re-laying out the page.
    progressFillElement.style.transform = `scaleX(${progressFraction})`;
    const elapsedText = formatPlaybackClock(currentPosition);
    const remainingText = `-${formatPlaybackClock(currentTimeline.trackDurationMilliseconds - currentPosition)}`;
    if (elapsedLabelElement.textContent !== elapsedText) elapsedLabelElement.textContent = elapsedText;
    if (remainingLabelElement.textContent !== remainingText) remainingLabelElement.textContent = remainingText;
  };
  const restartProgressDrawing = (): void => {
    window.clearInterval(progressRedrawTimer);
    progressRedrawTimer = undefined;
    drawProgress();
    if (isProgressAnimationActive && isTrackPlaying && currentTimeline) {
      progressRedrawTimer = window.setInterval(drawProgress, MILLISECONDS_PER_SECOND / PLAYBACK_PROGRESS_REDRAWS_PER_SECOND);
    }
  };

  return {
    progressBarElement,
    showTimeline(timeline, isPlaying) {
      currentTimeline = timeline;
      isTrackPlaying = isPlaying;
      progressBarElement.classList.toggle(WITHOUT_TIMELINE_CLASS, timeline === null);
      restartProgressDrawing();
    },
    setAnimationActive(isAnimationActive) {
      isProgressAnimationActive = isAnimationActive;
      restartProgressDrawing();
    },
  };
}

function formatPlaybackClock(totalMilliseconds: number): string {
  const totalSeconds = Math.floor(totalMilliseconds / MILLISECONDS_PER_SECOND);
  const minutes = Math.floor(totalSeconds / SECONDS_PER_MINUTE);
  const seconds = totalSeconds % SECONDS_PER_MINUTE;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

import "../../styles/expandedMusicView.css";

import { COVER_GAUGE_EXPANDED_GEOMETRY, PLAYBACK_POSITION_OPEN_REDRAWS_PER_SECOND } from "../../frontendConstants";
import { createCoverProgressGauge } from "./coverProgressGauge";
import { createMusicControlButtons } from "./musicControlButtons";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { createPlaybackPositionTicker, type PlaybackPosition } from "./playbackPositionTicker";
import type { AlbumArtRingColors } from "./sampleAlbumArtRingColors";
import { createSystemVolumeControl } from "./systemVolumeControl";

const MILLISECONDS_PER_SECOND = 1000;
const SECONDS_PER_MINUTE = 60;
const WITHOUT_TIMELINE_CLASS = "is-without-timeline";

export interface ExpandedMusicView {
  expandedViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
  showRingColors(albumArtRingColors: AlbumArtRingColors | null): void;
  setExpandedViewVisible(isExpandedViewVisible: boolean): void;
}

// The open pill (Gauges): the cover in its progress ring on the left; title and artist; below
// them the playback buttons, the speaker and the time ("1:43 / -1:51"). Ring and time move 4
// times a second while the pill is open and the song plays. The speaker's slider takes the
// whole row while it's open.
export function createExpandedMusicView(): ExpandedMusicView {
  const expandedViewElement = document.createElement("div");
  expandedViewElement.className = "expanded-music-view";

  const coverGauge = createCoverProgressGauge(COVER_GAUGE_EXPANDED_GEOMETRY);
  const trackTextElement = document.createElement("div");
  trackTextElement.className = "expanded-music-text";
  const expandedTitleElement = document.createElement("div");
  expandedTitleElement.className = "expanded-music-title";
  const expandedArtistElement = document.createElement("div");
  expandedArtistElement.className = "expanded-music-artist";
  trackTextElement.append(expandedTitleElement, expandedArtistElement);
  const musicControlButtons = createMusicControlButtons();
  const playbackTimeElement = document.createElement("span");
  playbackTimeElement.className = "expanded-music-time";
  const controlRowElement = document.createElement("div");
  controlRowElement.className = "expanded-music-control-row";
  // While the volume slider covers the time, the time isn't rewritten (once a second, laying the
  // row out again): nothing unseen should change. It's drawn once when the slider closes.
  let isTimeCoveredBySlider = false;
  const systemVolumeControl = createSystemVolumeControl(
    controlRowElement,
    [musicControlButtons.controlButtonsElement, playbackTimeElement],
    (isSliderOpen) => {
      isTimeCoveredBySlider = isSliderOpen;
      if (!isSliderOpen) {
        positionTicker.redrawNow();
      }
    },
  );
  controlRowElement.append(
    musicControlButtons.controlButtonsElement,
    systemVolumeControl.speakerButtonElement,
    systemVolumeControl.sliderElement,
    playbackTimeElement,
  );
  expandedViewElement.append(coverGauge.gaugeElement, trackTextElement, controlRowElement);

  const drawPlaybackPosition = (playbackPosition: PlaybackPosition): void => {
    coverGauge.drawPlaybackPosition(playbackPosition);
    // Live streams and players without a duration: the time keeps its space, without content.
    playbackTimeElement.classList.toggle(WITHOUT_TIMELINE_CLASS, playbackPosition === null);
    if (playbackPosition && !isTimeCoveredBySlider) {
      const { positionMilliseconds, durationMilliseconds } = playbackPosition;
      const playbackTimeText = `${formatPlaybackClock(positionMilliseconds)} / -${formatPlaybackClock(durationMilliseconds - positionMilliseconds)}`;
      if (playbackTimeElement.textContent !== playbackTimeText) {
        playbackTimeElement.textContent = playbackTimeText;
      }
    }
  };
  const positionTicker = createPlaybackPositionTicker(drawPlaybackPosition);

  return {
    expandedViewElement,
    showNowPlaying(nowPlaying) {
      const isPlaying = nowPlaying.playbackState === "playing";
      coverGauge.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      coverGauge.showIsPlaying(isPlaying);
      expandedTitleElement.textContent = nowPlaying.trackTitle;
      expandedArtistElement.textContent = nowPlaying.trackArtist;
      positionTicker.showTimeline(nowPlaying.timeline, isPlaying);
      musicControlButtons.showIsPlaying(isPlaying);
      musicControlButtons.showAvailableControls(nowPlaying.availableControls);
    },
    showRingColors(albumArtRingColors) {
      coverGauge.showRingColors(albumArtRingColors);
    },
    setExpandedViewVisible(isExpandedViewVisible) {
      positionTicker.setRedrawsPerSecond(isExpandedViewVisible ? PLAYBACK_POSITION_OPEN_REDRAWS_PER_SECOND : 0);
      systemVolumeControl.setSeen(isExpandedViewVisible);
    },
  };
}

function formatPlaybackClock(totalMilliseconds: number): string {
  const totalSeconds = Math.floor(totalMilliseconds / MILLISECONDS_PER_SECOND);
  const minutes = Math.floor(totalSeconds / SECONDS_PER_MINUTE);
  const seconds = totalSeconds % SECONDS_PER_MINUTE;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

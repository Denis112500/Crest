import "../../styles/compactMusicView.css";

import { COVER_GAUGE_COMPACT_GEOMETRY } from "../../frontendConstants";
import { createCoverProgressGauge } from "./coverProgressGauge";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { createPlaybackPositionTicker } from "./playbackPositionTicker";
import type { AlbumArtRingColors } from "./sampleAlbumArtRingColors";

export interface CompactMusicView {
  compactViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
  showRingColors(albumArtRingColors: AlbumArtRingColors | null): void;
  setPillOnScreen(isPillOnScreen: boolean): void;
}

// The small pill: the cover in its progress ring, and the title. The ring moves once a second
// while the song plays and the pill is on screen; a hidden window doesn't stop page timers in
// WebView2, so it stops itself while the pill is off screen (during a game, for example).
export function createCompactMusicView(): CompactMusicView {
  const compactViewElement = document.createElement("div");
  compactViewElement.className = "compact-music-view";
  const coverGauge = createCoverProgressGauge(COVER_GAUGE_COMPACT_GEOMETRY);
  const compactTitleElement = document.createElement("span");
  compactTitleElement.className = "compact-music-title";
  compactViewElement.append(coverGauge.gaugeElement, compactTitleElement);
  const positionTicker = createPlaybackPositionTicker((playbackPosition) =>
    coverGauge.drawPlaybackPosition(playbackPosition),
  );

  return {
    compactViewElement,
    showNowPlaying(nowPlaying) {
      const isPlaying = nowPlaying.playbackState === "playing";
      coverGauge.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      coverGauge.showIsPlaying(isPlaying);
      // textContent, never innerHTML: a track title must not be able to inject markup.
      compactTitleElement.textContent = nowPlaying.trackTitle;
      positionTicker.showTimeline(nowPlaying.timeline, isPlaying);
    },
    showRingColors(albumArtRingColors) {
      coverGauge.showRingColors(albumArtRingColors);
    },
    setPillOnScreen(isPillOnScreen) {
      positionTicker.setPositionSeen(isPillOnScreen);
    },
  };
}

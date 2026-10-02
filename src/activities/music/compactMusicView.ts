import "../../styles/compactMusicView.css";

import { createAlbumArtImage } from "./albumArtImage";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { createPlaybackBarsIndicator } from "./playbackBarsIndicator";

export interface CompactMusicView {
  compactViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
  setPillOnScreen(isPillOnScreen: boolean): void;
}

// The small pill: tiny album art, the title, and bars that bounce while playing.
export function createCompactMusicView(): CompactMusicView {
  const compactViewElement = document.createElement("div");
  compactViewElement.className = "compact-music-view";
  const compactAlbumArt = createAlbumArtImage("album-art-compact");
  const compactTitleElement = document.createElement("span");
  compactTitleElement.className = "compact-music-title";
  const playbackBars = createPlaybackBarsIndicator();
  compactViewElement.append(compactAlbumArt.albumArtElement, compactTitleElement, playbackBars.barsIndicatorElement);

  return {
    compactViewElement,
    showNowPlaying(nowPlaying) {
      compactAlbumArt.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      // textContent, never innerHTML: a track title must not be able to inject markup.
      compactTitleElement.textContent = nowPlaying.trackTitle;
      playbackBars.showIsPlaying(nowPlaying.playbackState === "playing");
    },
    setPillOnScreen(isPillOnScreen) {
      playbackBars.setPillOnScreen(isPillOnScreen);
    },
  };
}

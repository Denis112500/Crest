import "../../styles/companionSegmentViews.css";

import { createAlbumArtImage } from "./albumArtImage";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { createPlaybackBarsIndicator } from "./playbackBarsIndicator";

export interface CompanionMusicView {
  companionViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
  setCompanionOnScreen(isCompanionOnScreen: boolean): void;
}

// Music next to another main activity: tiny album art and the bars, no title (no room).
export function createCompanionMusicView(): CompanionMusicView {
  const companionViewElement = document.createElement("div");
  companionViewElement.className = "companion-segment-view";
  const companionAlbumArt = createAlbumArtImage("album-art-companion");
  const playbackBars = createPlaybackBarsIndicator();
  companionViewElement.append(companionAlbumArt.albumArtElement, playbackBars.barsIndicatorElement);

  return {
    companionViewElement,
    showNowPlaying(nowPlaying) {
      companionAlbumArt.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      playbackBars.showIsPlaying(nowPlaying.playbackState === "playing");
    },
    setCompanionOnScreen(isCompanionOnScreen) {
      playbackBars.setPillOnScreen(isCompanionOnScreen);
    },
  };
}

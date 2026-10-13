import "../../styles/companionMarkViews.css";

import { createAlbumArtImage } from "./albumArtImage";
import type { NowPlayingPayload } from "./nowPlayingTypes";

const COMPANION_WORD_BY_PLAYBACK_STATE: Record<NowPlayingPayload["playbackState"], string> = {
  playing: "Playing",
  paused: "Paused",
  changing: "Changing track",
  stopped: "Stopped",
};

export interface CompanionMusicView {
  companionViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
}

// Music next to another main activity: only the tiny album art. No bars: the companion is
// still (no endless animation in the closed pill's end, milestone F); the track is in its
// tooltip and accessible name.
export function createCompanionMusicView(): CompanionMusicView {
  const companionViewElement = document.createElement("div");
  companionViewElement.className = "companion-mark-view";
  companionViewElement.setAttribute("role", "img");
  const companionAlbumArt = createAlbumArtImage("album-art-companion");
  companionViewElement.append(companionAlbumArt.albumArtElement);

  return {
    companionViewElement,
    showNowPlaying(nowPlaying) {
      companionAlbumArt.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      const trackDescription = [nowPlaying.trackTitle, nowPlaying.trackArtist].filter(Boolean).join(" – ");
      const playbackStateWord = COMPANION_WORD_BY_PLAYBACK_STATE[nowPlaying.playbackState];
      const musicCompanionDescription = trackDescription ? `${playbackStateWord}: ${trackDescription}` : playbackStateWord;
      // Attributes, never markup: a track title must not be able to inject anything.
      companionViewElement.title = musicCompanionDescription;
      companionViewElement.setAttribute("aria-label", musicCompanionDescription);
    },
  };
}

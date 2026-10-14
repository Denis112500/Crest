import "../../styles/companionMarkViews.css";

import { COVER_GAUGE_COMPANION_GEOMETRY, PLAYBACK_POSITION_COMPACT_REDRAWS_PER_SECOND } from "../../frontendConstants";
import { createCoverProgressGauge } from "./coverProgressGauge";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { createPlaybackPositionTicker } from "./playbackPositionTicker";
import type { AlbumArtRingColors } from "./sampleAlbumArtRingColors";

const COMPANION_WORD_BY_PLAYBACK_STATE: Record<NowPlayingPayload["playbackState"], string> = {
  playing: "Playing",
  paused: "Paused",
  changing: "Changing track",
  stopped: "Stopped",
};

export interface CompanionMusicView {
  companionViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
  showRingColors(albumArtRingColors: AlbumArtRingColors | null): void;
  setCompanionOnScreen(isCompanionOnScreen: boolean): void;
}

// Music next to another main activity: only the cover in its small progress ring, which moves
// once a second while seen. The track is in its tooltip and accessible name.
export function createCompanionMusicView(): CompanionMusicView {
  const companionViewElement = document.createElement("div");
  companionViewElement.className = "companion-mark-view";
  companionViewElement.setAttribute("role", "img");
  const coverGauge = createCoverProgressGauge(COVER_GAUGE_COMPANION_GEOMETRY);
  companionViewElement.append(coverGauge.gaugeElement);
  const positionTicker = createPlaybackPositionTicker((playbackPosition) =>
    coverGauge.drawPlaybackPosition(playbackPosition),
  );

  return {
    companionViewElement,
    showNowPlaying(nowPlaying) {
      const isPlaying = nowPlaying.playbackState === "playing";
      coverGauge.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      coverGauge.showIsPlaying(isPlaying);
      positionTicker.showTimeline(nowPlaying.timeline, isPlaying);
      const trackDescription = [nowPlaying.trackTitle, nowPlaying.trackArtist].filter(Boolean).join(" – ");
      const playbackStateWord = COMPANION_WORD_BY_PLAYBACK_STATE[nowPlaying.playbackState];
      const musicCompanionDescription = trackDescription ? `${playbackStateWord}: ${trackDescription}` : playbackStateWord;
      // Attributes, never markup: a track title must not be able to inject anything.
      companionViewElement.title = musicCompanionDescription;
      companionViewElement.setAttribute("aria-label", musicCompanionDescription);
    },
    showRingColors(albumArtRingColors) {
      coverGauge.showRingColors(albumArtRingColors);
    },
    setCompanionOnScreen(isCompanionOnScreen) {
      positionTicker.setRedrawsPerSecond(isCompanionOnScreen ? PLAYBACK_POSITION_COMPACT_REDRAWS_PER_SECOND : 0);
    },
  };
}

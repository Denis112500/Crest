import "../../styles/expandedMusicView.css";

import { createAlbumArtImage } from "./albumArtImage";
import { createMusicControlButtons } from "./musicControlButtons";
import type { NowPlayingPayload } from "./nowPlayingTypes";
import { createPlaybackProgressBar } from "./playbackProgressBar";

export interface ExpandedMusicView {
  expandedViewElement: HTMLElement;
  showNowPlaying(nowPlaying: NowPlayingPayload): void;
  setProgressAnimationActive(isProgressAnimationActive: boolean): void;
}

// The open pill: large album art, title and artist, progress, and the playback buttons.
export function createExpandedMusicView(): ExpandedMusicView {
  const expandedViewElement = document.createElement("div");
  expandedViewElement.className = "expanded-music-view";

  const trackHeaderElement = document.createElement("div");
  trackHeaderElement.className = "expanded-music-header";
  const expandedAlbumArt = createAlbumArtImage("album-art-expanded");
  const trackTextElement = document.createElement("div");
  trackTextElement.className = "expanded-music-text";
  const expandedTitleElement = document.createElement("div");
  expandedTitleElement.className = "expanded-music-title";
  const expandedArtistElement = document.createElement("div");
  expandedArtistElement.className = "expanded-music-artist";
  trackTextElement.append(expandedTitleElement, expandedArtistElement);
  trackHeaderElement.append(expandedAlbumArt.albumArtElement, trackTextElement);

  const playbackProgressBar = createPlaybackProgressBar();
  const musicControlButtons = createMusicControlButtons();
  expandedViewElement.append(
    trackHeaderElement,
    playbackProgressBar.progressBarElement,
    musicControlButtons.controlButtonsElement,
  );

  return {
    expandedViewElement,
    showNowPlaying(nowPlaying) {
      const isPlaying = nowPlaying.playbackState === "playing";
      expandedAlbumArt.showAlbumArt(nowPlaying.albumArtDataUrl, nowPlaying.isAlbumArtLoading);
      expandedTitleElement.textContent = nowPlaying.trackTitle;
      expandedArtistElement.textContent = nowPlaying.trackArtist;
      playbackProgressBar.showTimeline(nowPlaying.timeline, isPlaying);
      musicControlButtons.showIsPlaying(isPlaying);
    },
    setProgressAnimationActive(isProgressAnimationActive) {
      playbackProgressBar.setAnimationActive(isProgressAnimationActive);
    },
  };
}

import "../../styles/musicViews.css";

import type { NowPlayingPayload } from "./nowPlayingTypes";

const TITLE_ARTIST_SEPARATOR = " · ";
const PAUSED_SUFFIX = " (paused)";

// Milestone (d): plain text. Milestone (e) adds album art and the playing bars.
export function renderCompactMusicView(pillShellElement: HTMLElement, activityPayload: unknown): void {
  const nowPlaying = activityPayload as NowPlayingPayload;
  // Some players leave the artist empty; skip empty parts instead of showing a dangling "·".
  const titleAndArtist = [nowPlaying.trackTitle, nowPlaying.trackArtist]
    .filter((textPart) => textPart.trim() !== "")
    .join(TITLE_ARTIST_SEPARATOR);
  const compactMusicTextElement = document.createElement("span");
  compactMusicTextElement.className = "compact-music-text";
  // textContent, never innerHTML: a track title must not be able to inject markup.
  compactMusicTextElement.textContent =
    nowPlaying.playbackState === "paused" ? titleAndArtist + PAUSED_SUFFIX : titleAndArtist;
  pillShellElement.replaceChildren(compactMusicTextElement);
}

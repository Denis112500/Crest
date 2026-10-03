// Mirrors `MediaSessionSnapshot` in src-tauri/src/media/media_session_snapshot.rs:
// the payload of the "music" activity.
export interface NowPlayingPayload {
  sourceAppIdentifier: string;
  trackTitle: string;
  trackArtist: string;
  albumTitle: string;
  albumArtDataUrl: string | null;
  isAlbumArtLoading: boolean;
  playbackState: "playing" | "paused" | "changing" | "stopped";
  timeline: NowPlayingTimeline | null;
  availableControls: NowPlayingControlAvailability;
}

// Which buttons the player accepts right now; the others are greyed out.
export interface NowPlayingControlAvailability {
  canTogglePlayPause: boolean;
  canSkipToNextTrack: boolean;
  canSkipToPreviousTrack: boolean;
}

export interface NowPlayingTimeline {
  trackDurationMilliseconds: number;
  reportedPositionMilliseconds: number;
  positionReportedAtUnixMilliseconds: number;
}

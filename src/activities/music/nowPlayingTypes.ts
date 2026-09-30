// Mirrors `MediaSessionSnapshot` in src-tauri/src/media/media_session_snapshot.rs:
// the payload of the "music" activity.
export interface NowPlayingPayload {
  sourceAppIdentifier: string;
  trackTitle: string;
  trackArtist: string;
  albumTitle: string;
  albumArtDataUrl: string | null;
  playbackState: "playing" | "paused" | "changing" | "stopped";
  timeline: NowPlayingTimeline | null;
}

export interface NowPlayingTimeline {
  trackDurationMilliseconds: number;
  reportedPositionMilliseconds: number;
  positionReportedAtUnixMilliseconds: number;
}

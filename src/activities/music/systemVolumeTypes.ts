// Mirrors `SystemVolumeState` in src-tauri/src/system_volume/system_volume_reading.rs: the
// default output device's volume. Rust sends `null` instead when there is no output device.
export interface SystemVolume {
  /** 0..1 in whole percent, the position of Windows' own slider. */
  level: number;
  isMuted: boolean;
}

// The music tile's speaker slides the slider out on a click (layout C); the idle tile (P.3b) shows
// it all the time, and its speaker is only an icon.
export type SystemVolumeSliderMode = "slidesOutOnClick" | "alwaysOpen";

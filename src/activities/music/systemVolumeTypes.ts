// Mirrors `SystemVolumeState` in src-tauri/src/system_volume/system_volume_reading.rs: the
// default output device's volume. Rust sends `null` instead when there is no output device.
export interface SystemVolume {
  /** 0..1 in whole percent, the position of Windows' own slider. */
  level: number;
  isMuted: boolean;
}

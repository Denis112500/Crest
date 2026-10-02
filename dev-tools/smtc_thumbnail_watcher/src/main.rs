// Dev tool, not part of Crest: prints every change of the current media session's title or
// thumbnail (byte size, content type, PNG pixel size, hash) and saves each distinct thumbnail.
// Written to find out what Brave sends on a track change (its logo first, the cover ~100 ms
// later). PowerShell 5.1 can't read the thumbnail bytes, so this is a tiny Rust program.
//
//   cd dev-tools/smtc_thumbnail_watcher
//   cargo run --release -- <watch seconds> <folder for thumbnails>
use std::time::{Duration, Instant};

use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as SmtcSessionManager;
use windows::Storage::Streams::DataReader;
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

const DEFAULT_WATCH_SECONDS: u64 = 300;
const SAMPLING_PAUSE: Duration = Duration::from_millis(60);
const PNG_SIGNATURE: &[u8] = b"\x89PNG";
/// A PNG stores its width and height as big-endian numbers right after the signature and the
/// IHDR chunk header.
const PNG_WIDTH_BYTE_RANGE: std::ops::Range<usize> = 16..20;
const PNG_HEIGHT_BYTE_RANGE: std::ops::Range<usize> = 20..24;

/// FNV-1a: a tiny, good-enough hash to tell thumbnails apart without extra crates.
fn hash_thumbnail_bytes(thumbnail_bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    thumbnail_bytes
        .iter()
        .fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME))
}

fn describe_png_size(thumbnail_bytes: &[u8]) -> String {
    let read_big_endian_number = |byte_range: std::ops::Range<usize>| {
        thumbnail_bytes.get(byte_range).and_then(|bytes| bytes.try_into().ok()).map(u32::from_be_bytes)
    };
    match (
        thumbnail_bytes.starts_with(PNG_SIGNATURE),
        read_big_endian_number(PNG_WIDTH_BYTE_RANGE),
        read_big_endian_number(PNG_HEIGHT_BYTE_RANGE),
    ) {
        (true, Some(pixel_width), Some(pixel_height)) => format!("{pixel_width}x{pixel_height}"),
        _ => "not a PNG".to_string(),
    }
}

fn describe_current_session(
    session_manager: &SmtcSessionManager,
    thumbnail_folder: &str,
) -> windows::core::Result<String> {
    let Ok(current_session) = session_manager.GetCurrentSession() else {
        return Ok("<no session>".to_string());
    };
    let media_properties = current_session.TryGetMediaPropertiesAsync()?.join()?;
    let track_title = media_properties.Title()?.to_string_lossy();
    let playback_status = current_session.GetPlaybackInfo()?.PlaybackStatus()?.0;
    let thumbnail_description = match media_properties.Thumbnail() {
        Ok(thumbnail_reference) => {
            let thumbnail_stream = thumbnail_reference.OpenReadAsync()?.join()?;
            let thumbnail_byte_count = u32::try_from(thumbnail_stream.Size()?).unwrap_or(u32::MAX);
            let thumbnail_reader = DataReader::CreateDataReader(&thumbnail_stream)?;
            let loaded_byte_count = thumbnail_reader.LoadAsync(thumbnail_byte_count)?.join()?;
            let mut thumbnail_bytes = vec![0u8; loaded_byte_count as usize];
            thumbnail_reader.ReadBytes(&mut thumbnail_bytes)?;
            let thumbnail_hash = hash_thumbnail_bytes(&thumbnail_bytes);
            let thumbnail_path = format!("{thumbnail_folder}/{thumbnail_hash:016x}.img");
            if !std::path::Path::new(&thumbnail_path).exists() {
                if let Err(write_error) = std::fs::write(&thumbnail_path, &thumbnail_bytes) {
                    eprintln!("could not save {thumbnail_path}: {write_error}");
                }
            }
            format!(
                "thumbnail={thumbnail_hash:016x} {}B type='{}' png={}",
                thumbnail_bytes.len(),
                thumbnail_stream.ContentType()?,
                describe_png_size(&thumbnail_bytes)
            )
        }
        Err(_) => "thumbnail=none".to_string(),
    };
    Ok(format!("'{track_title}' status={playback_status} {thumbnail_description}"))
}

fn main() -> windows::core::Result<()> {
    let watch_seconds = std::env::args().nth(1).and_then(|text| text.parse().ok()).unwrap_or(DEFAULT_WATCH_SECONDS);
    let thumbnail_folder = std::env::args().nth(2).unwrap_or_else(|| ".".to_string());
    // SAFETY: first WinRT call on this thread; the multithreaded apartment needs no message loop.
    unsafe { RoInitialize(RO_INIT_MULTITHREADED)? };
    let session_manager = SmtcSessionManager::RequestAsync()?.join()?;
    println!("watching for {watch_seconds} s, saving thumbnails into {thumbnail_folder}");
    let watch_started_at = Instant::now();
    let watch_deadline = watch_started_at + Duration::from_secs(watch_seconds);
    let mut previous_line = String::new();
    while Instant::now() < watch_deadline {
        let current_line = describe_current_session(&session_manager, &thumbnail_folder)
            .unwrap_or_else(|read_error| format!("read error: {read_error}"));
        if current_line != previous_line {
            println!("{:>8.3}s  {current_line}", watch_started_at.elapsed().as_secs_f64());
            previous_line = current_line;
        }
        std::thread::sleep(SAMPLING_PAUSE);
    }
    println!("watch finished");
    Ok(())
}

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};

/// Title, artist and album: identifies a track well enough to reuse its album art.
pub type AlbumArtTrackKey = (String, String, String);

/// Reading and encoding album art costs a few milliseconds, and position or play/pause
/// updates happen far more often than track changes, so the last result is kept.
#[derive(Default)]
pub struct SmtcAlbumArtCache {
    cached_track_key: Option<AlbumArtTrackKey>,
    cached_album_art_data_url: Option<String>,
}

impl SmtcAlbumArtCache {
    /// A missing result is read again next time: browsers often publish the title first
    /// and the artwork a moment later.
    pub fn album_art_for_track(
        &mut self,
        track_key: AlbumArtTrackKey,
        thumbnail_reference: Option<IRandomAccessStreamReference>,
    ) -> Option<String> {
        let is_cached_art_usable =
            self.cached_track_key.as_ref() == Some(&track_key) && self.cached_album_art_data_url.is_some();
        if !is_cached_art_usable {
            self.cached_album_art_data_url =
                thumbnail_reference.and_then(|reference| read_thumbnail_as_data_url(&reference));
            self.cached_track_key = Some(track_key);
        }
        self.cached_album_art_data_url.clone()
    }
}

fn read_thumbnail_as_data_url(thumbnail_reference: &IRandomAccessStreamReference) -> Option<String> {
    let thumbnail_bytes_and_type = || -> windows::core::Result<(Vec<u8>, String)> {
        let thumbnail_stream = thumbnail_reference.OpenReadAsync()?.join()?;
        let thumbnail_byte_count = u32::try_from(thumbnail_stream.Size()?).unwrap_or(u32::MAX);
        let thumbnail_reader = DataReader::CreateDataReader(&thumbnail_stream)?;
        let loaded_byte_count = thumbnail_reader.LoadAsync(thumbnail_byte_count)?.join()?;
        let mut thumbnail_bytes = vec![0u8; loaded_byte_count as usize];
        thumbnail_reader.ReadBytes(&mut thumbnail_bytes)?;
        Ok((thumbnail_bytes, thumbnail_stream.ContentType()?.to_string_lossy()))
    };
    let (thumbnail_bytes, reported_content_type) = thumbnail_bytes_and_type().ok()?;
    if thumbnail_bytes.is_empty() {
        return None;
    }
    let image_mime_type = if reported_content_type.starts_with("image/") {
        reported_content_type
    } else {
        detect_image_mime_type(&thumbnail_bytes)?.to_string()
    };
    Some(format!("data:{image_mime_type};base64,{}", BASE64_STANDARD.encode(&thumbnail_bytes)))
}

/// Some players leave the content type empty, so the format is read from the file's
/// first bytes ("magic numbers"). Unknown formats are treated as missing art.
fn detect_image_mime_type(image_bytes: &[u8]) -> Option<&'static str> {
    const PNG_SIGNATURE: &[u8] = b"\x89PNG";
    const JPEG_SIGNATURE: &[u8] = b"\xFF\xD8\xFF";
    const GIF_SIGNATURE: &[u8] = b"GIF8";
    const RIFF_CONTAINER_SIGNATURE: &[u8] = b"RIFF";
    const WEBP_FORMAT_TAG: &[u8] = b"WEBP";
    const WEBP_FORMAT_TAG_OFFSET: usize = 8;

    if image_bytes.starts_with(PNG_SIGNATURE) {
        Some("image/png")
    } else if image_bytes.starts_with(JPEG_SIGNATURE) {
        Some("image/jpeg")
    } else if image_bytes.starts_with(GIF_SIGNATURE) {
        Some("image/gif")
    } else if image_bytes.starts_with(RIFF_CONTAINER_SIGNATURE)
        && image_bytes.get(WEBP_FORMAT_TAG_OFFSET..WEBP_FORMAT_TAG_OFFSET + WEBP_FORMAT_TAG.len())
            == Some(WEBP_FORMAT_TAG)
    {
        Some("image/webp")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_common_image_formats_from_their_first_bytes() {
        assert_eq!(detect_image_mime_type(b"\x89PNG\r\n\x1a\n...."), Some("image/png"));
        assert_eq!(detect_image_mime_type(b"\xFF\xD8\xFF\xE0...."), Some("image/jpeg"));
        assert_eq!(detect_image_mime_type(b"RIFF\x00\x00\x00\x00WEBPVP8 "), Some("image/webp"));
        assert_eq!(detect_image_mime_type(b"not an image"), None);
    }
}

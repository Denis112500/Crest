//! The channel's message format, both directions: a 4-byte little-endian length, then that many
//! bytes. A pipe is a byte stream, so the reader needs to know where a message ends.

use std::io::{Read, Write};

use crate::backend_constants::CLAUDE_CODE_HOOK_EVENT_MAX_BYTES;

pub fn write_length_prefixed_message(message_writer: &mut impl Write, message_bytes: &[u8]) -> std::io::Result<()> {
    let message_length = u32::try_from(message_bytes.len()).map_err(std::io::Error::other)?;
    message_writer.write_all(&message_length.to_le_bytes())?;
    message_writer.write_all(message_bytes)
}

/// Refuses messages above `CLAUDE_CODE_HOOK_EVENT_MAX_BYTES` instead of allocating for them.
pub fn read_length_prefixed_message(message_reader: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut message_length_bytes = [0u8; 4];
    message_reader.read_exact(&mut message_length_bytes)?;
    let message_length = u32::from_le_bytes(message_length_bytes) as usize;
    if message_length > CLAUDE_CODE_HOOK_EVENT_MAX_BYTES {
        return Err(std::io::Error::other(format!("a message of {message_length} bytes was refused")));
    }
    let mut message_bytes = vec![0u8; message_length];
    message_reader.read_exact(&mut message_bytes)?;
    Ok(message_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_written_message_reads_back_unchanged_including_an_empty_one() {
        let mut channel_bytes = Vec::new();
        write_length_prefixed_message(&mut channel_bytes, br#"{"hook_event_name":"Stop"}"#).unwrap();
        write_length_prefixed_message(&mut channel_bytes, b"").unwrap();
        let mut channel_reader = channel_bytes.as_slice();
        assert_eq!(read_length_prefixed_message(&mut channel_reader).unwrap(), br#"{"hook_event_name":"Stop"}"#);
        assert!(read_length_prefixed_message(&mut channel_reader).unwrap().is_empty());
    }

    #[test]
    fn an_oversized_length_is_refused_before_reading() {
        let oversized_length = (CLAUDE_CODE_HOOK_EVENT_MAX_BYTES as u32 + 1).to_le_bytes();
        assert!(read_length_prefixed_message(&mut oversized_length.as_slice()).is_err());
    }
}

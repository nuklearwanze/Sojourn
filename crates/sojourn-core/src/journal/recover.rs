//! Journal scan & crash recovery support (FR-CORE-303, SC-004): validate the
//! chained frames, recover the longest intact prefix, and report precisely what
//! a torn tail lost.

use super::{Frame, FrameKind, HeaderBody, integrity};
use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Result of scanning a journal byte stream.
#[derive(Debug)]
pub struct ScanResult {
    /// The run identity (first frame).
    pub header: HeaderBody,
    /// All intact frames after the header, in order.
    pub frames: Vec<Frame>,
    /// Description of a torn/corrupt tail, if any (None = clean end).
    pub torn_tail: Option<TornTail>,
}

/// Precise loss report for a torn tail (US4 scenario 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TornTail {
    /// First sequence number lost.
    pub first_lost_seq: u64,
    /// Last intact tick.
    pub last_intact_tick: u64,
    /// Bytes discarded.
    pub bytes_discarded: u64,
}

/// Scan a journal byte stream, validating the checksum chain frame by frame.
///
/// `strict`: refuse *any* damage (ironman load verification — tampering or
/// corruption anywhere is a refusal, FR-CORE-305). Non-strict: return the longest
/// intact prefix plus a precise loss report (crash recovery).
pub fn scan(bytes: &[u8], strict: bool) -> Result<ScanResult, CoreError> {
    let mut frames: Vec<Frame> = Vec::new();
    let mut last_checksum = [0u8; 32];
    let mut pos: usize = 0;
    let mut next_seq: u64 = 0;
    let mut torn: Option<TornTail> = None;

    while pos < bytes.len() {
        match read_frame(bytes, pos, &last_checksum, next_seq) {
            Ok((frame, checksum, consumed)) => {
                last_checksum = checksum;
                next_seq += 1;
                pos += consumed;
                frames.push(frame);
            }
            Err(reason) => {
                if strict {
                    return Err(CoreError::IntegrityFailure {
                        what: format!("journal frame {next_seq}: {reason}"),
                    });
                }
                torn = Some(TornTail {
                    first_lost_seq: next_seq,
                    last_intact_tick: frames.last().map_or(0, |f| f.tick),
                    bytes_discarded: (bytes.len() - pos) as u64,
                });
                break;
            }
        }
    }

    let Some(first) = frames.first() else {
        return Err(CoreError::JournalCorrupt {
            valid_prefix_tick: 0,
            lost: "no intact frames (journal header unreadable)".into(),
        });
    };
    if first.kind != FrameKind::Header {
        return Err(CoreError::IntegrityFailure {
            what: "journal does not start with a HEADER frame".into(),
        });
    }
    let header: HeaderBody = postcard::from_bytes(&first.body).map_err(CoreError::ser)?;
    if header.format_version > super::JOURNAL_FORMAT_VERSION {
        return Err(CoreError::SaveFormatUnsupported {
            found: header.format_version,
            supported: super::JOURNAL_FORMAT_VERSION,
        });
    }
    frames.remove(0);
    Ok(ScanResult {
        header,
        frames,
        torn_tail: torn,
    })
}

/// Read one frame at `pos`; verify chain and sequence. Returns (frame, checksum,
/// bytes consumed) or a reason string for non-strict tail handling.
fn read_frame(
    bytes: &[u8],
    pos: usize,
    prev_checksum: &[u8; 32],
    expect_seq: u64,
) -> Result<(Frame, [u8; 32], usize), String> {
    let avail = bytes.len() - pos;
    if avail < 4 {
        return Err(format!("{avail} trailing bytes (incomplete length prefix)"));
    }
    let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().expect("4 bytes")) as usize;
    if len < 1 + 8 + 8 + 32 {
        return Err(format!("frame length {len} too small"));
    }
    if avail < 4 + len {
        return Err(format!(
            "truncated frame (need {} bytes, have {avail})",
            4 + len
        ));
    }
    let frame_bytes = &bytes[pos + 4..pos + 4 + len];
    let (head, rest) = frame_bytes.split_at(17);
    let (body, checksum) = rest.split_at(rest.len() - 32);
    let expected = integrity::chain(prev_checksum, head, body);
    if checksum != expected {
        return Err("checksum chain mismatch (corruption or tampering)".to_string());
    }
    let kind = match head[0] {
        0 => FrameKind::Header,
        1 => FrameKind::Command,
        2 => FrameKind::Outcome,
        3 => FrameKind::Event,
        4 => FrameKind::Checkpoint,
        5 => FrameKind::HashMark,
        k => return Err(format!("unknown frame kind {k}")),
    };
    let seq = u64::from_le_bytes(head[1..9].try_into().expect("8 bytes"));
    if seq != expect_seq {
        return Err(format!("sequence gap: expected {expect_seq}, found {seq}"));
    }
    let tick = u64::from_le_bytes(head[9..17].try_into().expect("8 bytes"));
    Ok((
        Frame {
            kind,
            seq,
            tick,
            body: body.to_vec(),
        },
        expected,
        4 + len,
    ))
}

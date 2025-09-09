//! Minimal HDLC framing/deframing for serial communication
// Uses the standard HDLC flag (0x7E) and escape (0x7D) bytes.
// This is a simple, no-CRC, no-address implementation for embedded use.

pub const HDLC_FLAG: u8 = 0x7E;
pub const HDLC_ESCAPE: u8 = 0x7D;
pub const HDLC_XOR: u8 = 0x20;

/// Frame a payload into an HDLC frame (adds flag, escapes as needed)
pub fn hdlc_frame(payload: &[u8], out: &mut heapless::Vec<u8, 128>) {
    out.clear();
    out.push(HDLC_FLAG).ok();
    for &b in payload {
        match b {
            HDLC_FLAG | HDLC_ESCAPE => {
                out.push(HDLC_ESCAPE).ok();
                out.push(b ^ HDLC_XOR).ok();
            }
            _ => {
                out.push(b).ok();
            }
        }
    }
    out.push(HDLC_FLAG).ok();
}

/// Deframe HDLC data (returns Some(payload) if a full frame is found)
pub fn hdlc_deframe(
    buf: &mut heapless::Vec<u8, 128>,
    out: &mut heapless::Vec<u8, 128>,
) -> Option<()> {
    let mut in_frame = false;
    let mut escape = false;
    out.clear();
    let mut i = 0;
    while i < buf.len() {
        let b = buf[i];
        if !in_frame {
            if b == HDLC_FLAG {
                in_frame = true;
                out.clear();
            }
        } else {
            if escape {
                out.push(b ^ HDLC_XOR).ok();
                escape = false;
            } else if b == HDLC_ESCAPE {
                escape = true;
            } else if b == HDLC_FLAG {
                if !out.is_empty() {
                    // Remove processed bytes from buf
                    buf.drain(..=i);
                    return Some(());
                }
                // else: empty frame, ignore
                in_frame = false;
            } else {
                out.push(b).ok();
            }
        }
        i += 1;
    }
    None
}

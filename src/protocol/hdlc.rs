//! Minimal HDLC framing/deframing for serial communication
// Uses the standard HDLC flag (0x7E) and escape (0x7D) bytes.
// This is a simple, no-CRC, no-address implementation for embedded use.

pub const HDLC_FLAG: u8 = 0x7E;
pub const HDLC_ESCAPE: u8 = 0x7D;
pub const HDLC_XOR: u8 = 0x20;

/// Frame a payload into an HDLC frame (adds flag, escapes as needed, appends 16-bit xsum)
pub fn hdlc_frame(payload: &[u8], out: &mut heapless::Vec<u8, 128>) {
    out.clear();
    out.push(HDLC_FLAG).ok();
    // Compute 16-bit checksum (sum of all bytes, little-endian)
    let xsum = payload.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16));
    // Write payload
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
    // Write checksum (little-endian, escaped)
    for &b in &xsum.to_le_bytes() {
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

/// Deframe HDLC data (returns Some(payload) if a full frame with valid 16-bit xsum is found)
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
                if out.len() >= 2 {
                    // Remove processed bytes from buf (shift remaining bytes)
                    if i + 1 < buf.len() {
                        let remaining = buf.len() - (i + 1);
                        for j in 0..remaining {
                            buf[j] = buf[i + 1 + j];
                        }
                        buf.truncate(remaining);
                    } else {
                        buf.clear();
                    }
                    // Split payload and checksum
                    let payload_len = out.len() - 2;
                    let (payload, xsum_bytes) = out.split_at(payload_len);
                    let xsum_recv = u16::from_le_bytes([xsum_bytes[0], xsum_bytes[1]]);
                    let xsum_calc = payload.iter().fold(0u16, |acc, &b| acc.wrapping_add(b as u16));
                    if xsum_recv == xsum_calc {
                        // Copy only payload to out
                        out.truncate(payload_len);
                        return Some(());
                    } else {
                        // Checksum mismatch, discard
                        out.clear();
                        return None;
                    }
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

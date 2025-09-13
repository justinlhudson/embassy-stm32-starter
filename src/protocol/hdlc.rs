//! Minimal HDLC framing/deframing for serial communication
// Uses the standard HDLC flag (0x7E) and escape (0x7D) bytes.
// Includes optional PPP/HDLC 16-bit FCS (CRC-16, poly 0x8408), compile-time toggle.

pub const HDLC_FLAG: u8 = 0x7E;
pub const HDLC_ESCAPE: u8 = 0x7D;
pub const HDLC_XOR: u8 = 0x20;

/// Compute PPP/HDLC 16-bit FCS.
/// Polynomial 0x8408 (reversed 0x1021), init 0xFFFF, reflected, final XOR 0xFFFF.
/// Returns the 16-bit FCS value to append (already complemented).
#[cfg(feature = "hdlc_fcs")]
fn fcs16_ppp(data: &[u8]) -> u16 {
  let mut fcs: u16 = 0xFFFF;
  for &b in data {
    let mut x = (fcs ^ (b as u16)) & 0x00FF;
    for _ in 0..8 {
      if (x & 0x0001) != 0 {
        x = (x >> 1) ^ 0x8408;
      } else {
        x >>= 1;
      }
    }
    fcs = (fcs >> 8) ^ x;
  }
  !fcs
}

/// Frame a payload into an HDLC frame (adds flag, escapes as needed, appends 16-bit FCS)
pub fn hdlc_frame<const M: usize>(payload: &[u8], out: &mut heapless::Vec<u8, M>) {
  out.clear();
  out.push(HDLC_FLAG).ok();

  defmt::debug!(
    "HDLC frame input: {} bytes, hex: {:02x}",
    payload.len(),
    &payload[..core::cmp::min(16, payload.len())]
  );

  // Compute FCS (PPP/HDLC) if enabled; otherwise 0
  #[cfg(feature = "hdlc_fcs")]
  let fcs = fcs16_ppp(payload);
  #[cfg(not(feature = "hdlc_fcs"))]
  let fcs: u16 = 0;
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
  // Write FCS (little-endian, escaped)
  for &b in &fcs.to_le_bytes() {
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

  defmt::debug!("HDLC frame output: {} bytes (including flags/FCS)", out.len());
}

/// Deframe HDLC data (returns Some(payload) if a full frame is found and FCS is valid when enabled)
pub fn hdlc_deframe<const N: usize, const M: usize>(
  buf: &mut heapless::Vec<u8, N>,
  out: &mut heapless::Vec<u8, M>,
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
          // Split payload and FCS
          let payload_len = out.len() - 2;
          let (payload, fcs_bytes) = out.split_at(payload_len);
          let fcs_recv = u16::from_le_bytes([fcs_bytes[0], fcs_bytes[1]]);

          defmt::debug!(
            "HDLC deframe: total_len={}, payload_len={}, fcs_bytes=[{:02x},{:02x}]",
            out.len(),
            payload_len,
            fcs_bytes[0],
            fcs_bytes[1]
          );

          #[cfg(feature = "hdlc_fcs")]
          {
            let fcs_calc = fcs16_ppp(payload);
            if fcs_recv == fcs_calc {
              out.truncate(payload_len);
              return Some(());
            } else {
              out.clear();
              defmt::error!(
                "HDLC FCS mismatch: recv={=u16}, calc={=u16}, len={}",
                fcs_recv,
                fcs_calc,
                payload_len
              );
              return None;
            }
          }
          #[cfg(not(feature = "hdlc_fcs"))]
          {
            let _ = payload; // suppress unused when FCS disabled
            let _ = fcs_recv; // suppress unused when FCS disabled
            // FCS disabled: accept frame without verification (strip trailing 2 bytes)
            out.truncate(payload_len);
            return Some(());
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

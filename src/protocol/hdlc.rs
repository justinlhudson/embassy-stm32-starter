//! Minimal HDLC framing for serial communication.
//!
//! Uses the standard HDLC flag (0x7E) and escape (0x7D) bytes.
//! Optional PPP/HDLC 16-bit FCS (CRC-16, poly 0x8408) is enabled with the
//! `hdlc_fcs` cargo feature; otherwise the FCS field is omitted.

pub const HDLC_FLAG: u8 = 0x7E;
pub const HDLC_ESCAPE: u8 = 0x7D;
pub const HDLC_XOR: u8 = 0x20;

/// PPP/HDLC 16-bit FCS: poly 0x8408, init 0xFFFF, reflected, final XOR 0xFFFF.
#[cfg(feature = "hdlc_fcs")]
pub fn fcs16_ppp(data: &[u8]) -> u16 {
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

#[inline]
fn push_escaped<const N: usize>(out: &mut heapless::Vec<u8, N>, b: u8) {
  match b {
    HDLC_FLAG | HDLC_ESCAPE => {
      let _ = out.push(HDLC_ESCAPE);
      let _ = out.push(b ^ HDLC_XOR);
    }
    _ => {
      let _ = out.push(b);
    }
  }
}

/// Frame a payload (adds opening flag, escapes payload [+ FCS], closing flag).
pub fn hdlc_frame<const M: usize>(payload: &[u8], out: &mut heapless::Vec<u8, M>) {
  out.clear();
  let _ = out.push(HDLC_FLAG);

  for &b in payload {
    push_escaped(out, b);
  }

  #[cfg(feature = "hdlc_fcs")]
  for &b in &fcs16_ppp(payload).to_le_bytes() {
    push_escaped(out, b);
  }

  let _ = out.push(HDLC_FLAG);
}

/// Deframe error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdlcError {
  /// No complete frame in the buffer yet.
  Incomplete,
  /// Frame received but FCS check failed.
  FcsMismatch { received: u16, calculated: u16, len: usize },
}

/// Try to extract one HDLC frame from `buf`. On success, `out` holds the
/// (un-escaped, FCS-stripped) payload and the consumed bytes are removed from
/// `buf`. On `Incomplete`, `buf` is left untouched up to the first flag (so
/// the caller can simply append more bytes and retry). On `FcsMismatch`, the
/// bad frame is consumed from `buf`.
pub fn hdlc_deframe<const N: usize, const M: usize>(buf: &mut heapless::Vec<u8, N>, out: &mut heapless::Vec<u8, M>) -> Result<(), HdlcError> {
  out.clear();

  // Find opening flag.
  let start = match buf.iter().position(|&b| b == HDLC_FLAG) {
    Some(p) => p,
    None => {
      buf.clear();
      return Err(HdlcError::Incomplete);
    }
  };

  // Find closing flag after the opening one. Skip consecutive flags so two
  // back-to-back 0x7E (idle / inter-frame) don't yield an empty frame loop.
  let mut i = start + 1;
  while i < buf.len() && buf[i] == HDLC_FLAG {
    i += 1;
  }
  let payload_start = i;
  let end = match buf[payload_start..].iter().position(|&b| b == HDLC_FLAG) {
    Some(p) => payload_start + p,
    None => {
      // Drop anything before the opening flag, keep the rest for next time.
      if start > 0 {
        buf.copy_within(start.., 0);
        buf.truncate(buf.len() - start);
      }
      return Err(HdlcError::Incomplete);
    }
  };

  // Un-escape the slice [payload_start..end] into `out`.
  let mut escape = false;
  for &b in &buf[payload_start..end] {
    if escape {
      let _ = out.push(b ^ HDLC_XOR);
      escape = false;
    } else if b == HDLC_ESCAPE {
      escape = true;
    } else {
      let _ = out.push(b);
    }
  }

  // Consume up to and including the closing flag.
  let consumed = end + 1;
  buf.copy_within(consumed.., 0);
  buf.truncate(buf.len() - consumed);

  // Validate FCS (when enabled).
  #[cfg(feature = "hdlc_fcs")]
  {
    if out.len() < 2 {
      return Err(HdlcError::Incomplete);
    }
    let payload_len = out.len() - 2;
    let fcs_recv = u16::from_le_bytes([out[payload_len], out[payload_len + 1]]);
    let fcs_calc = fcs16_ppp(&out[..payload_len]);
    if fcs_recv != fcs_calc {
      out.clear();
      return Err(HdlcError::FcsMismatch {
        received: fcs_recv,
        calculated: fcs_calc,
        len: payload_len,
      });
    }
    out.truncate(payload_len);
  }

  Ok(())
}

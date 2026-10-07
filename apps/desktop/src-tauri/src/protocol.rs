//! MovaPad Binary Protocol — Rust Implementation
//!
//! Mirror of the TypeScript protocol package. Both implementations
//! MUST produce identical wire format.

/// Current protocol version.
pub const PROTOCOL_VERSION: u8 = 1;

/// Message type identifiers (lower 4 bits of header byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Move = 0x01,
    LeftDown = 0x02,
    LeftUp = 0x03,
    RightDown = 0x04,
    RightUp = 0x05,
    Scroll = 0x06,
    Ping = 0x07,
    Pong = 0x08,
    SessionStart = 0x09,
    SessionEnd = 0x0A,
    DeviceInfo = 0x0B,
    Error = 0x0C,
    DragStart = 0x0D,
    DragEnd = 0x0E,
}

impl MessageType {
    /// Parse a message type from the lower 4 bits of a byte.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Move),
            0x02 => Some(Self::LeftDown),
            0x03 => Some(Self::LeftUp),
            0x04 => Some(Self::RightDown),
            0x05 => Some(Self::RightUp),
            0x06 => Some(Self::Scroll),
            0x07 => Some(Self::Ping),
            0x08 => Some(Self::Pong),
            0x09 => Some(Self::SessionStart),
            0x0A => Some(Self::SessionEnd),
            0x0B => Some(Self::DeviceInfo),
            0x0C => Some(Self::Error),
            0x0D => Some(Self::DragStart),
            0x0E => Some(Self::DragEnd),
            _ => None,
        }
    }
}

/// Protocol error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ErrorCode {
    Unknown = 0x0000,
    ProtocolMismatch = 0x0001,
    AuthFailed = 0x0002,
    SessionExpired = 0x0003,
    DeviceRevoked = 0x0004,
    RateLimited = 0x0005,
    InvalidMessage = 0x0006,
}

/// Discriminated union of all protocol messages.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolMessage {
    Move { dx: i16, dy: i16 },
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
    Scroll { dx: i16, dy: i16 },
    Ping { seq: u16 },
    Pong { seq: u16 },
    SessionStart { version: u8, device_id: [u8; 16] },
    SessionEnd,
    DeviceInfo { payload: String },
    Error { code: u16 },
    DragStart,
    DragEnd,
}

/// Decode error types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    EmptyPacket,
    VersionMismatch { expected: u8, got: u8 },
    UnknownType(u8),
    PayloadTooShort { msg_type: u8, expected: usize, got: usize },
    PayloadTooLarge { size: usize },
    InvalidPayload(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPacket => write!(f, "empty packet"),
            Self::VersionMismatch { expected, got } => {
                write!(f, "version mismatch: expected {expected}, got {got}")
            }
            Self::UnknownType(t) => write!(f, "unknown message type: 0x{t:02x}"),
            Self::PayloadTooShort { msg_type, expected, got } => {
                write!(f, "type 0x{msg_type:02x}: expected {expected} bytes, got {got}")
            }
            Self::PayloadTooLarge { size } => write!(f, "payload too large: {size} bytes"),
            Self::InvalidPayload(detail) => write!(f, "invalid payload: {detail}"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Encode a protocol message into a compact binary Vec<u8>.
pub fn encode(msg: &ProtocolMessage) -> Vec<u8> {
    match msg {
        ProtocolMessage::Move { dx, dy } => {
            let mut buf = Vec::with_capacity(5);
            buf.push(encode_header(MessageType::Move));
            buf.extend_from_slice(&dx.to_be_bytes());
            buf.extend_from_slice(&dy.to_be_bytes());
            buf
        }
        ProtocolMessage::LeftDown => vec![encode_header(MessageType::LeftDown)],
        ProtocolMessage::LeftUp => vec![encode_header(MessageType::LeftUp)],
        ProtocolMessage::RightDown => vec![encode_header(MessageType::RightDown)],
        ProtocolMessage::RightUp => vec![encode_header(MessageType::RightUp)],
        ProtocolMessage::Scroll { dx, dy } => {
            let mut buf = Vec::with_capacity(5);
            buf.push(encode_header(MessageType::Scroll));
            buf.extend_from_slice(&dx.to_be_bytes());
            buf.extend_from_slice(&dy.to_be_bytes());
            buf
        }
        ProtocolMessage::Ping { seq } => {
            let mut buf = Vec::with_capacity(3);
            buf.push(encode_header(MessageType::Ping));
            buf.extend_from_slice(&seq.to_be_bytes());
            buf
        }
        ProtocolMessage::Pong { seq } => {
            let mut buf = Vec::with_capacity(3);
            buf.push(encode_header(MessageType::Pong));
            buf.extend_from_slice(&seq.to_be_bytes());
            buf
        }
        ProtocolMessage::SessionStart { version, device_id } => {
            let mut buf = Vec::with_capacity(18);
            buf.push(encode_header(MessageType::SessionStart));
            buf.push(*version);
            buf.extend_from_slice(device_id);
            buf
        }
        ProtocolMessage::SessionEnd => vec![encode_header(MessageType::SessionEnd)],
        ProtocolMessage::DeviceInfo { payload } => {
            let json_bytes = payload.as_bytes();
            let len = json_bytes.len().min(65535) as u16;
            let mut buf = Vec::with_capacity(3 + len as usize);
            buf.push(encode_header(MessageType::DeviceInfo));
            buf.extend_from_slice(&len.to_be_bytes());
            buf.extend_from_slice(&json_bytes[..len as usize]);
            buf
        }
        ProtocolMessage::Error { code } => {
            let mut buf = Vec::with_capacity(3);
            buf.push(encode_header(MessageType::Error));
            buf.extend_from_slice(&code.to_be_bytes());
            buf
        }
        ProtocolMessage::DragStart => vec![encode_header(MessageType::DragStart)],
        ProtocolMessage::DragEnd => vec![encode_header(MessageType::DragEnd)],
    }
}

/// Decode a binary packet into a protocol message.
pub fn decode(data: &[u8]) -> Result<ProtocolMessage, DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::EmptyPacket);
    }

    let header = data[0];
    let version = (header >> 4) & 0x0F;
    let msg_type_raw = header & 0x0F;

    if version != PROTOCOL_VERSION {
        return Err(DecodeError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            got: version,
        });
    }

    let msg_type = MessageType::from_u8(msg_type_raw)
        .ok_or(DecodeError::UnknownType(msg_type_raw))?;

    match msg_type {
        MessageType::Move => {
            ensure_size(data, 5, msg_type_raw)?;
            let dx = i16::from_be_bytes([data[1], data[2]]);
            let dy = i16::from_be_bytes([data[3], data[4]]);
            Ok(ProtocolMessage::Move { dx, dy })
        }
        MessageType::LeftDown => Ok(ProtocolMessage::LeftDown),
        MessageType::LeftUp => Ok(ProtocolMessage::LeftUp),
        MessageType::RightDown => Ok(ProtocolMessage::RightDown),
        MessageType::RightUp => Ok(ProtocolMessage::RightUp),
        MessageType::Scroll => {
            ensure_size(data, 5, msg_type_raw)?;
            let dx = i16::from_be_bytes([data[1], data[2]]);
            let dy = i16::from_be_bytes([data[3], data[4]]);
            Ok(ProtocolMessage::Scroll { dx, dy })
        }
        MessageType::Ping => {
            ensure_size(data, 3, msg_type_raw)?;
            let seq = u16::from_be_bytes([data[1], data[2]]);
            Ok(ProtocolMessage::Ping { seq })
        }
        MessageType::Pong => {
            ensure_size(data, 3, msg_type_raw)?;
            let seq = u16::from_be_bytes([data[1], data[2]]);
            Ok(ProtocolMessage::Pong { seq })
        }
        MessageType::SessionStart => {
            ensure_size(data, 18, msg_type_raw)?;
            let version = data[1];
            let mut device_id = [0u8; 16];
            device_id.copy_from_slice(&data[2..18]);
            Ok(ProtocolMessage::SessionStart { version, device_id })
        }
        MessageType::SessionEnd => Ok(ProtocolMessage::SessionEnd),
        MessageType::DeviceInfo => {
            ensure_size(data, 3, msg_type_raw)?;
            let json_len = u16::from_be_bytes([data[1], data[2]]) as usize;
            if json_len > 65535 {
                return Err(DecodeError::PayloadTooLarge { size: json_len });
            }
            ensure_size(data, 3 + json_len, msg_type_raw)?;
            let payload = std::str::from_utf8(&data[3..3 + json_len])
                .map_err(|e| DecodeError::InvalidPayload(e.to_string()))?
                .to_string();
            Ok(ProtocolMessage::DeviceInfo { payload })
        }
        MessageType::Error => {
            ensure_size(data, 3, msg_type_raw)?;
            let code = u16::from_be_bytes([data[1], data[2]]);
            Ok(ProtocolMessage::Error { code })
        }
        MessageType::DragStart => Ok(ProtocolMessage::DragStart),
        MessageType::DragEnd => Ok(ProtocolMessage::DragEnd),
    }
}

/// Encode header byte: [version:4][type:4].
fn encode_header(msg_type: MessageType) -> u8 {
    ((PROTOCOL_VERSION & 0x0F) << 4) | (msg_type as u8 & 0x0F)
}

/// Ensure the data slice has at least `required` bytes.
fn ensure_size(data: &[u8], required: usize, msg_type: u8) -> Result<(), DecodeError> {
    if data.len() < required {
        Err(DecodeError::PayloadTooShort {
            msg_type,
            expected: required,
            got: data.len(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(msg: &ProtocolMessage) -> ProtocolMessage {
        let encoded = encode(msg);
        decode(&encoded).expect("decode failed")
    }

    #[test]
    fn move_roundtrip() {
        let msg = ProtocolMessage::Move { dx: 100, dy: -50 };
        assert_eq!(roundtrip(&msg), msg);
    }

    #[test]
    fn move_packet_size() {
        let encoded = encode(&ProtocolMessage::Move { dx: 10, dy: -4 });
        assert_eq!(encoded.len(), 5);
    }

    #[test]
    fn move_extreme_values() {
        let msg = ProtocolMessage::Move { dx: i16::MAX, dy: i16::MIN };
        assert_eq!(roundtrip(&msg), msg);
    }

    #[test]
    fn move_zero() {
        let msg = ProtocolMessage::Move { dx: 0, dy: 0 };
        assert_eq!(roundtrip(&msg), msg);
    }

    #[test]
    fn click_events_roundtrip() {
        assert_eq!(roundtrip(&ProtocolMessage::LeftDown), ProtocolMessage::LeftDown);
        assert_eq!(roundtrip(&ProtocolMessage::LeftUp), ProtocolMessage::LeftUp);
        assert_eq!(roundtrip(&ProtocolMessage::RightDown), ProtocolMessage::RightDown);
        assert_eq!(roundtrip(&ProtocolMessage::RightUp), ProtocolMessage::RightUp);
    }

    #[test]
    fn click_packet_size() {
        assert_eq!(encode(&ProtocolMessage::LeftDown).len(), 1);
        assert_eq!(encode(&ProtocolMessage::LeftUp).len(), 1);
        assert_eq!(encode(&ProtocolMessage::RightDown).len(), 1);
        assert_eq!(encode(&ProtocolMessage::RightUp).len(), 1);
    }

    #[test]
    fn scroll_roundtrip() {
        let msg = ProtocolMessage::Scroll { dx: 0, dy: -120 };
        assert_eq!(roundtrip(&msg), msg);
    }

    #[test]
    fn ping_pong_roundtrip() {
        let ping = ProtocolMessage::Ping { seq: 42 };
        assert_eq!(roundtrip(&ping), ping);

        let pong = ProtocolMessage::Pong { seq: 65535 };
        assert_eq!(roundtrip(&pong), pong);
    }

    #[test]
    fn session_start_roundtrip() {
        let device_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let msg = ProtocolMessage::SessionStart {
            version: PROTOCOL_VERSION,
            device_id,
        };
        assert_eq!(roundtrip(&msg), msg);
        assert_eq!(encode(&msg).len(), 18);
    }

    #[test]
    fn session_end_roundtrip() {
        assert_eq!(roundtrip(&ProtocolMessage::SessionEnd), ProtocolMessage::SessionEnd);
    }

    #[test]
    fn drag_events_roundtrip() {
        assert_eq!(roundtrip(&ProtocolMessage::DragStart), ProtocolMessage::DragStart);
        assert_eq!(roundtrip(&ProtocolMessage::DragEnd), ProtocolMessage::DragEnd);
    }

    #[test]
    fn error_roundtrip() {
        let msg = ProtocolMessage::Error { code: ErrorCode::AuthFailed as u16 };
        assert_eq!(roundtrip(&msg), msg);
    }

    #[test]
    fn device_info_roundtrip() {
        let payload = r#"{"deviceName":"Pixel 9","platform":"android","appVersion":"1.0.0"}"#.to_string();
        let msg = ProtocolMessage::DeviceInfo { payload: payload.clone() };
        assert_eq!(roundtrip(&msg), msg);
    }

    #[test]
    fn reject_empty_packet() {
        assert_eq!(decode(&[]), Err(DecodeError::EmptyPacket));
    }

    #[test]
    fn reject_wrong_version() {
        let mut data = encode(&ProtocolMessage::Move { dx: 0, dy: 0 });
        // Set version to 2
        data[0] = (2 << 4) | (data[0] & 0x0F);
        assert!(matches!(decode(&data), Err(DecodeError::VersionMismatch { .. })));
    }

    #[test]
    fn reject_unknown_type() {
        let header = (PROTOCOL_VERSION << 4) | 0x0F;
        assert!(matches!(decode(&[header]), Err(DecodeError::UnknownType(0x0F))));
    }

    #[test]
    fn reject_truncated_move() {
        let header = (PROTOCOL_VERSION << 4) | MessageType::Move as u8;
        assert!(matches!(decode(&[header, 0, 0]), Err(DecodeError::PayloadTooShort { .. })));
    }

    #[test]
    fn header_encoding() {
        let encoded = encode(&ProtocolMessage::Move { dx: 0, dy: 0 });
        let version = (encoded[0] >> 4) & 0x0F;
        let msg_type = encoded[0] & 0x0F;
        assert_eq!(version, PROTOCOL_VERSION);
        assert_eq!(msg_type, MessageType::Move as u8);
    }

    // Cross-implementation compatibility: verify exact byte sequences
    // match the TypeScript encoder output
    #[test]
    fn wire_format_move() {
        let encoded = encode(&ProtocolMessage::Move { dx: 10, dy: -4 });
        assert_eq!(encoded[0], 0x11); // version=1, type=MOVE(0x01)
        assert_eq!(encoded[1..3], 10_i16.to_be_bytes()); // dx
        assert_eq!(encoded[3..5], (-4_i16).to_be_bytes()); // dy
    }

    #[test]
    fn wire_format_left_click() {
        let down = encode(&ProtocolMessage::LeftDown);
        assert_eq!(down, vec![0x12]); // version=1, type=LEFT_DOWN(0x02)

        let up = encode(&ProtocolMessage::LeftUp);
        assert_eq!(up, vec![0x13]); // version=1, type=LEFT_UP(0x03)
    }
}

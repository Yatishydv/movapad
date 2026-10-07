/**
 * MovaPad Binary Protocol — Constants
 *
 * Centralizes all protocol magic values, version numbers, and limits.
 * Nothing in the protocol should use raw numeric literals.
 */

/** Current protocol version. Encoded in upper 4 bits of header byte. */
export const PROTOCOL_VERSION = 1;

/** Maximum protocol version we can encode (4 bits). */
export const MAX_PROTOCOL_VERSION = 15;

/** Maximum payload size for variable-length messages (64 KB). */
export const MAX_PAYLOAD_SIZE = 65535;

/** Minimum valid packet size (header byte only). */
export const MIN_PACKET_SIZE = 1;

/** Maximum valid i16 value for dx/dy. */
export const MAX_I16 = 32767;

/** Minimum valid i16 value for dx/dy. */
export const MIN_I16 = -32768;

/**
 * Message type identifiers.
 * Encoded in lower 4 bits of header byte.
 */
export const MessageType = {
  MOVE:          0x01,
  LEFT_DOWN:     0x02,
  LEFT_UP:       0x03,
  RIGHT_DOWN:    0x04,
  RIGHT_UP:      0x05,
  SCROLL:        0x06,
  PING:          0x07,
  PONG:          0x08,
  SESSION_START: 0x09,
  SESSION_END:   0x0a,
  DEVICE_INFO:   0x0b,
  ERROR:         0x0c,
  DRAG_START:    0x0d,
  DRAG_END:      0x0e,
} as const;

export type MessageTypeValue = typeof MessageType[keyof typeof MessageType];

/** Human-readable names for message types (for logging, never sent over wire). */
export const MESSAGE_TYPE_NAMES: Readonly<Record<MessageTypeValue, string>> = {
  [MessageType.MOVE]:          'MOVE',
  [MessageType.LEFT_DOWN]:     'LEFT_DOWN',
  [MessageType.LEFT_UP]:       'LEFT_UP',
  [MessageType.RIGHT_DOWN]:    'RIGHT_DOWN',
  [MessageType.RIGHT_UP]:      'RIGHT_UP',
  [MessageType.SCROLL]:        'SCROLL',
  [MessageType.PING]:          'PING',
  [MessageType.PONG]:          'PONG',
  [MessageType.SESSION_START]: 'SESSION_START',
  [MessageType.SESSION_END]:   'SESSION_END',
  [MessageType.DEVICE_INFO]:   'DEVICE_INFO',
  [MessageType.ERROR]:         'ERROR',
  [MessageType.DRAG_START]:    'DRAG_START',
  [MessageType.DRAG_END]:      'DRAG_END',
};

/**
 * Expected payload sizes for fixed-size message types.
 * Variable-length messages (DEVICE_INFO) are not included.
 */
export const PAYLOAD_SIZES: Readonly<Partial<Record<MessageTypeValue, number>>> = {
  [MessageType.MOVE]:          4, // dx: i16, dy: i16
  [MessageType.LEFT_DOWN]:     0,
  [MessageType.LEFT_UP]:       0,
  [MessageType.RIGHT_DOWN]:    0,
  [MessageType.RIGHT_UP]:      0,
  [MessageType.SCROLL]:        4, // dx: i16, dy: i16
  [MessageType.PING]:          2, // seq: u16
  [MessageType.PONG]:          2, // seq: u16
  [MessageType.SESSION_START]: 17, // version: u8, device_id: 16 bytes
  [MessageType.SESSION_END]:   0,
  [MessageType.ERROR]:         2, // code: u16
  [MessageType.DRAG_START]:    0,
  [MessageType.DRAG_END]:      0,
};

/** Error codes sent in ERROR messages. */
export const ErrorCode = {
  UNKNOWN:              0x0000,
  PROTOCOL_MISMATCH:    0x0001,
  AUTH_FAILED:          0x0002,
  SESSION_EXPIRED:      0x0003,
  DEVICE_REVOKED:       0x0004,
  RATE_LIMITED:         0x0005,
  INVALID_MESSAGE:      0x0006,
} as const;

export type ErrorCodeValue = typeof ErrorCode[keyof typeof ErrorCode];

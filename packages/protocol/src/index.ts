/**
 * MovaPad Binary Protocol
 *
 * Public API for encoding/decoding the MovaPad binary protocol.
 */

export {
  PROTOCOL_VERSION,
  MessageType,
  ErrorCode,
  MESSAGE_TYPE_NAMES,
  PAYLOAD_SIZES,
  MAX_PAYLOAD_SIZE,
  MIN_PACKET_SIZE,
  MAX_I16,
  MIN_I16,
  type MessageTypeValue,
  type ErrorCodeValue,
} from './constants.js';

export type {
  ProtocolMessage,
  MoveMessage,
  LeftDownMessage,
  LeftUpMessage,
  RightDownMessage,
  RightUpMessage,
  ScrollMessage,
  PingMessage,
  PongMessage,
  SessionStartMessage,
  SessionEndMessage,
  DeviceInfoMessage,
  DeviceInfoPayload,
  ErrorMessage,
  DragStartMessage,
  DragEndMessage,
  MessageOfType,
} from './types.js';

export { encode } from './encoder.js';
export { decode, type DecodeResult, type DecodeError, type DecodeErrorCode } from './decoder.js';

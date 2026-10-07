/**
 * MovaPad Binary Protocol — Message Types
 *
 * Discriminated union of all protocol messages.
 * Each message carries exactly the data it needs — no optional fields.
 */

import { MessageType, type MessageTypeValue, type ErrorCodeValue } from './constants.js';

/** Cursor movement (relative). Sent on the pointer channel. */
export interface MoveMessage {
  readonly type: typeof MessageType.MOVE;
  readonly dx: number; // i16: -32768..32767
  readonly dy: number; // i16: -32768..32767
}

/** Left mouse button pressed. */
export interface LeftDownMessage {
  readonly type: typeof MessageType.LEFT_DOWN;
}

/** Left mouse button released. */
export interface LeftUpMessage {
  readonly type: typeof MessageType.LEFT_UP;
}

/** Right mouse button pressed. */
export interface RightDownMessage {
  readonly type: typeof MessageType.RIGHT_DOWN;
}

/** Right mouse button released. */
export interface RightUpMessage {
  readonly type: typeof MessageType.RIGHT_UP;
}

/** Scroll event (relative). */
export interface ScrollMessage {
  readonly type: typeof MessageType.SCROLL;
  readonly dx: number; // i16: horizontal scroll
  readonly dy: number; // i16: vertical scroll
}

/** Ping for RTT measurement. */
export interface PingMessage {
  readonly type: typeof MessageType.PING;
  readonly seq: number; // u16: sequence number
}

/** Pong response to ping. */
export interface PongMessage {
  readonly type: typeof MessageType.PONG;
  readonly seq: number; // u16: sequence number
}

/** Session initialization. Sent once after DataChannel opens. */
export interface SessionStartMessage {
  readonly type: typeof MessageType.SESSION_START;
  readonly version: number;    // u8: protocol version
  readonly deviceId: Uint8Array; // 16 bytes: device identity
}

/** Session termination. Sent before graceful disconnect. */
export interface SessionEndMessage {
  readonly type: typeof MessageType.SESSION_END;
}

/** Device metadata exchange. Variable-length JSON payload. */
export interface DeviceInfoMessage {
  readonly type: typeof MessageType.DEVICE_INFO;
  readonly payload: DeviceInfoPayload;
}

export interface DeviceInfoPayload {
  readonly deviceName: string;
  readonly platform: 'android' | 'ios' | 'windows' | 'macos';
  readonly appVersion: string;
}

/** Error notification. */
export interface ErrorMessage {
  readonly type: typeof MessageType.ERROR;
  readonly code: ErrorCodeValue;
}

/** Drag operation started (left button held). */
export interface DragStartMessage {
  readonly type: typeof MessageType.DRAG_START;
}

/** Drag operation ended (left button released). */
export interface DragEndMessage {
  readonly type: typeof MessageType.DRAG_END;
}

/** Discriminated union of all protocol messages. */
export type ProtocolMessage =
  | MoveMessage
  | LeftDownMessage
  | LeftUpMessage
  | RightDownMessage
  | RightUpMessage
  | ScrollMessage
  | PingMessage
  | PongMessage
  | SessionStartMessage
  | SessionEndMessage
  | DeviceInfoMessage
  | ErrorMessage
  | DragStartMessage
  | DragEndMessage;

/** Extract message type from a ProtocolMessage. */
export type MessageOfType<T extends MessageTypeValue> =
  Extract<ProtocolMessage, { type: T }>;

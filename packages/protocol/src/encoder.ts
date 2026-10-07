/**
 * MovaPad Binary Protocol — Encoder
 *
 * Encodes ProtocolMessage objects into compact binary ArrayBuffers.
 *
 * Wire format:
 *   Byte 0: [version:4][type:4]
 *   Byte 1+: payload (type-dependent)
 *
 * Performance notes:
 * - MOVE packets are 5 bytes (vs ~60 bytes for JSON equivalent)
 * - Click packets are 1 byte
 * - Encoder allocates minimally-sized buffers
 * - No string serialization for pointer-channel messages
 */

import {
  PROTOCOL_VERSION,
  MessageType,
  PAYLOAD_SIZES,
  type MessageTypeValue,
} from './constants.js';
import type { ProtocolMessage } from './types.js';

/**
 * Encode the header byte: upper 4 bits = version, lower 4 bits = type.
 */
function encodeHeader(msgType: MessageTypeValue): number {
  return ((PROTOCOL_VERSION & 0x0f) << 4) | (msgType & 0x0f);
}

/**
 * Write a signed 16-bit integer to a DataView in big-endian order.
 */
function writeI16(view: DataView, offset: number, value: number): void {
  // Clamp to i16 range to prevent DataView errors
  const clamped = Math.max(-32768, Math.min(32767, Math.round(value)));
  view.setInt16(offset, clamped, false); // big-endian
}

/**
 * Write an unsigned 16-bit integer to a DataView in big-endian order.
 */
function writeU16(view: DataView, offset: number, value: number): void {
  const clamped = Math.max(0, Math.min(65535, Math.round(value)));
  view.setUint16(offset, clamped, false); // big-endian
}

/**
 * Encode a ProtocolMessage into a compact binary ArrayBuffer.
 *
 * @throws {Error} If the message type is unknown or payload is invalid.
 */
export function encode(message: ProtocolMessage): ArrayBuffer {
  switch (message.type) {
    case MessageType.MOVE: {
      const buf = new ArrayBuffer(1 + (PAYLOAD_SIZES[MessageType.MOVE] ?? 4));
      const view = new DataView(buf);
      view.setUint8(0, encodeHeader(MessageType.MOVE));
      writeI16(view, 1, message.dx);
      writeI16(view, 3, message.dy);
      return buf;
    }

    case MessageType.LEFT_DOWN: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.LEFT_DOWN));
      return buf;
    }

    case MessageType.LEFT_UP: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.LEFT_UP));
      return buf;
    }

    case MessageType.RIGHT_DOWN: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.RIGHT_DOWN));
      return buf;
    }

    case MessageType.RIGHT_UP: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.RIGHT_UP));
      return buf;
    }

    case MessageType.SCROLL: {
      const buf = new ArrayBuffer(1 + (PAYLOAD_SIZES[MessageType.SCROLL] ?? 4));
      const view = new DataView(buf);
      view.setUint8(0, encodeHeader(MessageType.SCROLL));
      writeI16(view, 1, message.dx);
      writeI16(view, 3, message.dy);
      return buf;
    }

    case MessageType.PING: {
      const buf = new ArrayBuffer(1 + (PAYLOAD_SIZES[MessageType.PING] ?? 2));
      const view = new DataView(buf);
      view.setUint8(0, encodeHeader(MessageType.PING));
      writeU16(view, 1, message.seq);
      return buf;
    }

    case MessageType.PONG: {
      const buf = new ArrayBuffer(1 + (PAYLOAD_SIZES[MessageType.PONG] ?? 2));
      const view = new DataView(buf);
      view.setUint8(0, encodeHeader(MessageType.PONG));
      writeU16(view, 1, message.seq);
      return buf;
    }

    case MessageType.SESSION_START: {
      const expectedSize = PAYLOAD_SIZES[MessageType.SESSION_START] ?? 17;
      const buf = new ArrayBuffer(1 + expectedSize);
      const view = new DataView(buf);
      const bytes = new Uint8Array(buf);
      view.setUint8(0, encodeHeader(MessageType.SESSION_START));
      view.setUint8(1, message.version & 0xff);
      if (message.deviceId.length !== 16) {
        throw new Error(
          `SESSION_START requires 16-byte deviceId, got ${message.deviceId.length}`
        );
      }
      bytes.set(message.deviceId, 2);
      return buf;
    }

    case MessageType.SESSION_END: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.SESSION_END));
      return buf;
    }

    case MessageType.DEVICE_INFO: {
      const jsonStr = JSON.stringify(message.payload);
      const textEncoder = new TextEncoder();
      const jsonBytes = textEncoder.encode(jsonStr);
      // Header (1) + length (2) + JSON payload
      const buf = new ArrayBuffer(1 + 2 + jsonBytes.length);
      const view = new DataView(buf);
      const bytes = new Uint8Array(buf);
      view.setUint8(0, encodeHeader(MessageType.DEVICE_INFO));
      writeU16(view, 1, jsonBytes.length);
      bytes.set(jsonBytes, 3);
      return buf;
    }

    case MessageType.ERROR: {
      const buf = new ArrayBuffer(1 + (PAYLOAD_SIZES[MessageType.ERROR] ?? 2));
      const view = new DataView(buf);
      view.setUint8(0, encodeHeader(MessageType.ERROR));
      writeU16(view, 1, message.code);
      return buf;
    }

    case MessageType.DRAG_START: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.DRAG_START));
      return buf;
    }

    case MessageType.DRAG_END: {
      const buf = new ArrayBuffer(1);
      new DataView(buf).setUint8(0, encodeHeader(MessageType.DRAG_END));
      return buf;
    }

    default: {
      // Exhaustive check — TypeScript will error if a case is missing
      const _exhaustive: never = message;
      throw new Error(`Unknown message type: ${(_exhaustive as ProtocolMessage).type}`);
    }
  }
}

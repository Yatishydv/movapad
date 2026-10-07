/**
 * MovaPad Binary Protocol — Decoder
 *
 * Decodes compact binary ArrayBuffers into typed ProtocolMessage objects.
 *
 * Security notes:
 * - Validates packet size before reading payload
 * - Validates protocol version for compatibility
 * - Returns structured errors rather than throwing on malformed input
 * - Caps variable-length payloads (DEVICE_INFO) at MAX_PAYLOAD_SIZE
 */

import {
  PROTOCOL_VERSION,
  MessageType,
  PAYLOAD_SIZES,
  MAX_PAYLOAD_SIZE,
  type MessageTypeValue,
  type ErrorCodeValue,
} from './constants.js';
import type {
  ProtocolMessage,
  DeviceInfoPayload,
} from './types.js';

/** Result of a decode operation. */
export type DecodeResult =
  | { ok: true; message: ProtocolMessage }
  | { ok: false; error: DecodeError };

/** Structured decode error. */
export interface DecodeError {
  readonly code: DecodeErrorCode;
  readonly detail: string;
}

export type DecodeErrorCode =
  | 'EMPTY_PACKET'
  | 'VERSION_MISMATCH'
  | 'UNKNOWN_TYPE'
  | 'PAYLOAD_TOO_SHORT'
  | 'PAYLOAD_TOO_LARGE'
  | 'INVALID_PAYLOAD';

/**
 * Extract version (upper 4 bits) and type (lower 4 bits) from header byte.
 */
function parseHeader(header: number): { version: number; msgType: number } {
  return {
    version: (header >> 4) & 0x0f,
    msgType: header & 0x0f,
  };
}

/**
 * Read a signed 16-bit integer from a DataView in big-endian order.
 */
function readI16(view: DataView, offset: number): number {
  return view.getInt16(offset, false); // big-endian
}

/**
 * Read an unsigned 16-bit integer from a DataView in big-endian order.
 */
function readU16(view: DataView, offset: number): number {
  return view.getUint16(offset, false); // big-endian
}

/**
 * Validate that the message type is known.
 */
function isValidMessageType(value: number): value is MessageTypeValue {
  return value >= MessageType.MOVE && value <= MessageType.DRAG_END;
}

/**
 * Validate that the packet has enough bytes for the expected payload.
 */
function validatePayloadSize(
  msgType: MessageTypeValue,
  packetLength: number
): DecodeError | null {
  const expectedPayload = PAYLOAD_SIZES[msgType];
  if (expectedPayload !== undefined) {
    const expectedTotal = 1 + expectedPayload; // header + payload
    if (packetLength < expectedTotal) {
      return {
        code: 'PAYLOAD_TOO_SHORT',
        detail: `Type 0x${msgType.toString(16)}: expected ${expectedTotal} bytes, got ${packetLength}`,
      };
    }
  }
  return null;
}

/**
 * Decode a binary packet into a ProtocolMessage.
 *
 * Returns a DecodeResult discriminated union — callers must check `ok`
 * before accessing the message.
 */
export function decode(data: ArrayBuffer | Uint8Array): DecodeResult {
  const buffer = data instanceof Uint8Array ? data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) : data;
  const bytes = new Uint8Array(buffer);

  if (bytes.length === 0) {
    return { ok: false, error: { code: 'EMPTY_PACKET', detail: 'Received empty packet' } };
  }

  const view = new DataView(buffer);
  const { version, msgType } = parseHeader(bytes[0]!);

  // Version compatibility check
  if (version !== PROTOCOL_VERSION) {
    return {
      ok: false,
      error: {
        code: 'VERSION_MISMATCH',
        detail: `Expected protocol version ${PROTOCOL_VERSION}, got ${version}`,
      },
    };
  }

  if (!isValidMessageType(msgType)) {
    return {
      ok: false,
      error: {
        code: 'UNKNOWN_TYPE',
        detail: `Unknown message type: 0x${msgType.toString(16)}`,
      },
    };
  }

  // Validate payload size for fixed-size messages
  const sizeError = validatePayloadSize(msgType, bytes.length);
  if (sizeError !== null) {
    return { ok: false, error: sizeError };
  }

  switch (msgType) {
    case MessageType.MOVE:
      return {
        ok: true,
        message: {
          type: MessageType.MOVE,
          dx: readI16(view, 1),
          dy: readI16(view, 3),
        },
      };

    case MessageType.LEFT_DOWN:
      return { ok: true, message: { type: MessageType.LEFT_DOWN } };

    case MessageType.LEFT_UP:
      return { ok: true, message: { type: MessageType.LEFT_UP } };

    case MessageType.RIGHT_DOWN:
      return { ok: true, message: { type: MessageType.RIGHT_DOWN } };

    case MessageType.RIGHT_UP:
      return { ok: true, message: { type: MessageType.RIGHT_UP } };

    case MessageType.SCROLL:
      return {
        ok: true,
        message: {
          type: MessageType.SCROLL,
          dx: readI16(view, 1),
          dy: readI16(view, 3),
        },
      };

    case MessageType.PING:
      return {
        ok: true,
        message: {
          type: MessageType.PING,
          seq: readU16(view, 1),
        },
      };

    case MessageType.PONG:
      return {
        ok: true,
        message: {
          type: MessageType.PONG,
          seq: readU16(view, 1),
        },
      };

    case MessageType.SESSION_START: {
      const msgVersion = view.getUint8(1);
      const deviceId = new Uint8Array(buffer, 2, 16);
      return {
        ok: true,
        message: {
          type: MessageType.SESSION_START,
          version: msgVersion,
          deviceId: new Uint8Array(deviceId), // defensive copy
        },
      };
    }

    case MessageType.SESSION_END:
      return { ok: true, message: { type: MessageType.SESSION_END } };

    case MessageType.DEVICE_INFO: {
      if (bytes.length < 3) {
        return {
          ok: false,
          error: {
            code: 'PAYLOAD_TOO_SHORT',
            detail: 'DEVICE_INFO requires at least 3 bytes (header + length)',
          },
        };
      }
      const jsonLength = readU16(view, 1);
      if (jsonLength > MAX_PAYLOAD_SIZE) {
        return {
          ok: false,
          error: {
            code: 'PAYLOAD_TOO_LARGE',
            detail: `DEVICE_INFO payload ${jsonLength} exceeds max ${MAX_PAYLOAD_SIZE}`,
          },
        };
      }
      if (bytes.length < 3 + jsonLength) {
        return {
          ok: false,
          error: {
            code: 'PAYLOAD_TOO_SHORT',
            detail: `DEVICE_INFO declares ${jsonLength} bytes but packet only has ${bytes.length - 3}`,
          },
        };
      }
      const jsonBytes = new Uint8Array(buffer, 3, jsonLength);
      const textDecoder = new TextDecoder();
      let payload: DeviceInfoPayload;
      try {
        const parsed: unknown = JSON.parse(textDecoder.decode(jsonBytes));
        if (!isDeviceInfoPayload(parsed)) {
          return {
            ok: false,
            error: {
              code: 'INVALID_PAYLOAD',
              detail: 'DEVICE_INFO payload has invalid structure',
            },
          };
        }
        payload = parsed;
      } catch {
        return {
          ok: false,
          error: {
            code: 'INVALID_PAYLOAD',
            detail: 'DEVICE_INFO contains invalid JSON',
          },
        };
      }
      return {
        ok: true,
        message: {
          type: MessageType.DEVICE_INFO,
          payload,
        },
      };
    }

    case MessageType.ERROR:
      return {
        ok: true,
        message: {
          type: MessageType.ERROR,
          code: readU16(view, 1) as ErrorCodeValue,
        },
      };

    case MessageType.DRAG_START:
      return { ok: true, message: { type: MessageType.DRAG_START } };

    case MessageType.DRAG_END:
      return { ok: true, message: { type: MessageType.DRAG_END } };

    default: {
      // Should be unreachable after isValidMessageType check
      return {
        ok: false,
        error: {
          code: 'UNKNOWN_TYPE',
          detail: `Unhandled message type: 0x${(msgType as number).toString(16)}`,
        },
      };
    }
  }
}

/**
 * Type guard for DeviceInfoPayload.
 * Validates structure without trusting client data.
 */
function isDeviceInfoPayload(value: unknown): value is DeviceInfoPayload {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  if (typeof obj['deviceName'] !== 'string') return false;
  if (typeof obj['platform'] !== 'string') return false;
  if (!['android', 'ios', 'windows', 'macos'].includes(obj['platform'] as string)) return false;
  if (typeof obj['appVersion'] !== 'string') return false;
  return true;
}

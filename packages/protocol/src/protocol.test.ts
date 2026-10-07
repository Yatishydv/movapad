/**
 * MovaPad Protocol — Unit Tests
 *
 * Tests encode/decode roundtrip for every message type,
 * edge cases, malformed packets, and protocol versioning.
 */

import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { encode, decode, MessageType, ErrorCode, PROTOCOL_VERSION } from './index.js';
import type { ProtocolMessage } from './index.js';

// ─── Helpers ────────────────────────────────────────────────────────────────

function roundtrip(msg: ProtocolMessage): ProtocolMessage {
  const encoded = encode(msg);
  const result = decode(encoded);
  assert.equal(result.ok, true, `Decode failed: ${!result.ok ? result.error.detail : ''}`);
  if (!result.ok) throw new Error('unreachable');
  return result.message;
}

// ─── MOVE ───────────────────────────────────────────────────────────────────

describe('MOVE', () => {
  it('encodes to exactly 5 bytes', () => {
    const buf = encode({ type: MessageType.MOVE, dx: 10, dy: -4 });
    assert.equal(buf.byteLength, 5);
  });

  it('roundtrips positive deltas', () => {
    const msg = roundtrip({ type: MessageType.MOVE, dx: 100, dy: 200 });
    assert.equal(msg.type, MessageType.MOVE);
    if (msg.type === MessageType.MOVE) {
      assert.equal(msg.dx, 100);
      assert.equal(msg.dy, 200);
    }
  });

  it('roundtrips negative deltas', () => {
    const msg = roundtrip({ type: MessageType.MOVE, dx: -50, dy: -100 });
    assert.equal(msg.type, MessageType.MOVE);
    if (msg.type === MessageType.MOVE) {
      assert.equal(msg.dx, -50);
      assert.equal(msg.dy, -100);
    }
  });

  it('roundtrips zero deltas', () => {
    const msg = roundtrip({ type: MessageType.MOVE, dx: 0, dy: 0 });
    assert.equal(msg.type, MessageType.MOVE);
    if (msg.type === MessageType.MOVE) {
      assert.equal(msg.dx, 0);
      assert.equal(msg.dy, 0);
    }
  });

  it('roundtrips maximum i16 values', () => {
    const msg = roundtrip({ type: MessageType.MOVE, dx: 32767, dy: -32768 });
    assert.equal(msg.type, MessageType.MOVE);
    if (msg.type === MessageType.MOVE) {
      assert.equal(msg.dx, 32767);
      assert.equal(msg.dy, -32768);
    }
  });

  it('clamps values exceeding i16 range', () => {
    const buf = encode({ type: MessageType.MOVE, dx: 40000, dy: -40000 });
    const result = decode(buf);
    assert.equal(result.ok, true);
    if (result.ok && result.message.type === MessageType.MOVE) {
      assert.equal(result.message.dx, 32767);
      assert.equal(result.message.dy, -32768);
    }
  });
});

// ─── Click Events ───────────────────────────────────────────────────────────

describe('Click events', () => {
  it('LEFT_DOWN roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.LEFT_DOWN });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.LEFT_DOWN });
    assert.equal(msg.type, MessageType.LEFT_DOWN);
  });

  it('LEFT_UP roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.LEFT_UP });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.LEFT_UP });
    assert.equal(msg.type, MessageType.LEFT_UP);
  });

  it('RIGHT_DOWN roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.RIGHT_DOWN });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.RIGHT_DOWN });
    assert.equal(msg.type, MessageType.RIGHT_DOWN);
  });

  it('RIGHT_UP roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.RIGHT_UP });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.RIGHT_UP });
    assert.equal(msg.type, MessageType.RIGHT_UP);
  });
});

// ─── SCROLL ─────────────────────────────────────────────────────────────────

describe('SCROLL', () => {
  it('encodes to exactly 5 bytes', () => {
    const buf = encode({ type: MessageType.SCROLL, dx: 0, dy: -3 });
    assert.equal(buf.byteLength, 5);
  });

  it('roundtrips vertical scroll', () => {
    const msg = roundtrip({ type: MessageType.SCROLL, dx: 0, dy: -120 });
    assert.equal(msg.type, MessageType.SCROLL);
    if (msg.type === MessageType.SCROLL) {
      assert.equal(msg.dx, 0);
      assert.equal(msg.dy, -120);
    }
  });

  it('roundtrips horizontal scroll', () => {
    const msg = roundtrip({ type: MessageType.SCROLL, dx: 50, dy: 0 });
    assert.equal(msg.type, MessageType.SCROLL);
    if (msg.type === MessageType.SCROLL) {
      assert.equal(msg.dx, 50);
      assert.equal(msg.dy, 0);
    }
  });
});

// ─── PING / PONG ────────────────────────────────────────────────────────────

describe('PING / PONG', () => {
  it('PING roundtrips with seq number (3 bytes)', () => {
    const buf = encode({ type: MessageType.PING, seq: 42 });
    assert.equal(buf.byteLength, 3);
    const msg = roundtrip({ type: MessageType.PING, seq: 42 });
    assert.equal(msg.type, MessageType.PING);
    if (msg.type === MessageType.PING) {
      assert.equal(msg.seq, 42);
    }
  });

  it('PONG roundtrips with seq number (3 bytes)', () => {
    const buf = encode({ type: MessageType.PONG, seq: 65535 });
    assert.equal(buf.byteLength, 3);
    const msg = roundtrip({ type: MessageType.PONG, seq: 65535 });
    assert.equal(msg.type, MessageType.PONG);
    if (msg.type === MessageType.PONG) {
      assert.equal(msg.seq, 65535);
    }
  });
});

// ─── SESSION_START ──────────────────────────────────────────────────────────

describe('SESSION_START', () => {
  it('encodes to exactly 18 bytes', () => {
    const deviceId = new Uint8Array(16).fill(0xab);
    const buf = encode({
      type: MessageType.SESSION_START,
      version: PROTOCOL_VERSION,
      deviceId,
    });
    assert.equal(buf.byteLength, 18);
  });

  it('roundtrips device ID correctly', () => {
    const deviceId = new Uint8Array([
      0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
      0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    ]);
    const msg = roundtrip({
      type: MessageType.SESSION_START,
      version: PROTOCOL_VERSION,
      deviceId,
    });
    assert.equal(msg.type, MessageType.SESSION_START);
    if (msg.type === MessageType.SESSION_START) {
      assert.equal(msg.version, PROTOCOL_VERSION);
      assert.deepEqual(msg.deviceId, deviceId);
    }
  });

  it('throws on wrong deviceId length', () => {
    assert.throws(() => {
      encode({
        type: MessageType.SESSION_START,
        version: 1,
        deviceId: new Uint8Array(8), // wrong length
      });
    }, /16-byte deviceId/);
  });
});

// ─── SESSION_END ────────────────────────────────────────────────────────────

describe('SESSION_END', () => {
  it('roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.SESSION_END });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.SESSION_END });
    assert.equal(msg.type, MessageType.SESSION_END);
  });
});

// ─── DEVICE_INFO ────────────────────────────────────────────────────────────

describe('DEVICE_INFO', () => {
  it('roundtrips device metadata', () => {
    const msg = roundtrip({
      type: MessageType.DEVICE_INFO,
      payload: {
        deviceName: 'Pixel 9 Pro',
        platform: 'android',
        appVersion: '1.0.0',
      },
    });
    assert.equal(msg.type, MessageType.DEVICE_INFO);
    if (msg.type === MessageType.DEVICE_INFO) {
      assert.equal(msg.payload.deviceName, 'Pixel 9 Pro');
      assert.equal(msg.payload.platform, 'android');
      assert.equal(msg.payload.appVersion, '1.0.0');
    }
  });

  it('handles all platforms', () => {
    for (const platform of ['android', 'ios', 'windows', 'macos'] as const) {
      const msg = roundtrip({
        type: MessageType.DEVICE_INFO,
        payload: { deviceName: 'Test', platform, appVersion: '1.0.0' },
      });
      if (msg.type === MessageType.DEVICE_INFO) {
        assert.equal(msg.payload.platform, platform);
      }
    }
  });
});

// ─── ERROR ──────────────────────────────────────────────────────────────────

describe('ERROR', () => {
  it('roundtrips error codes (3 bytes)', () => {
    const buf = encode({ type: MessageType.ERROR, code: ErrorCode.AUTH_FAILED });
    assert.equal(buf.byteLength, 3);
    const msg = roundtrip({ type: MessageType.ERROR, code: ErrorCode.AUTH_FAILED });
    assert.equal(msg.type, MessageType.ERROR);
    if (msg.type === MessageType.ERROR) {
      assert.equal(msg.code, ErrorCode.AUTH_FAILED);
    }
  });
});

// ─── DRAG ───────────────────────────────────────────────────────────────────

describe('Drag events', () => {
  it('DRAG_START roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.DRAG_START });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.DRAG_START });
    assert.equal(msg.type, MessageType.DRAG_START);
  });

  it('DRAG_END roundtrips (1 byte)', () => {
    const buf = encode({ type: MessageType.DRAG_END });
    assert.equal(buf.byteLength, 1);
    const msg = roundtrip({ type: MessageType.DRAG_END });
    assert.equal(msg.type, MessageType.DRAG_END);
  });
});

// ─── Malformed Packets ──────────────────────────────────────────────────────

describe('Malformed packets', () => {
  it('rejects empty packet', () => {
    const result = decode(new ArrayBuffer(0));
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'EMPTY_PACKET');
    }
  });

  it('rejects wrong protocol version', () => {
    const buf = new ArrayBuffer(5);
    const view = new DataView(buf);
    // Version 2, type MOVE
    view.setUint8(0, (2 << 4) | MessageType.MOVE);
    view.setInt16(1, 10, false);
    view.setInt16(3, 20, false);
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'VERSION_MISMATCH');
    }
  });

  it('rejects unknown message type', () => {
    const buf = new ArrayBuffer(1);
    new DataView(buf).setUint8(0, (PROTOCOL_VERSION << 4) | 0x0f); // 0x0f is not a valid type
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'UNKNOWN_TYPE');
    }
  });

  it('rejects truncated MOVE packet', () => {
    const buf = new ArrayBuffer(3); // needs 5, only 3
    new DataView(buf).setUint8(0, (PROTOCOL_VERSION << 4) | MessageType.MOVE);
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'PAYLOAD_TOO_SHORT');
    }
  });

  it('rejects truncated SESSION_START packet', () => {
    const buf = new ArrayBuffer(10); // needs 18, only 10
    new DataView(buf).setUint8(0, (PROTOCOL_VERSION << 4) | MessageType.SESSION_START);
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'PAYLOAD_TOO_SHORT');
    }
  });

  it('rejects DEVICE_INFO with invalid JSON', () => {
    const header = (PROTOCOL_VERSION << 4) | MessageType.DEVICE_INFO;
    const garbage = new TextEncoder().encode('not{json');
    const buf = new ArrayBuffer(3 + garbage.length);
    const view = new DataView(buf);
    const bytes = new Uint8Array(buf);
    view.setUint8(0, header);
    view.setUint16(1, garbage.length, false);
    bytes.set(garbage, 3);
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'INVALID_PAYLOAD');
    }
  });

  it('rejects DEVICE_INFO with valid JSON but wrong structure', () => {
    const header = (PROTOCOL_VERSION << 4) | MessageType.DEVICE_INFO;
    const payload = new TextEncoder().encode(JSON.stringify({ foo: 'bar' }));
    const buf = new ArrayBuffer(3 + payload.length);
    const view = new DataView(buf);
    const bytes = new Uint8Array(buf);
    view.setUint8(0, header);
    view.setUint16(1, payload.length, false);
    bytes.set(payload, 3);
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'INVALID_PAYLOAD');
    }
  });

  it('rejects DEVICE_INFO with invalid platform', () => {
    const header = (PROTOCOL_VERSION << 4) | MessageType.DEVICE_INFO;
    const payload = new TextEncoder().encode(
      JSON.stringify({ deviceName: 'Test', platform: 'linux', appVersion: '1.0.0' })
    );
    const buf = new ArrayBuffer(3 + payload.length);
    const view = new DataView(buf);
    const bytes = new Uint8Array(buf);
    view.setUint8(0, header);
    view.setUint16(1, payload.length, false);
    bytes.set(payload, 3);
    const result = decode(buf);
    assert.equal(result.ok, false);
    if (!result.ok) {
      assert.equal(result.error.code, 'INVALID_PAYLOAD');
    }
  });
});

// ─── Uint8Array input ───────────────────────────────────────────────────────

describe('Uint8Array input', () => {
  it('decodes from Uint8Array (not just ArrayBuffer)', () => {
    const buf = encode({ type: MessageType.MOVE, dx: 5, dy: -3 });
    const uint8 = new Uint8Array(buf);
    const result = decode(uint8);
    assert.equal(result.ok, true);
    if (result.ok && result.message.type === MessageType.MOVE) {
      assert.equal(result.message.dx, 5);
      assert.equal(result.message.dy, -3);
    }
  });

  it('decodes from Uint8Array slice (with byteOffset)', () => {
    // Simulate receiving data in a larger buffer with an offset
    const buf = encode({ type: MessageType.LEFT_DOWN });
    const padded = new Uint8Array(10);
    padded.set(new Uint8Array(buf), 3);
    const slice = padded.subarray(3, 3 + buf.byteLength);
    const result = decode(slice);
    assert.equal(result.ok, true);
    if (result.ok) {
      assert.equal(result.message.type, MessageType.LEFT_DOWN);
    }
  });
});

// ─── Protocol header encoding ───────────────────────────────────────────────

describe('Header encoding', () => {
  it('encodes version in upper nibble, type in lower nibble', () => {
    const buf = encode({ type: MessageType.MOVE, dx: 0, dy: 0 });
    const header = new Uint8Array(buf)[0]!;
    const version = (header >> 4) & 0x0f;
    const msgType = header & 0x0f;
    assert.equal(version, PROTOCOL_VERSION);
    assert.equal(msgType, MessageType.MOVE);
  });
});

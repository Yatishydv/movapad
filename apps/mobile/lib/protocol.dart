import 'dart:typed_data';

/// MovaPad Binary Protocol Constants
const int protocolVersion = 1;

class MessageType {
  static const int move = 0x01;
  static const int leftDown = 0x02;
  static const int leftUp = 0x03;
  static const int rightDown = 0x04;
  static const int rightUp = 0x05;
  static const int scroll = 0x06;
  static const int ping = 0x07;
  static const int pong = 0x08;
  static const int sessionStart = 0x09;
  static const int sessionEnd = 0x0a;
  static const int deviceInfo = 0x0b;
  static const int error = 0x0c;
  static const int dragStart = 0x0d;
  static const int dragEnd = 0x0e;
}

/// Helper to encode the header byte
int _encodeHeader(int msgType) {
  return ((protocolVersion & 0x0f) << 4) | (msgType & 0x0f);
}

/// Helper to clamp to i16
int _clampI16(int val) {
  if (val > 32767) return 32767;
  if (val < -32768) return -32768;
  return val;
}

/// Encodes a MOVE message [header, dx, dx, dy, dy]
Uint8List encodeMove(int dx, int dy) {
  final buf = ByteData(5);
  buf.setUint8(0, _encodeHeader(MessageType.move));
  buf.setInt16(1, _clampI16(dx), Endian.big);
  buf.setInt16(3, _clampI16(dy), Endian.big);
  return buf.buffer.asUint8List();
}

/// Encodes a SCROLL message [header, dx, dx, dy, dy]
Uint8List encodeScroll(int dx, int dy) {
  final buf = ByteData(5);
  buf.setUint8(0, _encodeHeader(MessageType.scroll));
  buf.setInt16(1, _clampI16(dx), Endian.big);
  buf.setInt16(3, _clampI16(dy), Endian.big);
  return buf.buffer.asUint8List();
}

/// Encodes a 1-byte message without payload
Uint8List _encodeEmpty(int type) {
  final buf = ByteData(1);
  buf.setUint8(0, _encodeHeader(type));
  return buf.buffer.asUint8List();
}

Uint8List encodeLeftDown() => _encodeEmpty(MessageType.leftDown);
Uint8List encodeLeftUp() => _encodeEmpty(MessageType.leftUp);
Uint8List encodeRightDown() => _encodeEmpty(MessageType.rightDown);
Uint8List encodeRightUp() => _encodeEmpty(MessageType.rightUp);
Uint8List encodeDragStart() => _encodeEmpty(MessageType.dragStart);
Uint8List encodeDragEnd() => _encodeEmpty(MessageType.dragEnd);
Uint8List encodeSessionEnd() => _encodeEmpty(MessageType.sessionEnd);

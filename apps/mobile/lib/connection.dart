
import 'dart:typed_data';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import 'protocol.dart';

enum AppConnectionState { disconnected, connecting, connected, error }

class ConnectionManager extends ChangeNotifier {
  WebSocketChannel? _channel;
  AppConnectionState _state = AppConnectionState.disconnected;
  String _errorMessage = '';

  AppConnectionState get state => _state;
  String get errorMessage => _errorMessage;

  void connect(String ipAddress) {
    if (_state == AppConnectionState.connected || _state == AppConnectionState.connecting) {
      return;
    }

    _updateState(AppConnectionState.connecting);

    Uri uri;
    if (ipAddress.startsWith('http://') || ipAddress.startsWith('https://')) {
      ipAddress = ipAddress.replaceFirst('http://', '').replaceFirst('https://', '');
    }
    
    if (ipAddress.contains('.loca.lt') || ipAddress.contains('ngrok')) {
      // It's a public tunnel, use wss:// and no port
      uri = Uri.parse('wss://$ipAddress');
    } else {
      // It's a local IP address, use ws://
      // If they didn't specify a port, add 8081
      if (!ipAddress.contains(':')) {
        uri = Uri.parse('ws://$ipAddress:8081');
      } else {
        uri = Uri.parse('ws://$ipAddress');
      }
    }
    try {
      _channel = WebSocketChannel.connect(uri);

      _channel!.stream.listen(
        (message) {
          // Handle incoming messages if needed later
        },
        onDone: () {
          _updateState(AppConnectionState.disconnected);
        },
        onError: (error) {
          debugPrint('WebSocket Error: $error');
          _errorMessage = error.toString();
          _updateState(AppConnectionState.error);
        },
      );

      _updateState(AppConnectionState.connected);
    } catch (e) {
      debugPrint('WebSocket Exception: $e');
      _errorMessage = e.toString();
      _updateState(AppConnectionState.error);
    }
  }

  void disconnect() {
    if (_state == AppConnectionState.connected) {
      _sendBytes(encodeSessionEnd());
      _channel?.sink.close();
      _channel = null;
      _updateState(AppConnectionState.disconnected);
    }
  }

  void _updateState(AppConnectionState newState) {
    _state = newState;
    notifyListeners();
  }

  void _sendBytes(Uint8List data) {
    if (_state == AppConnectionState.connected) {
      _channel?.sink.add(data);
    }
  }

  // --- Input methods ---

  void move(double dx, double dy) {
    _sendBytes(encodeMove(dx.round(), dy.round()));
  }

  void scroll(double dx, double dy) {
    _sendBytes(encodeScroll(dx.round(), dy.round()));
  }

  void leftDown() => _sendBytes(encodeLeftDown());
  void leftUp() => _sendBytes(encodeLeftUp());
  
  void rightDown() => _sendBytes(encodeRightDown());
  void rightUp() => _sendBytes(encodeRightUp());

  void dragStart() => _sendBytes(encodeDragStart());
  void dragEnd() => _sendBytes(encodeDragEnd());
}

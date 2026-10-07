import 'dart:convert';
import 'package:flutter_webrtc/flutter_webrtc.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

class WebRtcManager {
  RTCPeerConnection? _peerConnection;
  RTCDataChannel? _dataChannel;
  WebSocketChannel? _signaling;

  final Function(RTCDataChannel) onDataChannel;
  final Function(String) onSignalingError;

  WebRtcManager({
    required this.onDataChannel,
    required this.onSignalingError,
  });

  Future<void> connectToSignaling(String ipAddress, String pin) async {
    final uri = Uri.parse('ws://$ipAddress:8080');
    _signaling = WebSocketChannel.connect(uri);

    _signaling!.stream.listen((message) async {
      final data = jsonDecode(message);
      final type = data['type'];

      switch (type) {
        case 'ROOM_JOINED':
          // Wait for Desktop to send offer
          break;
        case 'PEER_CONNECTED':
          // Desktop got notified
          break;
        case 'offer':
          await _handleOffer(data);
          break;
        case 'candidate':
          await _handleCandidate(data);
          break;
        case 'ERROR':
          onSignalingError(data['message'] ?? 'Unknown signaling error');
          break;
        case 'PEER_DISCONNECTED':
          dispose();
          onSignalingError('Peer disconnected');
          break;
      }
    }, onError: (err) {
      onSignalingError('Signaling WS error: $err');
    });

    _signaling!.sink.add(jsonEncode({
      'type': 'JOIN_ROOM',
      'pin': pin,
    }));
  }

  Future<void> _createPeerConnection() async {
    final configuration = {
      'iceServers': [
        {'urls': 'stun:stun.l.google.com:19302'},
      ]
    };

    _peerConnection = await createPeerConnection(configuration);

    _peerConnection!.onIceCandidate = (RTCIceCandidate candidate) {
      _signaling?.sink.add(jsonEncode({
        'type': 'candidate',
        'candidate': candidate.candidate,
        'sdpMid': candidate.sdpMid,
        'sdpMLineIndex': candidate.sdpMLineIndex,
      }));
    };

    _peerConnection!.onDataChannel = (RTCDataChannel channel) {
      _dataChannel = channel;
      onDataChannel(channel);
    };
  }

  Future<void> _handleOffer(Map<String, dynamic> data) async {
    if (_peerConnection == null) {
      await _createPeerConnection();
    }

    final sdp = data['sdp'];
    await _peerConnection!.setRemoteDescription(
      RTCSessionDescription(sdp, 'offer'),
    );

    final answer = await _peerConnection!.createAnswer({});
    await _peerConnection!.setLocalDescription(answer);

    _signaling?.sink.add(jsonEncode({
      'type': 'answer',
      'sdp': answer.sdp,
    }));
  }

  Future<void> _handleCandidate(Map<String, dynamic> data) async {
    if (_peerConnection == null) return;
    
    final candidate = RTCIceCandidate(
      data['candidate'],
      data['sdpMid'],
      data['sdpMLineIndex'],
    );
    await _peerConnection!.addCandidate(candidate);
  }

  void dispose() {
    _signaling?.sink.close();
    _dataChannel?.close();
    _peerConnection?.close();
    
    _signaling = null;
    _dataChannel = null;
    _peerConnection = null;
  }
}

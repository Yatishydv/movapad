import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'connection.dart';
import 'scanner.dart';
import 'dart:ui';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setPreferredOrientations([DeviceOrientation.portraitUp]);
  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    statusBarColor: Colors.transparent,
    statusBarBrightness: Brightness.dark,
  ));
  runApp(const MovaPadApp());
}

class MovaPadApp extends StatelessWidget {
  const MovaPadApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'MovaPad',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: const Color(0xFF09090B),
        useMaterial3: true,
        fontFamily: 'SF Pro Display',
      ),
      home: const TouchpadScreen(),
    );
  }
}

class TouchpadScreen extends StatefulWidget {
  const TouchpadScreen({super.key});

  @override
  State<TouchpadScreen> createState() => _TouchpadScreenState();
}

class _TouchpadScreenState extends State<TouchpadScreen> with SingleTickerProviderStateMixin {
  final ConnectionManager _connection = ConnectionManager();
  final TextEditingController _ipController = TextEditingController(text: '127.0.0.1');
  
  late AnimationController _glowController;
  late Animation<double> _glowAnimation;
  Offset _lastLongPressOffset = Offset.zero;

  @override
  void initState() {
    super.initState();
    _connection.addListener(_onConnectionChanged);
    
    _glowController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 2),
    )..repeat(reverse: true);
    
    _glowAnimation = Tween<double>(begin: 0.2, end: 0.6).animate(
      CurvedAnimation(parent: _glowController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _connection.removeListener(_onConnectionChanged);
    _ipController.dispose();
    _glowController.dispose();
    super.dispose();
  }

  void _onConnectionChanged() {
    setState(() {});
    if (_connection.state == AppConnectionState.connected) {
      HapticFeedback.mediumImpact();
    }
  }

  void _connect() {
    final ip = _ipController.text.trim();
    if (ip.isNotEmpty) {
      HapticFeedback.selectionClick();
      _connection.connect(ip);
    }
  }

  void _disconnect() {
    HapticFeedback.selectionClick();
    _connection.disconnect();
  }

  @override
  Widget build(BuildContext context) {
    final isConnected = _connection.state == AppConnectionState.connected;

    return Scaffold(
      body: Stack(
        children: [
          // Background ambient gradient
          Positioned(
            top: -100,
            left: -100,
            child: AnimatedBuilder(
              animation: _glowAnimation,
              builder: (context, child) {
                return Container(
                  width: 300,
                  height: 300,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: isConnected 
                        ? const Color(0xFF3B82F6).withOpacity(_glowAnimation.value)
                        : const Color(0xFF27272A).withOpacity(0.5),
                  ),
                );
              },
            ),
          ),
          Positioned.fill(
            child: BackdropFilter(
              filter: ImageFilter.blur(sigmaX: 80, sigmaY: 80),
              child: const SizedBox(),
            ),
          ),

          SafeArea(
            child: CustomScrollView(
              physics: isConnected ? const NeverScrollableScrollPhysics() : null,
              slivers: [
                SliverToBoxAdapter(child: _buildHeader()),
                if (!isConnected) SliverToBoxAdapter(child: _buildConnectionPanel()),
                SliverFillRemaining(
                  hasScrollBody: false,
                  child: _buildTouchpad(isConnected),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24.0, vertical: 16.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Row(
            children: [
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: Colors.transparent,
                  border: Border.all(color: Colors.white24, width: 1.5),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: const Icon(Icons.video_label, color: Colors.white, size: 18),
              ),
              const SizedBox(width: 12),
              const Text(
                'MovaPad',
                style: TextStyle(
                  fontSize: 22,
                  fontWeight: FontWeight.w700,
                  letterSpacing: -0.5,
                  color: Colors.white,
                ),
              ),
            ],
          ),
          IconButton(
            icon: const Icon(Icons.settings_outlined, color: Colors.white70),
            onPressed: () {
              HapticFeedback.lightImpact();
              // Future: Settings screen
            },
          ),
        ],
      ),
    );
  }

  Widget _buildConnectionPanel() {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24.0, vertical: 8.0),
      child: Container(
        padding: const EdgeInsets.all(20),
        decoration: BoxDecoration(
          color: Colors.white.withOpacity(0.03),
          borderRadius: BorderRadius.circular(24),
          border: Border.all(color: Colors.white.withOpacity(0.05)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Connect to Mac',
              style: TextStyle(fontSize: 14, color: Colors.white54, fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: _ipController,
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
              decoration: InputDecoration(
                hintText: 'Enter IP or Tunnel URL',
                hintStyle: TextStyle(color: Colors.white.withOpacity(0.2)),
                filled: true,
                fillColor: Colors.black26,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(16),
                  borderSide: BorderSide.none,
                ),
                contentPadding: const EdgeInsets.symmetric(horizontal: 20, vertical: 18),
                prefixIcon: const Icon(Icons.wifi, color: Colors.white54),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.qr_code_scanner, color: Colors.blueAccent),
                  onPressed: () async {
                    final scannedCode = await Navigator.push(
                      context,
                      MaterialPageRoute(builder: (context) => const QRScannerScreen()),
                    );
                    if (scannedCode != null && scannedCode is String) {
                      _ipController.text = scannedCode;
                      _connect();
                    }
                  },
                ),
              ),
            ),
            if (_connection.state == AppConnectionState.error)
              Padding(
                padding: const EdgeInsets.only(top: 12.0),
                child: Text(
                  _connection.errorMessage,
                  style: const TextStyle(color: Color(0xFFEF4444), fontSize: 13),
                ),
              ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              height: 54,
              child: ElevatedButton(
                onPressed: _connection.state == AppConnectionState.connecting ? null : _connect,
                style: ElevatedButton.styleFrom(
                  backgroundColor: const Color(0xFF3B82F6),
                  foregroundColor: Colors.white,
                  elevation: 0,
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
                ),
                child: _connection.state == AppConnectionState.connecting
                    ? const SizedBox(width: 20, height: 20, child: CircularProgressIndicator(color: Colors.white, strokeWidth: 2))
                    : const Text('Connect', style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600)),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildTouchpad(bool isConnected) {
    return Container(
      margin: const EdgeInsets.fromLTRB(24, 16, 24, 32),
      decoration: BoxDecoration(
        color: isConnected ? Colors.white.withOpacity(0.02) : Colors.transparent,
        borderRadius: BorderRadius.circular(32),
        border: Border.all(
          color: isConnected ? Colors.white.withOpacity(0.08) : Colors.transparent,
          width: 1,
        ),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(32),
        child: isConnected
            ? GestureDetector(
                behavior: HitTestBehavior.opaque,
                // Advanced Gestures via onScaleUpdate to detect finger count
                onScaleUpdate: (details) {
                  if (details.pointerCount == 1) {
                    _connection.move(details.focalPointDelta.dx * 1.5, details.focalPointDelta.dy * 1.5);
                  } else if (details.pointerCount == 2) {
                    _connection.scroll(details.focalPointDelta.dx, details.focalPointDelta.dy);
                  }
                },
                // A quick tap acts as a click
                onTap: () {
                  HapticFeedback.selectionClick();
                  _connection.leftDown();
                  Future.delayed(const Duration(milliseconds: 50), () => _connection.leftUp());
                },
                
                // Long press to click and drag (e.g. text selection)
                onLongPressStart: (details) {
                  HapticFeedback.heavyImpact();
                  _lastLongPressOffset = Offset.zero;
                  _connection.leftDown();
                },
                onLongPressMoveUpdate: (details) {
                  final delta = details.localOffsetFromOrigin - _lastLongPressOffset;
                  _lastLongPressOffset = details.localOffsetFromOrigin;
                  _connection.move(delta.dx * 1.5, delta.dy * 1.5);
                },
                onLongPressEnd: (details) => _connection.leftUp(),
                onLongPressCancel: () => _connection.leftUp(),
                
                // Two finger tap -> Right click
                onSecondaryTap: () {
                  HapticFeedback.lightImpact();
                  _connection.rightDown();
                  Future.delayed(const Duration(milliseconds: 50), () => _connection.rightUp());
                },
                
                child: Stack(
                  children: [
                    Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(Icons.touch_app_rounded, size: 48, color: Colors.white.withOpacity(0.1)),
                          const SizedBox(height: 16),
                          const Text(
                            'Touchpad Active',
                            style: TextStyle(
                              color: Colors.white30,
                              fontSize: 16,
                              fontWeight: FontWeight.w600,
                              letterSpacing: 1.2,
                            ),
                          ),
                        ],
                      ),
                    ),
                    Positioned(
                      bottom: 24,
                      left: 0,
                      right: 0,
                      child: Center(
                        child: TextButton.icon(
                          onPressed: _disconnect,
                          icon: const Icon(Icons.link_off, size: 18),
                          label: const Text('Disconnect'),
                          style: TextButton.styleFrom(
                            foregroundColor: const Color(0xFFEF4444).withOpacity(0.8),
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              )
            : const SizedBox(),
      ),
    );
  }
}

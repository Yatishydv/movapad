import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:movapad_mobile/main.dart';

void main() {
  testWidgets('MovaPad app smoke test', (WidgetTester tester) async {
    await tester.pumpWidget(const MovaPadApp());
    expect(find.text('MovaPad (Phase 1)'), findsOneWidget);
  });
}

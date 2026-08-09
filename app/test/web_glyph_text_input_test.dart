import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/web_glyph_text_input.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('WebGlyphTextInput inserts text via hardware keyboard', (tester) async {
    final controller = EditorController.forTest();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Stack(
            children: [
              WebGlyphTextInput(controller: controller),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    controller.ensureGlyphCaret();
    controller.webGlyphFocusNode.requestFocus();
    await tester.pump();

    await tester.enterText(find.byType(TextField), 'hi');
    await tester.pumpAndSettle();

    expect(controller.documentText, 'hi');
  });

  testWidgets('WebGlyphTextInput keeps focus after document click refocus', (tester) async {
    final controller = EditorController.forTest();
    addTearDown(controller.dispose);

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Stack(
            children: [
              WebGlyphTextInput(controller: controller),
              const SizedBox.expand(key: Key('doc')),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('doc')));
    await tester.pumpAndSettle();

    controller.focusGlyphInput();
    await tester.pump();

    expect(controller.webGlyphFocusNode.hasFocus, isTrue);
  });
}

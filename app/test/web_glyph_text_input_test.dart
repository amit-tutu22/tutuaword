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

  testWidgets('WebGlyphTextInput inserts spaces via soft-keyboard field',
      (tester) async {
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

    await tester.enterText(find.byType(TextField), 'hi there');
    await tester.pumpAndSettle();

    expect(controller.documentText, 'hi there');
  });

  testWidgets('enterText hello newline does not duplicate the line', (tester) async {
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

    // Multiline field delivers Return as '\n' via onChanged (not onSubmitted).
    await tester.enterText(find.byType(TextField), 'hello\n');
    await tester.pumpAndSettle();

    expect(controller.documentText, 'hello\n');
    expect(controller.documentText, isNot(contains('hellohello')));
  });

  testWidgets('WebGlyphTextInput does not cover canvas pointer hits', (tester) async {
    final controller = EditorController.forTest();
    addTearDown(controller.dispose);
    var taps = 0;

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Stack(
            children: [
              Positioned.fill(
                child: GestureDetector(
                  key: const Key('canvas'),
                  behavior: HitTestBehavior.opaque,
                  onTap: () => taps++,
                  child: const ColoredBox(color: Colors.white),
                ),
              ),
              WebGlyphTextInput(controller: controller),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('canvas')));
    await tester.pump();

    expect(taps, 1);
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
              Positioned.fill(
                child: GestureDetector(
                  key: const Key('doc'),
                  behavior: HitTestBehavior.opaque,
                  onTap: () {},
                  child: const ColoredBox(color: Colors.transparent),
                ),
              ),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byKey(const Key('doc')), warnIfMissed: false);
    await tester.pumpAndSettle();

    controller.webGlyphFocusNode.requestFocus();
    await tester.pump();

    expect(controller.webGlyphFocusNode.hasFocus, isTrue);
  });
}

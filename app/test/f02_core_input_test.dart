import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S1 core input', () {
    test('I-F02-S1-typing-latency glyph inserts stay responsive', () async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);

      controller.ensureGlyphCaret();
      final samples = <int>[];
      for (var i = 0; i < 50; i++) {
        final sw = Stopwatch()..start();
        await controller.insertGlyphCharacter('a');
        samples.add(sw.elapsedMicroseconds);
      }

      samples.sort();
      final p99Index = (samples.length * 0.99).floor().clamp(0, samples.length - 1);
      final p99Us = samples[p99Index];

      // End-to-end Flutter+FFI budget is looser than the Rust layout gate.
      expect(p99Us, lessThan(50000), reason: 'p99 ${p99Us}us');
      expect(controller.documentText.toLowerCase(), contains('a'));
    });

    testWidgets('tab key inserts via glyph keyboard path', (tester) async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      controller.ensureGlyphCaret();
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await tester.sendKeyEvent(LogicalKeyboardKey.keyH);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      final beforeTab = controller.caretOffset;
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();

      expect(controller.caretOffset, greaterThan(beforeTab));
    });
  });
}

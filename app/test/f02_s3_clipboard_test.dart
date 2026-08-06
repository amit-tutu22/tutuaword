import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/paste_special_dialog.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S3 clipboard and paste special', () {
    testWidgets('I-F02-S3-paste-special-dialog plain vs formatted', (tester) async {
      final controller = EditorController(enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();

      const html = '<html><body><p><b>bold</b></p></body></html>';
      const payload = EditorClipboardPayload(
        plainText: 'bold',
        html: html,
      );

      await controller.pastePayload(payload, plainText: true);
      expect(controller.documentText.toLowerCase(), contains('bold'));
      expect(controller.bold, isFalse);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () => controller.showPasteSpecialDialog(context),
                child: const Text('Paste Special'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Paste Special'));
      await tester.pumpAndSettle();

      expect(find.text('Paste Special'), findsWidgets);
      expect(find.text('Keep Source Formatting'), findsOneWidget);
      expect(find.text('Unformatted Text'), findsOneWidget);

      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      await controller.pastePayload(payload, plainText: false);
      controller.ensureGlyphCaret();
      expect(controller.documentText.toLowerCase(), contains('bold'));
      expect(controller.bold, isTrue);
    });
  });
}

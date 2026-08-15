import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/paste_special_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F02.S3 clipboard and paste special', () {
    testWidgets('I-F02-S3-paste-special-dialog plain vs formatted', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      controller.ensureGlyphCaret();

      const html = '<html><body><p><b>bold</b></p></body></html>';
      const payload = EditorClipboardPayload(
        plainText: 'bold',
        html: html,
      );

      await controller.pastePayload(payload, plainText: true);
      await controller.ensureLayoutReady();
      expect(controller.documentText.toLowerCase(), contains('bold'));
      expect(controller.bold, isFalse);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () async {
                  final mode = await PasteSpecialDialog.show(
                    context,
                    hasFormattedContent: true,
                  );
                  if (mode == null) return;
                  await controller.pastePayload(
                    payload,
                    plainText: mode == PasteSpecialMode.plainText,
                  );
                },
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
      await controller.ensureLayoutReady();
      expect(controller.documentText.toLowerCase(), contains('bold'));
      expect(controller.bold, isFalse);
    });

    testWidgets('paste plain multiline keeps all lines without tofu controls', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      const wingding = '\uF035';
      const arrowPua = '\uF0E0';
      final payload = EditorClipboardPayload(
        plainText: 'Assess ${arrowPua} Migrate\nHyperSDK\nBank-grade HA/DR$wingding',
      );
      await controller.pastePayload(payload, plainText: true);
      await controller.ensureLayoutReady();

      expect(controller.documentText, contains('Assess'));
      expect(controller.documentText, contains('Migrate'));
      expect(controller.documentText, contains('HyperSDK'));
      expect(controller.documentText, contains('Bank-grade HA/DR'));
      expect(controller.documentText, isNot(contains(wingding)));
      expect(controller.documentText, isNot(contains(arrowPua)));
      expect(controller.documentText, contains('→'));
      expect(controller.documentText, contains('•'));
    });
  });
}

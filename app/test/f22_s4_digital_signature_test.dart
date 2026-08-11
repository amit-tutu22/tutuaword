import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/digital_signature_dialog.dart';

DocumentSessionStore _isolatedStore(String prefix) => DocumentSessionStore(
      root: Directory.systemTemp.createTempSync(prefix),
    );

void main() {
  group('F22.S4 digital signatures', () {
    test('I-F22-S4-sign-and-clear', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s4_sign_'),
        enableAutosave: false,
      );
      addTearDown(controller.dispose);

      expect(
        controller.sessionController.signDocument(
          name: 'Ada',
          email: 'ada@example.com',
        ),
        isTrue,
      );
      expect(
        controller.sessionController.fetchDigitalSignaturesJson(),
        contains('Ada'),
      );
      expect(
        controller.sessionController.verifyDigitalSignaturesJson(),
        contains('valid'),
      );

      expect(controller.sessionController.clearDigitalSignatures(), isTrue);
      expect(
        controller.sessionController.fetchDigitalSignaturesJson(),
        '[]',
      );
    });

    test('I-F22-S4-tampered-verification', () {
      final engine = MockDocumentEngine();
      engine.signDocument(name: 'Bob');
      engine.setSignaturesTamperedForTest(true);
      final json = engine.verifyDigitalSignatures();
      expect(json, contains('tampered'));
    });

    testWidgets('I-F22-S4-sign-dialog', (tester) async {
      DigitalSignatureSignRequest? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Scaffold(
                body: TextButton(
                  key: const Key('open_sign'),
                  onPressed: () async {
                    final value = await DigitalSignatureDialog.show(
                      context,
                      signatures: const [],
                      verifications: const [],
                    );
                    if (value is DigitalSignatureSignRequest) {
                      result = value;
                    }
                  },
                  child: const Text('Open'),
                ),
              );
            },
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_sign')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('digital_signature_dialog')), findsOneWidget);

      await tester.enterText(
        find.byKey(const Key('digital_signature_name')),
        'Ada Lovelace',
      );
      await tester.enterText(
        find.byKey(const Key('digital_signature_email')),
        'ada@example.com',
      );
      await tester.tap(find.byKey(const Key('digital_signature_sign')));
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      expect(result!.name, 'Ada Lovelace');
      expect(result!.email, 'ada@example.com');
    });
  });
}

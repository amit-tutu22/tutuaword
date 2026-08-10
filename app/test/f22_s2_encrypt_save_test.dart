import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/protect_password_dialog.dart';

DocumentSessionStore _isolatedStore(String prefix) => DocumentSessionStore(
      root: Directory.systemTemp.createTempSync(prefix),
    );

void main() {
  group('F22.S2 encrypt on save', () {
    test('I-F22-S2-protect-sets-password-and-encrypts-save', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s2_protect_'),
        enableAutosave: false,
      );
      addTearDown(controller.dispose);

      await controller.newDocument();
      expect(controller.encryptionPasswordSet, isFalse);

      expect(controller.sessionController.protectWithPassword('secret'), isTrue);
      expect(controller.encryptionPasswordSet, isTrue);
      expect(controller.statusText, contains('encrypted'));

      final saved = engine.saveDocumentAsBytes('docx');
      expect(saved, isNotNull);
      expect(
        DocumentReader.isPasswordProtectedDocx(saved!, path: 'out.docx'),
        isTrue,
      );
    });

    test('I-F22-S2-remove-password-plaintext-save', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s2_remove_'),
        enableAutosave: false,
      );
      addTearDown(controller.dispose);

      controller.sessionController.protectWithPassword('secret');
      expect(controller.encryptionPasswordSet, isTrue);

      expect(controller.sessionController.removePasswordProtection(), isTrue);
      expect(controller.encryptionPasswordSet, isFalse);

      final saved = engine.saveDocumentBytes();
      expect(saved, isNotNull);
      expect(
        DocumentReader.isPasswordProtectedDocx(saved!, path: 'out.docx'),
        isFalse,
      );
    });

    test('I-F22-S2-open-retains-encryption-for-resave', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s2_resave_'),
        enableAutosave: false,
        passwordPrompt: ({fileName, errorMessage}) async => 'secret',
      );
      addTearDown(controller.dispose);

      engine.setEncryptionPassword('secret');
      final encrypted = engine.saveDocumentBytes()!;
      expect(
        DocumentReader.isPasswordProtectedDocx(encrypted, path: 'locked.docx'),
        isTrue,
      );

      final dir = Directory.systemTemp.createTempSync('tutuaword_f22_s2_resave_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/locked.docx';
      await File(path).writeAsBytes(encrypted);

      await controller.openDocumentFromPath(path);
      expect(controller.encryptionPasswordSet, isTrue);

      final resaved = engine.saveDocumentBytes();
      expect(
        DocumentReader.isPasswordProtectedDocx(resaved!, path: 'resave.docx'),
        isTrue,
      );
    });

    testWidgets('I-F22-S2-protect-dialog-mismatch', (tester) async {
      String? result;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () async {
                  result = await ProtectPasswordDialog.show(context);
                },
                child: const Text('Protect'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Protect'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('protect_password_dialog')), findsOneWidget);

      await tester.enterText(find.byKey(const Key('protect_password_field')), 'one');
      await tester.enterText(
        find.byKey(const Key('protect_password_confirm_field')),
        'two',
      );
      await tester.tap(find.byKey(const Key('protect_password_ok')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('protect_password_error')), findsOneWidget);
      expect(result, isNull);
    });

    testWidgets('I-F22-S2-protect-dialog-returns-password', (tester) async {
      String? result;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () async {
                  result = await ProtectPasswordDialog.show(context);
                },
                child: const Text('Protect'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Protect'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byKey(const Key('protect_password_field')), 'alpha');
      await tester.enterText(
        find.byKey(const Key('protect_password_confirm_field')),
        'alpha',
      );
      await tester.tap(find.byKey(const Key('protect_password_ok')));
      await tester.pumpAndSettle();
      expect(result, 'alpha');
    });
  });
}

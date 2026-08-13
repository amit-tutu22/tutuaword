import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/password_dialog.dart';

Uint8List encryptedDocxBytes() {
  final archive = Archive()
    ..addFile(ArchiveFile('[Content_Types].xml', 32, '<?xml version="1.0"?><Types/>'.codeUnits))
    ..addFile(ArchiveFile('EncryptionInfo', 9, 'encrypted'.codeUnits))
    ..addFile(ArchiveFile('EncryptedPackage', 9, 'encrypted'.codeUnits));
  return ZipEncoder().encodeBytes(archive);
}

DocumentSessionStore _isolatedStore(String prefix) => DocumentSessionStore(
      root: Directory.systemTemp.createTempSync(prefix),
    );

void main() {
  group('F22.S1 password-protected open', () {
    test('U-F22-S1-detects-encrypted-zip-docx', () {
      expect(
        DocumentReader.isPasswordProtectedDocx(encryptedDocxBytes(), path: 'locked.docx'),
        isTrue,
      );
    });

    test('U-F22-S1-detects-encrypted-zip-case-insensitive-entries', () {
      final archive = Archive()
        ..addFile(
          ArchiveFile(
            '[content_types].xml',
            32,
            '<?xml version="1.0"?><Types/>'.codeUnits,
          ),
        )
        ..addFile(ArchiveFile('encryptioninfo', 9, 'encrypted'.codeUnits))
        ..addFile(ArchiveFile('encryptedpackage', 9, 'encrypted'.codeUnits));
      final bytes = ZipEncoder().encodeBytes(archive);
      expect(
        DocumentReader.isPasswordProtectedDocx(bytes, path: 'locked.docx'),
        isTrue,
      );
    });

    test('U-F22-S1-plain-docx-not-flagged-when-entry-casing-differs', () {
      final archive = Archive()
        ..addFile(
          ArchiveFile(
            '[content_types].xml',
            32,
            '<?xml version="1.0"?><Types/>'.codeUnits,
          ),
        )
        ..addFile(
          ArchiveFile(
            'Word/Document.xml',
            64,
            '<?xml version="1.0"?><w:document/>'.codeUnits,
          ),
        );
      final bytes = ZipEncoder().encodeBytes(archive);
      expect(
        DocumentReader.isPasswordProtectedDocx(bytes, path: 'plain.docx'),
        isFalse,
      );
    });

    test('U-F22-S1-detects-ole-encryption-markers', () {
      final bytes = Uint8List.fromList([
        0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1,
        ...('....EncryptionInfo....'.codeUnits),
      ]);
      expect(
        DocumentReader.isPasswordProtectedDocx(bytes, path: 'locked.docx'),
        isTrue,
      );
    });

    test('I-F22-S1-headless-open-requires-password', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s1_req_'),
        enableAutosave: false,
      );
      addTearDown(controller.dispose);

      final dir = Directory.systemTemp.createTempSync('tutuaword_f22_s1_req_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/locked.docx';
      await File(path).writeAsBytes(encryptedDocxBytes());

      await controller.openDocumentFromPath(path);
      expect(controller.statusText, contains('Password required'));
    });

    test('I-F22-S1-prompt-cancel-aborts-open', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s1_cancel_'),
        enableAutosave: false,
        passwordPrompt: ({fileName, errorMessage}) async => null,
      );
      addTearDown(controller.dispose);

      final dir = Directory.systemTemp.createTempSync('tutuaword_f22_s1_cancel_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/locked.docx';
      await File(path).writeAsBytes(encryptedDocxBytes());

      await controller.openDocumentFromPath(path);
      expect(controller.statusText, contains('Open cancelled'));
    });

    test('I-F22-S1-prompt-correct-password-opens', () async {
      var prompts = 0;
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s1_ok_'),
        enableAutosave: false,
        passwordPrompt: ({fileName, errorMessage}) async {
          prompts += 1;
          return 'secret';
        },
      );
      addTearDown(controller.dispose);

      final dir = Directory.systemTemp.createTempSync('tutuaword_f22_s1_ok_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/locked.docx';
      await File(path).writeAsBytes(encryptedDocxBytes());

      await controller.openDocumentFromPath(path);
      expect(prompts, 1);
      expect(controller.statusText, contains('Opened'));
      expect(controller.currentPath, path);
    });

    test('I-F22-S1-wrong-then-correct-password-reprompts', () async {
      final passwords = <String?>['wrong', 'secret'];
      var index = 0;
      final errors = <String?>[];
      final engine = MockDocumentEngine();
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s1_retry_'),
        enableAutosave: false,
        passwordPrompt: ({fileName, errorMessage}) async {
          errors.add(errorMessage);
          return passwords[index++];
        },
      );
      addTearDown(controller.dispose);

      final dir = Directory.systemTemp.createTempSync('tutuaword_f22_s1_retry_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/locked.docx';
      await File(path).writeAsBytes(encryptedDocxBytes());

      await controller.openDocumentFromPath(path);
      expect(index, 2);
      expect(errors[1], contains('Incorrect password'));
      expect(controller.statusText, contains('Opened'));
    });

    testWidgets('I-F22-S1-password-dialog-returns-value', (tester) async {
      String? result;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () async {
                  result = await PasswordDialog.show(
                    context,
                    fileName: 'locked.docx',
                  );
                },
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('password_dialog')), findsOneWidget);

      await tester.enterText(find.byKey(const Key('password_dialog_field')), 'hunter2');
      await tester.tap(find.byKey(const Key('password_dialog_ok')));
      await tester.pumpAndSettle();

      expect(result, 'hunter2');
    });
  });
}

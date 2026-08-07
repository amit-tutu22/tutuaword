import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';
import 'package:tutuaword/editor/autosave_scheduler.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/document_properties_dialog.dart';
import 'editor_test_helpers.dart';
import 'native_ffi_test_helpers.dart';

/// Pump [DocumentView] after open — async display-list decode never fully settles.
Future<void> pumpDocumentView(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 400));
}

DocumentSessionStore _isolatedStore(String prefix) {
  return DocumentSessionStore(root: Directory.systemTemp.createTempSync(prefix));
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  setUp(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
      const MethodChannel('tutuaword/macos_file_access'),
      (call) async {
        switch (call.method) {
          case 'createBookmark':
            return null;
          case 'startAccess':
            return true;
          default:
            return null;
        }
      },
    );
  });

  tearDown(() {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(
      const MethodChannel('tutuaword/macos_file_access'),
      null,
    );
  });

  Uint8List docxWithCoreProperties() {
    const documentXml = '''
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Props</w:t></w:r></w:p></w:body></w:document>''';
    const coreXml = '''
<cp:coreProperties xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:title>Quarterly Report</dc:title>
  <dc:creator>Jane Author</dc:creator>
</cp:coreProperties>''';
    const appXml =
        '<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Pages>3</Pages></Properties>';
    final archive = Archive()
      ..addFile(ArchiveFile('word/document.xml', documentXml.length, documentXml.codeUnits))
      ..addFile(ArchiveFile('docProps/core.xml', coreXml.length, coreXml.codeUnits))
      ..addFile(ArchiveFile('docProps/app.xml', appXml.length, appXml.codeUnits))
      ..addFile(ArchiveFile('[Content_Types].xml', 8, '<Types/>'.codeUnits))
      ..addFile(ArchiveFile('word/_rels/document.xml.rels', 16, '<Relationships/>'.codeUnits));
    return Uint8List.fromList(ZipEncoder().encode(archive)!);
  }

  Uint8List encryptedDocxBytes() {
    final archive = Archive()
      ..addFile(ArchiveFile('[Content_Types].xml', 32, '<?xml version="1.0"?><Types/>'.codeUnits))
      ..addFile(ArchiveFile('EncryptionInfo', 9, 'encrypted'.codeUnits))
      ..addFile(ArchiveFile('EncryptedPackage', 9, 'encrypted'.codeUnits));
    return Uint8List.fromList(ZipEncoder().encode(archive)!);
  }

  group('F01.S1 document lifecycle', () {
    test('newDocument clears path and text', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'X');
      expect(controller.documentText, isNotEmpty);

      await controller.newDocument();

      expect(controller.documentText, isEmpty);
      expect(controller.documentTitle, 'Document1');
    });

    test('I-F01-S1-open-docx loads corpus fixture', () async {
      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s1_'),
      );
      final controller = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      final corpus = File('../crates/tw-docx/tests/corpus/simple_paragraph.docx');
      if (!corpus.existsSync()) return;

      await controller.openDocumentFromPath(corpus.path);

      expect(controller.displayListForPage(0), isNotEmpty);
    });

    test('I-F01-S1-save-as-docx roundtrip via temp file', () async {
      if (!await nativeFfiEventsAvailable()) return;

      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s1_'),
      );
      final controller = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      if (!controller.isEngineConnected) {
        controller.dispose();
        return;
      }

      await controller.newDocument();
      controller.ensureGlyphCaret();
      await typeTextDirect(controller, 'RT');

      final dir = Directory.systemTemp.createTempSync('tutuaword_f01_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final outPath = '${dir.path}/roundtrip.docx';

      expect(await controller.saveDocumentToPath(outPath, formatExtension: 'docx'), isTrue);
      expect(File(outPath).existsSync(), isTrue);

      await controller.newDocument();
      await controller.openDocumentFromPath(outPath);
      await controller.ensureLayoutReady();

      expect(controller.documentText.toLowerCase(), contains('rt'));
      await controller.ensureLayoutReady();
      controller.dispose();
    });
  });

  group('F01.S2 autosave and recent files', () {
    test('U-F01-S2-autosave-serializes draft bytes', () async {
      final root = Directory.systemTemp.createTempSync('tutuaword_f01_s2_');
      final store = DocumentSessionStore(root: root);
      final bytes = TwdocWriter.fromText('Autosaved draft');
      final savedAt = DateTime.utc(2026, 1, 2, 3, 4, 5);

      await store.writeAutosave(
        bytes: bytes,
        sourcePath: '/tmp/example.twdoc',
        savedAt: savedAt,
      );

      final snapshot = await store.readAutosave();
      expect(snapshot, isNotNull);
      expect(snapshot!.bytes, bytes);
      expect(snapshot.sourcePath, '/tmp/example.twdoc');
      expect(snapshot.savedAt, savedAt);
    });

    test('recent paths dedupe and cap at ten', () async {
      final root = Directory.systemTemp.createTempSync('tutuaword_f01_s2_recent_');
      final store = DocumentSessionStore(root: root);
      var recent = <RecentDocumentEntry>[];
      for (var i = 0; i < 12; i++) {
        final file = File('${root.path}/doc$i.twdoc');
        await file.writeAsString('x');
        recent = store.bumpRecentEntry(
          recent,
          RecentDocumentEntry(path: file.path, bookmark: 'bookmark-$i'),
        );
      }
      expect(recent.length, DocumentSessionStore.maxRecentFiles);
      expect(recent.first.path, '${root.path}/doc11.twdoc');
      expect(recent.first.bookmark, 'bookmark-11');
      await store.saveRecentEntries(recent);
      final loaded = store.loadRecentEntries();
      expect(loaded.map((entry) => entry.path), recent.map((entry) => entry.path));
      expect(loaded.first.bookmark, 'bookmark-11');
    });

    test('recent entries preserve bookmark when path is bumped again', () {
      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s2_recent_bookmark_'),
      );
      const original = RecentDocumentEntry(path: '/tmp/report.docx', bookmark: 'abc');
      final bumped = store.bumpRecentEntry(const [], original);
      final again = store.bumpRecentEntry(
        bumped,
        const RecentDocumentEntry(path: '/tmp/report.docx'),
      );
      expect(again.first.bookmark, 'abc');
    });

    test('autosave interval is configurable', () async {
      final root = Directory.systemTemp.createTempSync('tutuaword_f01_s2_settings_');
      final store = DocumentSessionStore(root: root);
      await store.saveAutosaveInterval(const Duration(seconds: 15));
      expect(store.loadAutosaveInterval(), const Duration(seconds: 15));
    });

    test('I-F01-S2-crash-recover restores unsaved draft', () async {
      if (!await nativeFfiEventsAvailable()) return;

      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s2_recover_'),
      );
      final first = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      if (!first.isEngineConnected) {
        first.dispose();
        return;
      }

      await first.newDocument();
      first.ensureGlyphCaret();
      for (final ch in 'DRAFT'.split('')) {
        await first.insertGlyphCharacter(ch);
      }
      await first.ensureLayoutReady();
      await first.performAutosave();
      first.dispose();

      final second = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      addTearDown(second.dispose);

      expect(await second.tryRecoverAutosave(), isTrue);
      expect(second.documentText.toUpperCase(), contains('DRAFT'));
      expect(second.statusText, contains('Recovered'));
    });

    test('performAutosave skips when document is clean', () async {
      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s2_skip_'),
      );
      final controller = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await controller.performAutosave();
      expect(await store.readAutosave(), isNull);
    });

    test('AutosaveScheduler manual tick invokes callback', () async {
      var ticks = 0;
      final scheduler = AutosaveScheduler(
        interval: const Duration(hours: 1),
        onTick: () async {
          ticks++;
        },
      );
      await scheduler.tickNow();
      expect(ticks, 1);
      scheduler.stop();
    });
  });

  group('F01.S3 export and print preview', () {
    test('I-F01-S3-export-pdf-menu saves structural pdf bytes', () async {
      if (!await nativeFfiEventsAvailable()) return;

      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s3_'),
      );
      final controller = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await controller.newDocument();
      controller.ensureGlyphCaret();
      await controller.insertGlyphCharacter('P');
      await controller.insertGlyphCharacter('D');
      await controller.insertGlyphCharacter('F');
      await controller.ensureLayoutReady();

      final dir = Directory.systemTemp.createTempSync('tutuaword_f01_s3_pdf_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final outPath = '${dir.path}/export.pdf';

      expect(await controller.exportPdfToPath(outPath), isTrue);
      final bytes = File(outPath).readAsBytesSync();
      expect(bytes.length, greaterThan(4));
      expect(String.fromCharCodes(bytes.sublist(0, 4)), '%PDF');
      expect(controller.statusText, contains('PDF exported'));
    });

    testWidgets('I-F01-S3-print-preview-toggle disables page editing', (tester) async {
      final store = _isolatedStore('tutuaword_f01_s3_preview_');
      final controller = EditorController(sessionStore: store, enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DocumentView(controller: controller),
          ),
        ),
      );
      await pumpDocumentView(tester);

      expect(controller.printPreview, isFalse);
      expect(controller.isPageEditable(0), isTrue);
      expect(find.byType(GlyphEditorSurface), findsWidgets);

      controller.togglePrintPreview();
      await tester.pump();

      expect(controller.printPreview, isTrue);
      expect(controller.isPageEditable(0), isFalse);
      expect(controller.statusText, contains('Print preview'));
      expect(find.byType(GlyphEditorSurface), findsNothing);

      controller.togglePrintPreview();
      await tester.pump();

      expect(controller.printPreview, isFalse);
      expect(find.byType(GlyphEditorSurface), findsWidgets);
    });
  });

  group('F01.S4 protection and properties', () {
    test('DocumentReader rejects password-protected docx', () {
      expect(
        () => DocumentReader.extractText(encryptedDocxBytes(), path: 'locked.docx'),
        throwsA(isA<FormatException>()),
      );
    });

    test('I-F01-S4-properties-dialog shows core metadata fields', () async {
      final store = DocumentSessionStore(
        root: Directory.systemTemp.createTempSync('tutuaword_f01_s4_'),
      );
      final controller = EditorController(
        sessionStore: store,
        enableAutosave: false,
      );
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      final dir = Directory.systemTemp.createTempSync('tutuaword_f01_s4_props_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/props.docx';
      await File(path).writeAsBytes(docxWithCoreProperties());

      await controller.openDocumentFromPath(path);

      expect(controller.documentProperties.title, 'Quarterly Report');
      expect(controller.documentProperties.author, 'Jane Author');
      expect(controller.documentProperties.pageCount, 3);
    });

    testWidgets('I-F01-S4-properties-dialog renders title and author', (tester) async {
      const properties = DocumentProperties(
        title: 'Quarterly Report',
        author: 'Jane Author',
        pageCount: 3,
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () => DocumentPropertiesDialog.show(context, properties),
                child: const Text('Open Properties'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Open Properties'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 300));

      expect(find.text('Document Properties'), findsOneWidget);
      expect(find.text('Quarterly Report'), findsOneWidget);
      expect(find.text('Jane Author'), findsOneWidget);
    });

    test('read-only open sets protection flags', () async {
      final store = _isolatedStore('tutuaword_f01_s4_ro_open_');
      final controller = EditorController(sessionStore: store, enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      const documentXml = '''
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Locked</w:t></w:r></w:p></w:body></w:document>''';
      const settingsXml = '''
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:documentProtection w:edit="readOnly" w:enforcement="1"/>
</w:settings>''';
      final archive = Archive()
        ..addFile(ArchiveFile('word/document.xml', documentXml.length, documentXml.codeUnits))
        ..addFile(ArchiveFile('word/settings.xml', settingsXml.length, settingsXml.codeUnits))
        ..addFile(ArchiveFile('[Content_Types].xml', 8, '<Types/>'.codeUnits))
        ..addFile(ArchiveFile('word/_rels/document.xml.rels', 16, '<Relationships/>'.codeUnits));
      final bytes = Uint8List.fromList(ZipEncoder().encode(archive)!);

      final dir = Directory.systemTemp.createTempSync('tutuaword_f01_s4_ro_open_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/readonly.docx';
      await File(path).writeAsBytes(bytes);
      await controller.openDocumentFromPath(path);

      expect(controller.documentReadOnly, isTrue);
      expect(controller.isPageEditable(0), isFalse);
      expect(controller.statusText, contains('read-only'));
    });

    testWidgets('read-only document disables glyph editing', (tester) async {
      final store = _isolatedStore('tutuaword_f01_s4_ro_');
      final controller = EditorController(sessionStore: store, enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      await controller.newDocument();
      controller.setDocumentReadOnlyForTest(true);
      expect(controller.isPageEditable(0), isFalse);

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await pumpDocumentView(tester);
      expect(find.byType(GlyphEditorSurface), findsNothing);
    });

    test('password-protected open shows clear status message', () async {
      final store = _isolatedStore('tutuaword_f01_s4_pw_');
      final controller = EditorController(sessionStore: store, enableAutosave: false);
      addTearDown(controller.dispose);
      if (!controller.isEngineConnected) return;

      final dir = Directory.systemTemp.createTempSync('tutuaword_f01_s4_pw_');
      addTearDown(() {
        if (dir.existsSync()) dir.deleteSync(recursive: true);
      });
      final path = '${dir.path}/locked.docx';
      await File(path).writeAsBytes(encryptedDocxBytes());

      await controller.openDocumentFromPath(path);

      expect(controller.statusText, contains('Password-protected'));
    });
  });
}

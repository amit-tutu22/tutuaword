import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';
import 'package:tutuaword/editor/editor_controller.dart';

Uint8List _minimalDocx(String documentXml) {
  final archive = Archive()
    ..addFile(ArchiveFile('word/document.xml', documentXml.length, documentXml.codeUnits))
    ..addFile(ArchiveFile('[Content_Types].xml', 8, '<Types/>'.codeUnits))
    ..addFile(ArchiveFile('word/_rels/document.xml.rels', 16, '<Relationships/>'.codeUnits));
  return ZipEncoder().encodeBytes(archive);
}

Uint8List _minimalOdt(String contentXml) {
  final archive = Archive()
    ..addFile(ArchiveFile('content.xml', contentXml.length, contentXml.codeUnits))
    ..addFile(ArchiveFile('mimetype', 39, 'application/vnd.oasis.opendocument.text'.codeUnits));
  return ZipEncoder().encodeBytes(archive);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('DocumentReader Phase 3 formats', () {
    test('reads minimal DOCX text', () {
      const xml = '''
<w:document><w:body>
  <w:p><w:r><w:t>Hello DOCX</w:t></w:r></w:p>
</w:body></w:document>''';
      final text = DocumentReader.extractText(_minimalDocx(xml), path: 'sample.docx');
      expect(text, contains('Hello DOCX'));
    });

    test('reads minimal ODT text', () {
      const xml = '''
<office:document><office:body>
  <text:p><text:span>Hello ODT</text:span></text:p>
</office:body></office:document>''';
      final text = DocumentReader.extractText(_minimalOdt(xml), path: 'sample.odt');
      expect(text, contains('Hello ODT'));
    });

    test('reads HTML text', () {
      final html = Uint8List.fromList(
        utf8.encode('<html><body><h1>Title</h1><p>Body</p></body></html>'),
      );
      final text = DocumentReader.extractText(html, path: 'page.html');
      expect(text, contains('Title'));
      expect(text, contains('Body'));
    });

    test('reads Markdown as plain text fallback', () {
      final md = Uint8List.fromList(utf8.encode('# Heading\n\nParagraph'));
      final text = DocumentReader.extractText(md, path: 'readme.md');
      expect(text, contains('# Heading'));
      expect(text, contains('Paragraph'));
    });
  });

  group('EditorController Phase 3 (mock mode)', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController.forTest();
    });

    tearDown(() {
      controller.dispose();
    });

    test('spellCheckDocument updates status in mock mode', () async {
      await controller.spellCheckDocument();
      if (!controller.isEngineConnected) {
        expect(controller.statusText, contains('Spell check'));
      } else {
        expect(controller.statusText, anyOf(contains('Spell check'), contains('No spelling')));
      }
      expect(controller.spellMisspellings, isEmpty);
    });

    test('toggleTrackChanges updates status text', () {
      controller.toggleTrackChanges();
      expect(controller.trackChanges, isTrue);
      expect(controller.statusText, contains('Track changes on'));
      controller.toggleTrackChanges();
      expect(controller.trackChanges, isFalse);
    });

    test('replacePageText preserves spell check state container', () {
      controller.replacePageText(0, 'Teh misspelled wrd');
      expect(controller.documentText, contains('Teh'));
    });

    test('setDisplayListForTest switches to glyph rendering mode', () {
      final bytes = Uint8List.fromList([2, 0, 0, 0]); // too short -> no glyphs
      controller.setDisplayListForTest(bytes, preferTextRendering: false);
      expect(controller.preferTextRendering, isFalse);
    });
  });

  group('TwdocWriter cross-format fallback', () {
    test('native writer still round-trips after format helpers exist', () {
      final bytes = TwdocWriter.fromText('Phase 3 fallback');
      expect(TwdocReader.extractText(bytes), 'Phase 3 fallback');
    });
  });
}

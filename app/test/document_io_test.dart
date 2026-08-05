import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';

Uint8List _minimalDocx(String documentXml) {
  final archive = Archive()
    ..addFile(ArchiveFile('word/document.xml', documentXml.length, documentXml.codeUnits))
    ..addFile(ArchiveFile('[Content_Types].xml', 8, '<Types/>'.codeUnits));
  return Uint8List.fromList(ZipEncoder().encode(archive)!);
}

void main() {
  test('reads plain text files', () {
    final text = DocumentReader.extractText(
      Uint8List.fromList('Hello\nWorld'.codeUnits),
      path: 'notes.txt',
    );
    expect(text, 'Hello\nWorld');
  });

  test('detects zip format without extension using content', () {
    final docx = _minimalDocx('<w:document><w:body><w:p><w:r><w:t>Zip</w:t></w:r></w:p></w:body></w:document>');
    final text = DocumentReader.extractText(docx);
    expect(text, contains('Zip'));
  });

  test('reads markdown file by extension', () {
    final text = DocumentReader.extractText(
      Uint8List.fromList('# Title'.codeUnits),
      path: 'notes.md',
    );
    expect(text, '# Title');
  });

  test('reads twdoc files', () {
    final bytes = TwdocWriter.fromText('Native format');
    expect(
      DocumentReader.extractText(bytes, path: 'doc.twdoc'),
      'Native format',
    );
  });

  test('rejects legacy doc files', () {
    expect(
      () => DocumentReader.extractText(Uint8List(8), path: 'old.doc'),
      throwsFormatException,
    );
  });

  test('sample fixture opens if present', () {
    final fixture = File('../fixtures/sample.twdoc');
    if (!fixture.existsSync()) return;
    final text = DocumentReader.extractText(fixture.readAsBytesSync(), path: fixture.path);
    expect(text, isNotEmpty);
  });
}

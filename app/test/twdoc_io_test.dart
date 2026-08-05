import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';

void main() {
  test('TwdocWriter round-trip extracts text', () {
    const text = 'Hello from tutuaword';
    final bytes = TwdocWriter.fromText(text);
    expect(TwdocReader.extractText(bytes), text);
  });

  test('sample fixture opens if present', () {
    final fixture = File('../fixtures/sample.twdoc');
    if (!fixture.existsSync()) return;
    final text = TwdocReader.extractText(fixture.readAsBytesSync());
    expect(text, isNotEmpty);
  });
}

import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';

void main() {
  group('TwdocWriter integration', () {
    test('round-trip preserves multi-line text', () {
      const source = 'Heading\n\nBody paragraph\nSecond line';
      final bytes = TwdocWriter.fromText(source);
      expect(TwdocReader.extractText(bytes), source);
    });

    test('written bytes are non-empty and start with ZIP signature', () {
      final bytes = TwdocWriter.fromText('ZIP container test');
      expect(bytes, isNotEmpty);
      expect(bytes[0], 0x50);
      expect(bytes[1], 0x4B);
    });

    test('empty document produces valid container', () {
      final bytes = TwdocWriter.fromText('');
      expect(bytes, isNotEmpty);
      expect(TwdocReader.extractText(bytes), isEmpty);
    });
  });
}

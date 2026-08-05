import 'dart:io';

import 'package:tutuaword/bridge/twdoc_io.dart';

void main() {
  final bytes = TwdocWriter.fromText(
    'Welcome to tutuaword. Open this file to verify Open works.',
  );
  File('../fixtures/sample.twdoc').writeAsBytesSync(bytes);
  stdout.writeln('Created sample.twdoc (${bytes.length} bytes)');
}

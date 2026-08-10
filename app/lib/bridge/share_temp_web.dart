import 'dart:typed_data';

/// Web cannot hand a local path to the share sheet; callers share text instead.
Future<String?> writeShareTempFile({
  required String fileName,
  required Uint8List bytes,
}) async =>
    null;

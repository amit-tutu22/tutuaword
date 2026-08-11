import 'dart:io';
import 'dart:typed_data';

import 'package:path/path.dart' as p;

/// Writes [bytes] into a unique temp file for the OS share sheet.
Future<String> writeShareTempFile({
  required String fileName,
  required Uint8List bytes,
}) async {
  final dir = await Directory.systemTemp.createTemp('tutuaword_share_');
  final safe = fileName.replaceAll(RegExp(r'[^\w.\- ]+'), '_').trim();
  final name = safe.isEmpty ? 'document.docx' : safe;
  final file = File(p.join(dir.path, name));
  await file.writeAsBytes(bytes, flush: true);
  return file.path;
}

import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:tutuaword/bridge/document_io.dart';

/// Result of a cross-platform file pick (bytes always loaded).
class PickedDocumentFile {
  const PickedDocumentFile({
    required this.bytes,
    required this.path,
    required this.name,
  });

  final Uint8List bytes;

  /// Logical path for format detection / recent files.
  /// On web this is the display [name] (blob URLs have no extension).
  final String path;
  final String name;
}

/// Pick a single file and load its bytes.
///
/// Chrome: [FilePicker]'s default `cancelUploadOnWindowBlur` races the file
/// input `change` event when the dialog closes, so Open appears to cancel.
/// We disable that on all platforms for single-file picks.
Future<PickedDocumentFile?> pickDocumentFile({
  String dialogTitle = 'Open document',
  List<String> allowedExtensions = kSupportedOpenExtensions,
  FileType type = FileType.custom,
}) async {
  final result = await FilePicker.pickFiles(
    dialogTitle: dialogTitle,
    type: type,
    allowedExtensions: type == FileType.custom ? allowedExtensions : null,
    allowMultiple: false,
    withData: true,
    cancelUploadOnWindowBlur: false,
  );
  if (result == null || result.files.isEmpty) return null;

  final file = result.files.single;
  final bytes = await file.readAsBytes();
  if (bytes.isEmpty) {
    throw StateError('Selected file is empty');
  }

  final path = kIsWeb ? file.name : (file.path ?? file.name);
  return PickedDocumentFile(bytes: bytes, path: path, name: file.name);
}

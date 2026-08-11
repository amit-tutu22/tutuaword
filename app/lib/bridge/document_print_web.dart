import 'dart:typed_data';

import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/file_bytes.dart';

/// Browser print host — downloads the print PDF (F25.S1).
class WebDocumentPrintHost implements DocumentPrintHost {
  @override
  Future<PrintDialogResult> presentPrintDialog({
    required Uint8List pdfBytes,
    String jobName = 'Document',
    Map<String, dynamic> attributes = const {},
  }) async {
    final _ = attributes;
    if (!isPdfHeader(pdfBytes)) {
      return const PrintDialogResult(
        outcome: PrintDialogOutcome.failed,
        message: 'Print failed: invalid PDF',
      );
    }
    final filename =
        jobName.toLowerCase().endsWith('.pdf') ? jobName : '$jobName.pdf';
    await downloadBytes(filename: filename, bytes: pdfBytes);
    return PrintDialogResult(
      outcome: PrintDialogOutcome.fallbackSaved,
      message: 'Downloaded $filename for printing',
    );
  }
}

DocumentPrintHost createPlatformPrintHost() => WebDocumentPrintHost();

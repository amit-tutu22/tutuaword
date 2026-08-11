import 'dart:typed_data';

/// Outcome of presenting the OS print dialog (F25.S1).
enum PrintDialogOutcome {
  /// Platform print UI was shown (user may still cancel inside it).
  presented,
  /// User cancelled before/without printing.
  cancelled,
  /// Channel/plugin unavailable; PDF written to a temp path instead.
  fallbackSaved,
  /// Print could not run.
  failed,
  /// Host does not support printing (e.g. some web embeds).
  unsupported,
}

class PrintDialogResult {
  const PrintDialogResult({
    required this.outcome,
    this.message,
    this.fallbackPath,
  });

  final PrintDialogOutcome outcome;
  final String? message;
  final String? fallbackPath;

  bool get ok =>
      outcome == PrintDialogOutcome.presented ||
      outcome == PrintDialogOutcome.fallbackSaved;
}

/// Platform (or test) host that presents a print dialog for PDF bytes.
abstract class DocumentPrintHost {
  Future<PrintDialogResult> presentPrintDialog({
    required Uint8List pdfBytes,
    String jobName = 'Document',
    Map<String, dynamic> attributes = const {},
  });
}

/// Records print requests for unit/integration tests.
class RecordingPrintHost implements DocumentPrintHost {
  final List<({Uint8List bytes, String jobName, Map<String, dynamic> attributes})>
      calls = [];
  PrintDialogResult nextResult = const PrintDialogResult(
    outcome: PrintDialogOutcome.presented,
  );

  @override
  Future<PrintDialogResult> presentPrintDialog({
    required Uint8List pdfBytes,
    String jobName = 'Document',
    Map<String, dynamic> attributes = const {},
  }) async {
    calls.add((
      bytes: Uint8List.fromList(pdfBytes),
      jobName: jobName,
      attributes: Map<String, dynamic>.from(attributes),
    ));
    return nextResult;
  }
}

/// True when [bytes] look like a PDF header (print pipeline gate).
bool isPdfHeader(Uint8List bytes) {
  if (bytes.length < 4) return false;
  return bytes[0] == 0x25 && // %
      bytes[1] == 0x50 && // P
      bytes[2] == 0x44 && // D
      bytes[3] == 0x46; // F
}

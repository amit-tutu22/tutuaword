/// Outcome of presenting the OS share sheet (device apps: Mail, Messages, etc.).
enum DocumentShareOutcome {
  /// System share UI was presented (or share completed).
  presented,
  /// User dismissed without sharing.
  cancelled,
  /// Share could not run.
  failed,
  /// Host does not support sharing on this platform.
  unsupported,
}

class DocumentShareResult {
  const DocumentShareResult({
    required this.outcome,
    this.message,
  });

  final DocumentShareOutcome outcome;
  final String? message;

  bool get ok => outcome == DocumentShareOutcome.presented;
}

/// Platform (or test) host that opens the native share sheet.
abstract class DocumentShareHost {
  Future<DocumentShareResult> share({
    required String text,
    String? subject,
    String? filePath,
    String? mimeType,
  });
}

/// Records share requests for unit/integration tests.
class RecordingShareHost implements DocumentShareHost {
  final List<
      ({
        String text,
        String? subject,
        String? filePath,
        String? mimeType,
      })> calls = [];

  DocumentShareResult nextResult = const DocumentShareResult(
    outcome: DocumentShareOutcome.presented,
  );

  @override
  Future<DocumentShareResult> share({
    required String text,
    String? subject,
    String? filePath,
    String? mimeType,
  }) async {
    calls.add((
      text: text,
      subject: subject,
      filePath: filePath,
      mimeType: mimeType,
    ));
    return nextResult;
  }
}

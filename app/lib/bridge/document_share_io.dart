import 'package:share_plus/share_plus.dart';
import 'package:tutuaword/bridge/document_share.dart';

/// Desktop / mobile share via the OS share sheet (`share_plus`).
class IoDocumentShareHost implements DocumentShareHost {
  @override
  Future<DocumentShareResult> share({
    required String text,
    String? subject,
    String? filePath,
    String? mimeType,
  }) async {
    try {
      final ShareResult result;
      if (filePath != null && filePath.isNotEmpty) {
        result = await SharePlus.instance.share(
          ShareParams(
            files: [
              XFile(
                filePath,
                mimeType: mimeType,
                name: filePath.split(RegExp(r'[/\\]')).last,
              ),
            ],
            text: text.isEmpty ? null : text,
            subject: subject,
            title: subject,
          ),
        );
      } else {
        if (text.isEmpty) {
          return const DocumentShareResult(
            outcome: DocumentShareOutcome.failed,
            message: 'Nothing to share',
          );
        }
        result = await SharePlus.instance.share(
          ShareParams(
            text: text,
            subject: subject,
            title: subject,
          ),
        );
      }
      return switch (result.status) {
        ShareResultStatus.success => const DocumentShareResult(
            outcome: DocumentShareOutcome.presented,
          ),
        ShareResultStatus.dismissed => const DocumentShareResult(
            outcome: DocumentShareOutcome.cancelled,
          ),
        ShareResultStatus.unavailable => const DocumentShareResult(
            outcome: DocumentShareOutcome.unsupported,
            message: 'Share unavailable on this device',
          ),
      };
    } catch (e) {
      return DocumentShareResult(
        outcome: DocumentShareOutcome.failed,
        message: e.toString(),
      );
    }
  }
}

DocumentShareHost createPlatformShareHost() => IoDocumentShareHost();

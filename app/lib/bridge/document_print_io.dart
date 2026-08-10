import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/document_print.dart';

/// Desktop print host via MethodChannel + temp-file fallback (F25.S1).
class IoDocumentPrintHost implements DocumentPrintHost {
  IoDocumentPrintHost({MethodChannel? channel})
      : _channel = channel ?? const MethodChannel('tutuaword/print');

  final MethodChannel _channel;
  static bool _pluginMissing = false;

  @override
  Future<PrintDialogResult> presentPrintDialog({
    required Uint8List pdfBytes,
    String jobName = 'Document',
    Map<String, dynamic> attributes = const {},
  }) async {
    if (!isPdfHeader(pdfBytes)) {
      return const PrintDialogResult(
        outcome: PrintDialogOutcome.failed,
        message: 'Print failed: invalid PDF',
      );
    }

    if (!_pluginMissing) {
      try {
        final outcome = await _channel.invokeMethod<String>(
          'presentPrintDialog',
          {
            'pdfBase64': base64Encode(pdfBytes),
            'jobName': jobName,
            'attributes': attributes,
          },
        );
        switch (outcome) {
          case 'presented':
            return const PrintDialogResult(outcome: PrintDialogOutcome.presented);
          case 'cancelled':
            return const PrintDialogResult(outcome: PrintDialogOutcome.cancelled);
          case 'unsupported':
            return const PrintDialogResult(
              outcome: PrintDialogOutcome.unsupported,
            );
          default:
            break;
        }
      } on MissingPluginException {
        _pluginMissing = true;
      } on PlatformException catch (e) {
        return PrintDialogResult(
          outcome: PrintDialogOutcome.failed,
          message: e.message ?? 'Print failed',
        );
      }
    }

    try {
      final dir = Directory.systemTemp.createTempSync('tutuaword_print_');
      final path = p.join(dir.path, '${_safeFileStem(jobName)}.pdf');
      await File(path).writeAsBytes(pdfBytes, flush: true);
      return PrintDialogResult(
        outcome: PrintDialogOutcome.fallbackSaved,
        fallbackPath: path,
        message: 'Print dialog unavailable — saved $path',
      );
    } catch (e) {
      return PrintDialogResult(
        outcome: PrintDialogOutcome.failed,
        message: 'Print failed: $e',
      );
    }
  }

  static String _safeFileStem(String jobName) {
    final stem = jobName
        .toLowerCase()
        .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
        .replaceAll(RegExp(r'^-+|-+$'), '');
    return stem.isEmpty ? 'document' : stem;
  }
}

DocumentPrintHost createPlatformPrintHost() => IoDocumentPrintHost();

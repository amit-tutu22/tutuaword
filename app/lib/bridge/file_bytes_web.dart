import 'dart:html' as html;
import 'dart:typed_data';

Future<void> writeBytesToPath(String path, Uint8List bytes) async {
  await downloadBytes(filename: path, bytes: bytes);
}

Future<void> downloadBytes({
  required String filename,
  required Uint8List bytes,
}) async {
  final blob = html.Blob([bytes]);
  final url = html.Url.createObjectUrlFromBlob(blob);
  final anchor = html.AnchorElement(href: url)
    ..setAttribute('download', filename)
    ..style.display = 'none';
  html.document.body?.append(anchor);
  anchor.click();
  anchor.remove();
  // Revoke only after the browser has had time to start the download.
  await Future<void>.delayed(const Duration(seconds: 1));
  html.Url.revokeObjectUrl(url);
}

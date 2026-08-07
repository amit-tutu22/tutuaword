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
    ..click();
  html.Url.revokeObjectUrl(url);
  anchor.remove();
}

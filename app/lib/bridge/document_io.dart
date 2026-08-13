import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/twdoc_io.dart';

/// Microsoft Word-compatible extensions for the Open dialog.
const kSupportedOpenExtensions = [
  'twdoc',
  'docx',
  'doc',
  'rtf',
  'txt',
  'odt',
  'html',
  'htm',
  'md',
  'markdown',
];

/// Extract readable text from any supported document format (Dart fallback).
class DocumentReader {
  static bool isPasswordProtectedDocx(Uint8List bytes, {String? path}) {
    final ext = _extensionFromPath(path);
    if (ext != null && ext != 'docx') return false;
    // Encrypted OOXML is often an OLE/CFB container (not a ZIP).
    // CFB directory entries store stream names as UTF-16LE.
    if (_looksLikeOle(bytes)) {
      return _containsOleStreamName(bytes, 'EncryptionInfo') ||
          _containsOleStreamName(bytes, 'EncryptedPackage');
    }
    if (!_looksLikeZip(bytes)) return false;
    try {
      final archive = ZipDecoder().decodeBytes(bytes);
      final hasEncrypted = _findZipFile(archive, 'EncryptedPackage') != null ||
          _findZipFile(archive, 'EncryptionInfo') != null;
      final hasDocument = _findZipFile(archive, 'word/document.xml') != null;
      final hasContentTypes =
          _findZipFile(archive, '[Content_Types].xml') != null;
      return hasEncrypted || (hasContentTypes && !hasDocument);
    } catch (_) {
      return false;
    }
  }

  static String extractText(Uint8List bytes, {String? path}) {
    if (isPasswordProtectedDocx(bytes, path: path)) {
      throw const FormatException('document is password-protected');
    }
    final ext = _extensionFromPath(path);
    if (ext != null) {
      switch (ext) {
        case 'twdoc':
          return TwdocReader.extractText(bytes);
        case 'docx':
          if (_zipContains(bytes, 'content.json')) {
            return TwdocReader.extractText(bytes);
          }
          return _extractDocxText(bytes);
        case 'odt':
          return _extractOdtText(bytes);
        case 'rtf':
          return _extractRtfText(bytes);
        case 'html':
        case 'htm':
          return _extractHtmlText(bytes);
        case 'txt':
        case 'md':
        case 'markdown':
          return utf8.decode(bytes);
        case 'doc':
          throw const FormatException(
            'Legacy .doc (Word 97-2003) format is not supported yet',
          );
      }
    }

    if (_looksLikeZip(bytes)) {
      if (_zipContains(bytes, 'content.json')) {
        return TwdocReader.extractText(bytes);
      }
      if (_zipContains(bytes, 'word/document.xml')) {
        return _extractDocxText(bytes);
      }
      if (_zipContains(bytes, 'content.xml')) {
        return _extractOdtText(bytes);
      }
    }
    if (_looksLikeRtf(bytes)) {
      return _extractRtfText(bytes);
    }
    if (_looksLikeHtml(bytes)) {
      return _extractHtmlText(bytes);
    }
    return utf8.decode(bytes);
  }

  static String? _extensionFromPath(String? path) {
    if (path == null) return null;
    final ext = p.extension(path);
    if (ext.isEmpty) return null;
    return ext.substring(1).toLowerCase();
  }

  static bool _looksLikeZip(Uint8List bytes) =>
      bytes.length >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B;

  static bool _looksLikeOle(Uint8List bytes) =>
      bytes.length >= 8 &&
      bytes[0] == 0xD0 &&
      bytes[1] == 0xCF &&
      bytes[2] == 0x11 &&
      bytes[3] == 0xE0 &&
      bytes[4] == 0xA1 &&
      bytes[5] == 0xB1 &&
      bytes[6] == 0x1A &&
      bytes[7] == 0xE1;

  static bool _containsAsciiCi(Uint8List haystack, String needle) {
    if (needle.isEmpty || haystack.length < needle.length) return false;
    final lowerNeedle = needle.toLowerCase().codeUnits;
    final n = lowerNeedle.length;
    for (var i = 0; i <= haystack.length - n; i++) {
      var match = true;
      for (var j = 0; j < n; j++) {
        final b = haystack[i + j];
        final c = b >= 65 && b <= 90 ? b + 32 : b;
        if (c != lowerNeedle[j]) {
          match = false;
          break;
        }
      }
      if (match) return true;
    }
    return false;
  }

  static bool _containsOleStreamName(Uint8List haystack, String name) {
    if (_containsAsciiCi(haystack, name)) return true;
    final utf16 = <int>[];
    for (final unit in name.codeUnits) {
      utf16.add(unit & 0xff);
      utf16.add((unit >> 8) & 0xff);
    }
    final n = utf16.length;
    if (haystack.length < n) return false;
    for (var i = 0; i <= haystack.length - n; i++) {
      var match = true;
      for (var j = 0; j < n; j++) {
        final a = haystack[i + j];
        final b = utf16[j];
        // ASCII letters only appear in the low byte of UTF-16LE names.
        final aa = (j.isEven && a >= 65 && a <= 90) ? a + 32 : a;
        final bb = (j.isEven && b >= 65 && b <= 90) ? b + 32 : b;
        if (aa != bb) {
          match = false;
          break;
        }
      }
      if (match) return true;
    }
    return false;
  }

  static bool _looksLikeRtf(Uint8List bytes) {
    final prefix = utf8.decode(bytes.take(8).toList(), allowMalformed: true);
    return prefix.trimLeft().startsWith('{\\rtf');
  }

  static bool _looksLikeHtml(Uint8List bytes) {
    final prefix =
        utf8.decode(bytes.take(256).toList(), allowMalformed: true).toLowerCase();
    return prefix.contains('<html') ||
        prefix.contains('<!doctype') ||
        prefix.trimLeft().startsWith('<');
  }

  static bool _zipContains(Uint8List bytes, String name) {
    try {
      final archive = ZipDecoder().decodeBytes(bytes);
      return _findZipFile(archive, name) != null;
    } catch (_) {
      return false;
    }
  }

  static ArchiveFile? _findZipFile(Archive archive, String normalizedPath) {
    final target =
        normalizedPath.replaceAll('\\', '/').replaceFirst(RegExp(r'^/+'), '').toLowerCase();
    for (final file in archive.files) {
      final entry =
          file.name.replaceAll('\\', '/').replaceFirst(RegExp(r'^/+'), '').toLowerCase();
      if (entry == target) return file;
    }
    return null;
  }

  static String _extractDocxText(Uint8List bytes) {
    final archive = ZipDecoder().decodeBytes(bytes);
    final content = _findZipFile(archive, 'word/document.xml');
    if (content == null) {
      throw const FormatException('word/document.xml missing from .docx');
    }
    final xml = utf8.decode(content.readBytes() ?? const <int>[]);
    return _extractOoxmlParagraphs(xml, 'w:p', 'w:t');
  }

  static String _extractOdtText(Uint8List bytes) {
    final archive = ZipDecoder().decodeBytes(bytes);
    final content = _findZipFile(archive, 'content.xml');
    if (content == null) {
      throw const FormatException('content.xml missing from .odt');
    }
    final xml = utf8.decode(content.readBytes() ?? const <int>[]);
    return _extractOoxmlParagraphs(xml, 'text:p', 'text:span');
  }

  static String _extractOoxmlParagraphs(
    String xml,
    String paragraphTag,
    String textTag,
  ) {
    final paragraphs = <String>[];
    for (final chunk in xml.split('<$paragraphTag').skip(1)) {
      final end = chunk.indexOf('</$paragraphTag>');
      final body = end == -1 ? chunk : chunk.substring(0, end);
      final buffer = StringBuffer();
      _collectTagText(body, textTag, buffer);
      paragraphs.add(_decodeXmlEntities(buffer.toString().trim()));
    }
    return paragraphs.join('\n\n');
  }

  static void _collectTagText(String xml, String tag, StringBuffer out) {
    final open = '<$tag';
    final close = '</$tag>';
    var rest = xml;
    while (true) {
      final start = rest.indexOf(open);
      if (start == -1) break;
      final afterOpen = rest.substring(start);
      final gt = afterOpen.indexOf('>');
      if (gt == -1) break;
      final contentStart = start + gt + 1;
      final relEnd = rest.indexOf(close, contentStart);
      if (relEnd == -1) break;
      out.write(rest.substring(contentStart, relEnd));
      rest = rest.substring(relEnd);
    }
  }

  static String _decodeXmlEntities(String text) => text
      .replaceAll('&lt;', '<')
      .replaceAll('&gt;', '>')
      .replaceAll('&amp;', '&')
      .replaceAll('&quot;', '"')
      .replaceAll('&apos;', "'");

  static String _extractRtfText(Uint8List bytes) {
    final rtf = utf8.decode(bytes, allowMalformed: true);
    if (!rtf.trimLeft().startsWith('{\\rtf')) {
      throw const FormatException('Not a valid RTF document');
    }
    final out = StringBuffer();
    final chars = rtf.split('');
    var i = 0;
    while (i < chars.length) {
      final ch = chars[i];
      if (ch == '\\') {
        i++;
        if (i >= chars.length) break;
        final next = chars[i];
        if (next == 'p' && i + 1 < chars.length && chars[i + 1] == 'a') {
          while (i < chars.length && chars[i] != ' ') i++;
          out.write('\n');
        } else if (next == 't' && i + 1 < chars.length && chars[i + 1] == 'a') {
          while (i < chars.length && chars[i] != ' ') i++;
          out.write('\t');
        } else if (next == "'") {
          final hex = chars.sublist(i + 1, i + 3).join();
          i += 2;
          out.writeCharCode(int.parse(hex, radix: 16));
        } else {
          while (i < chars.length && RegExp(r'[a-zA-Z]').hasMatch(chars[i])) {
            i++;
          }
          while (i < chars.length && RegExp(r'[0-9-]').hasMatch(chars[i])) {
            i++;
          }
          if (i < chars.length && chars[i] == ' ') i++;
        }
      } else if (ch == '{' || ch == '}') {
        // skip group markers
      } else if (ch.trim().isNotEmpty || out.isNotEmpty) {
        out.write(ch);
      }
      i++;
    }
    return out.toString().trim();
  }

  static String _extractHtmlText(Uint8List bytes) {
    final html = utf8.decode(bytes, allowMalformed: true);
    final out = StringBuffer();
    var inTag = false;
    for (final ch in html.split('')) {
      if (ch == '<') {
        inTag = true;
      } else if (ch == '>') {
        inTag = false;
        out.write('\n');
      } else if (!inTag) {
        out.write(ch);
      }
    }
    return _decodeXmlEntities(out.toString().trim());
  }
}

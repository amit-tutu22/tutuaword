import 'dart:convert';
import 'dart:typed_data';

import 'package:archive/archive.dart';

/// Read plain text from a `.twdoc` (ZIP + content.json) file without Rust.
class TwdocReader {
  static String extractText(Uint8List bytes) {
    try {
      final archive = ZipDecoder().decodeBytes(bytes);
      final content = archive.findFile('content.json');
      if (content == null) {
        throw const FormatException('content.json missing from .twdoc');
      }
      final jsonStr = utf8.decode(content.content as List<int>);
      final doc = jsonDecode(jsonStr) as Map<String, dynamic>;
      final sections = doc['sections'] as List<dynamic>? ?? [];
      final paragraphs = <String>[];
      for (final section in sections) {
        final blocks = (section as Map<String, dynamic>)['blocks'] as List<dynamic>? ?? [];
        for (final block in blocks) {
          final map = block as Map<String, dynamic>;
          if (map.containsKey('Paragraph')) {
            paragraphs.add(_paragraphText(map['Paragraph'] as Map<String, dynamic>));
          }
        }
      }
      return paragraphs.join('\n');
    } catch (e) {
      throw FormatException('Failed to read .twdoc: $e');
    }
  }

  static String _paragraphText(Map<String, dynamic> para) {
    final runs = para['runs'] as List<dynamic>? ?? [];
    final buffer = StringBuffer();
    for (final run in runs) {
      final content = (run as Map<String, dynamic>)['content'] as Map<String, dynamic>?;
      if (content != null && content.containsKey('Text')) {
        buffer.write(content['Text']);
      }
    }
    return buffer.toString();
  }
}

/// Build a minimal `.twdoc` from plain text (mock save path).
class TwdocWriter {
  static Uint8List fromText(String text) {
    final doc = {
      'id': '00000000-0000-0000-0000-000000000001',
      'styles': {'defaults': {}, 'paragraph_styles': {}, 'character_styles': {}},
      'settings': {
        'track_changes_enabled': false,
        'default_tab_stop': 36.0,
        'numbering': {'definitions': {}},
      },
      'sections': [
        {
          'id': '00000000-0000-0000-0000-000000000002',
          'format': {
            'page_width': 612.0,
            'page_height': 792.0,
            'margin_top': 72.0,
            'margin_bottom': 72.0,
            'margin_left': 72.0,
            'margin_right': 72.0,
          },
          'blocks': [
            {
              'Paragraph': {
                'id': '00000000-0000-0000-0000-000000000003',
                'format': {},
                'style_id': null,
                'runs': [
                  {
                    'id': '00000000-0000-0000-0000-000000000004',
                    'format': {},
                    'content': {'Text': text},
                  },
                ],
              },
            },
          ],
        },
      ],
    };

    final archive = Archive()
      ..addFile(ArchiveFile('manifest.json', 0, utf8.encode('{}')))
      ..addFile(
        ArchiveFile(
          'content.json',
          0,
          utf8.encode(const JsonEncoder.withIndent('  ').convert(doc)),
        ),
      )
      ..addFile(ArchiveFile('styles.json', 0, utf8.encode('{}')))
      ..addFile(ArchiveFile('settings.json', 0, utf8.encode('{}')));

    return Uint8List.fromList(ZipEncoder().encode(archive)!);
  }
}

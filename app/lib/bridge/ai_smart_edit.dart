import 'dart:convert';

import 'package:tutuaword/bridge/ai_client.dart';

/// Smart editing plan (F28.S6) — headings + TOC draft; applied only on Accept.

class AiHeadingSuggestion {
  const AiHeadingSuggestion({
    required this.paragraphIndex,
    required this.styleName,
    required this.previewText,
    this.reason = '',
  });

  final int paragraphIndex;
  final String styleName;
  final String previewText;
  final String reason;
}

class AiSmartEditPlan {
  AiSmartEditPlan({
    this.headings = const [],
    this.insertToc = false,
    this.autoFormatNotes = const [],
  });

  List<AiHeadingSuggestion> headings;
  bool insertToc;
  List<String> autoFormatNotes;

  bool get isEmpty => headings.isEmpty && !insertToc;

  static const promptInstruction =
      'Analyze the numbered paragraphs and suggest heading styles. '
      'Reply with ONLY JSON: '
      '{"headings":[{"index":0,"style":"Heading 1"}],"insert_toc":true}';
}

class _Para {
  _Para(this.index, this.text);
  final int index;
  final String text;
}

List<_Para> _bodyParagraphs(String documentText) {
  final lines = documentText.split('\n');
  final out = <_Para>[];
  for (var i = 0; i < lines.length; i++) {
    final t = lines[i];
    if (t.trim().isEmpty) continue;
    out.add(_Para(i, t));
  }
  return out;
}

String? _looksLikeHeading(String text) {
  final t = text.trim();
  if (t.isEmpty || t.length > 80) return null;
  if (t.endsWith('.') && t.length > 40) return null;
  final words = t.split(RegExp(r'\s+')).where((w) => w.isNotEmpty).toList();
  if (words.isEmpty) return null;
  final letters = t.replaceAll(RegExp(r'[^A-Za-z]'), '');
  final allCaps =
      letters.isNotEmpty && letters == letters.toUpperCase();
  final titleCase = words
          .where((w) => w.isNotEmpty && w[0] == w[0].toUpperCase())
          .length >=
      (words.length + 1) ~/ 2;
  final short = t.length <= 48;
  if (allCaps && short) return 'Heading 1';
  if (t.endsWith(':') && short) return 'Heading 2';
  if (short && (titleCase || !t.contains('.'))) {
    if (t.length <= 28 && titleCase) return 'Heading 1';
    return 'Heading 2';
  }
  return null;
}

/// Deterministic heuristic plan (no network).
AiSmartEditPlan analyzeDocumentHeuristics(String documentText) {
  final paras = _bodyParagraphs(documentText);
  final headings = <AiHeadingSuggestion>[];
  final notes = <String>[];
  var emptyStreak = 0;
  var messy = 0;
  for (final line in documentText.split('\n')) {
    if (line.trim().isEmpty) {
      emptyStreak++;
      if (emptyStreak >= 2) {
        notes.add('Multiple consecutive blank paragraphs detected');
        emptyStreak = 0;
      }
    } else {
      emptyStreak = 0;
    }
    if (line.contains('  ') || line.contains('\t')) messy++;
  }
  if (messy > 0) {
    notes.add('$messy paragraph(s) have irregular spacing');
  }

  var sawH1 = false;
  for (final para in paras) {
    final suggested = _looksLikeHeading(para.text);
    if (suggested == null) continue;
    var style = suggested;
    if (suggested == 'Heading 1') {
      if (sawH1) {
        style = 'Heading 2';
      } else {
        sawH1 = true;
      }
    }
    headings.add(AiHeadingSuggestion(
      paragraphIndex: para.index,
      styleName: style,
      previewText: para.text.length > 60
          ? para.text.substring(0, 60)
          : para.text,
      reason: 'heuristic: short title-like paragraph',
    ));
  }

  return AiSmartEditPlan(
    headings: headings,
    insertToc: headings.isNotEmpty,
    autoFormatNotes: notes.toSet().toList(),
  );
}

AiSmartEditPlan parseSmartEditPlan(String text, String documentText) {
  final plan = analyzeDocumentHeuristics(documentText);
  final start = text.indexOf('{');
  final end = text.lastIndexOf('}');
  if (start < 0 || end <= start) return plan;
  final map =
      jsonDecode(text.substring(start, end + 1)) as Map<String, dynamic>;
  final paras = _bodyParagraphs(documentText);
  final raw = map['headings'];
  if (raw is List && raw.isNotEmpty) {
    final ai = <AiHeadingSuggestion>[];
    for (final item in raw) {
      if (item is! Map) continue;
      final index = (item['index'] as num?)?.toInt() ?? -1;
      var style = item['style'] as String? ?? 'Heading 1';
      if (!style.startsWith('Heading')) style = 'Heading 1';
      _Para? para;
      for (final p in paras) {
        if (p.index == index) {
          para = p;
          break;
        }
      }
      if (para == null) continue;
      ai.add(AiHeadingSuggestion(
        paragraphIndex: para.index,
        styleName: style,
        previewText: para.text.length > 60
            ? para.text.substring(0, 60)
            : para.text,
        reason: 'ai suggestion',
      ));
    }
    if (ai.isNotEmpty) plan.headings = ai;
  }
  if (map['insert_toc'] is bool) {
    plan.insertToc = map['insert_toc'] as bool;
  }
  return plan;
}

Future<AiSmartEditPlan> suggestSmartEdit({
  required AiClient client,
  required String documentText,
}) async {
  final paras = _bodyParagraphs(documentText);
  final listing = paras
      .map((p) =>
          '${p.index}: ${p.text.length > 80 ? p.text.substring(0, 80) : p.text}')
      .join('\n');
  try {
    final text = await client.generate(
      const AiDocumentContext(pageCount: 1, totalTokenEstimate: 200),
      '${AiSmartEditPlan.promptInstruction}\nParagraphs:\n$listing',
    );
    return parseSmartEditPlan(text, documentText);
  } catch (_) {
    return analyzeDocumentHeuristics(documentText);
  }
}

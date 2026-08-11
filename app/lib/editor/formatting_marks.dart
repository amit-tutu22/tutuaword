import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_engine.dart';

enum FormattingMarkKind { space, tab, paragraph }

class FormattingMark {
  const FormattingMark({
    required this.kind,
    required this.x,
    required this.y,
    required this.height,
  });

  final FormattingMarkKind kind;
  final double x;
  final double y;
  final double height;
}

/// Collect space / tab / ¶ markers for [pageIndex] using caret geometry.
List<FormattingMark> collectFormattingMarks({
  required DocumentEngine engine,
  required int pageIndex,
  required String runId,
  required String text,
}) {
  if (text.isEmpty) return const [];
  final marks = <FormattingMark>[];

  void addAt(int offset, FormattingMarkKind kind) {
    final caret = engine.caretAtPosition(pageIndex, runId, offset);
    if (caret == null) return;
    marks.add(
      FormattingMark(
        kind: kind,
        x: caret.x,
        y: caret.y,
        height: caret.height,
      ),
    );
  }

  for (var i = 0; i < text.length; i++) {
    final ch = text[i];
    if (ch == ' ') {
      addAt(i, FormattingMarkKind.space);
    } else if (ch == '\t') {
      addAt(i, FormattingMarkKind.tab);
    } else if (ch == '\n' || ch == '\r') {
      addAt(i, FormattingMarkKind.paragraph);
    }
  }

  // Trailing paragraph mark at end of run (Word always shows ¶ at para end).
  if (text.isNotEmpty && text[text.length - 1] != '\n') {
    addAt(text.length, FormattingMarkKind.paragraph);
  }

  return marks;
}

/// Paints non-printing characters (spaces ·, tabs →, paragraphs ¶).
class FormattingMarksPainter extends CustomPainter {
  FormattingMarksPainter({required this.marks});

  final List<FormattingMark> marks;

  static const _color = Color(0xFF6B8EAE);

  @override
  void paint(Canvas canvas, Size size) {
    if (marks.isEmpty) return;
    final textPainter = TextPainter(
      textDirection: ui.TextDirection.ltr,
      textAlign: TextAlign.left,
    );

    for (final mark in marks) {
      switch (mark.kind) {
        case FormattingMarkKind.space:
          final cy = mark.y - mark.height * 0.35;
          canvas.drawCircle(
            Offset(mark.x + 1.5, cy),
            1.15,
            Paint()..color = _color,
          );
        case FormattingMarkKind.tab:
          final y = mark.y - mark.height * 0.45;
          final path = Path()
            ..moveTo(mark.x, y)
            ..lineTo(mark.x + 8, y)
            ..moveTo(mark.x + 5, y - 2.5)
            ..lineTo(mark.x + 8, y)
            ..lineTo(mark.x + 5, y + 2.5);
          canvas.drawPath(
            path,
            Paint()
              ..color = _color
              ..style = PaintingStyle.stroke
              ..strokeWidth = 1.0
              ..strokeCap = StrokeCap.round
              ..strokeJoin = StrokeJoin.round,
          );
        case FormattingMarkKind.paragraph:
          textPainter.text = TextSpan(
            text: '¶',
            style: TextStyle(
              color: _color,
              fontSize: (mark.height * 0.7).clamp(8.0, 14.0),
              height: 1.0,
            ),
          );
          textPainter.layout();
          textPainter.paint(
            canvas,
            Offset(mark.x + 1, mark.y - mark.height + 1),
          );
      }
    }
  }

  @override
  bool shouldRepaint(covariant FormattingMarksPainter oldDelegate) =>
      !identical(oldDelegate.marks, marks);
}

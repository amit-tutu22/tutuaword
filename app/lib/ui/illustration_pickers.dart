import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Word-style Insert → SmartArt gallery with layout previews.
class SmartArtPicker extends StatelessWidget {
  const SmartArtPicker({super.key, required this.onSelected});

  final ValueChanged<String> onSelected;

  static const _items = <({String id, String label, _SmartArtGlyph glyph})>[
    (id: 'process', label: 'Process', glyph: _SmartArtGlyph.process),
    (id: 'hierarchy', label: 'Hierarchy', glyph: _SmartArtGlyph.hierarchy),
    (id: 'cycle', label: 'Cycle', glyph: _SmartArtGlyph.cycle),
  ];

  static Future<String?> show(BuildContext anchorContext) {
    return _showAnchored<String>(
      anchorContext,
      width: 280,
      builder: (pop) => SmartArtPicker(onSelected: pop),
    );
  }

  @override
  Widget build(BuildContext context) {
    return _GalleryShell(
      title: 'Choose a SmartArt Graphic',
      subtitle: 'List / Process',
      children: [
        for (final item in _items)
          _PreviewTile(
            key: Key('smartart_picker_${item.id}'),
            label: item.label,
            onTap: () => onSelected(item.id),
            painter: _SmartArtPreviewPainter(item.glyph),
          ),
      ],
    );
  }
}

/// Word-style Insert → Chart gallery (All Charts → recommended types).
class ChartPicker extends StatelessWidget {
  const ChartPicker({super.key, required this.onSelected});

  final ValueChanged<int> onSelected;

  static const _items = <({int type, String label, _ChartGlyph glyph})>[
    (
      type: EditorController.chartColumn,
      label: 'Clustered Column',
      glyph: _ChartGlyph.column,
    ),
    (
      type: EditorController.chartBar,
      label: 'Clustered Bar',
      glyph: _ChartGlyph.bar,
    ),
    (
      type: EditorController.chartLine,
      label: 'Line',
      glyph: _ChartGlyph.line,
    ),
    (
      type: EditorController.chartPie,
      label: 'Pie',
      glyph: _ChartGlyph.pie,
    ),
  ];

  static Future<int?> show(BuildContext anchorContext) {
    return _showAnchored<int>(
      anchorContext,
      width: 360,
      builder: (pop) => ChartPicker(onSelected: pop),
    );
  }

  @override
  Widget build(BuildContext context) {
    return _GalleryShell(
      title: 'Insert Chart',
      subtitle: 'Recommended Charts',
      children: [
        for (final item in _items)
          _PreviewTile(
            key: Key('chart_picker_${item.type}'),
            label: item.label,
            wide: true,
            onTap: () => onSelected(item.type),
            painter: _ChartPreviewPainter(item.glyph),
          ),
      ],
    );
  }
}

Future<T?> _showAnchored<T>(
  BuildContext anchorContext, {
  required double width,
  required Widget Function(ValueChanged<T> pop) builder,
}) async {
  final box = anchorContext.findRenderObject() as RenderBox?;
  if (box == null) return null;
  final overlay =
      Overlay.of(anchorContext).context.findRenderObject() as RenderBox;
  final origin = box.localToGlobal(Offset.zero, ancestor: overlay);
  final size = box.size;

  return showGeneralDialog<T>(
    context: anchorContext,
    barrierDismissible: true,
    barrierLabel: 'Dismiss',
    barrierColor: Colors.black26,
    pageBuilder: (context, animation, secondaryAnimation) {
      return Stack(
        children: [
          Positioned(
            left: origin.dx.clamp(8.0, overlay.size.width - width),
            top: (origin.dy + size.height + 2)
                .clamp(8.0, overlay.size.height - 280),
            child: Material(
              elevation: 8,
              color: Colors.white,
              borderRadius: BorderRadius.circular(4),
              child: SizedBox(
                width: width,
                child: builder((value) => Navigator.pop(context, value)),
              ),
            ),
          ),
        ],
      );
    },
  );
}

class _GalleryShell extends StatelessWidget {
  const _GalleryShell({
    required this.title,
    required this.subtitle,
    required this.children,
  });

  final String title;
  final String subtitle;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 10, 12, 12),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: WordTheme.ribbonLabel.copyWith(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: const Color(0xFF2B579A),
            ),
          ),
          const SizedBox(height: 2),
          Text(
            subtitle,
            style: WordTheme.ribbonLabel.copyWith(
              fontSize: 11,
              color: const Color(0xFF666666),
            ),
          ),
          const SizedBox(height: 10),
          Wrap(spacing: 8, runSpacing: 8, children: children),
        ],
      ),
    );
  }
}

class _PreviewTile extends StatefulWidget {
  const _PreviewTile({
    super.key,
    required this.label,
    required this.onTap,
    required this.painter,
    this.wide = false,
  });

  final String label;
  final VoidCallback onTap;
  final CustomPainter painter;
  final bool wide;

  @override
  State<_PreviewTile> createState() => _PreviewTileState();
}

class _PreviewTileState extends State<_PreviewTile> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final w = widget.wide ? 76.0 : 72.0;
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Tooltip(
          message: widget.label,
          child: Container(
            width: w,
            padding: const EdgeInsets.fromLTRB(4, 4, 4, 6),
            decoration: BoxDecoration(
              color: _hovered ? const Color(0xFFCDE6F7) : Colors.white,
              border: Border.all(
                color: _hovered
                    ? const Color(0xFF2B579A)
                    : const Color(0xFFB0B0B0),
              ),
              borderRadius: BorderRadius.circular(3),
            ),
            child: Column(
              children: [
                Container(
                  width: w - 10,
                  height: 48,
                  color: Colors.white,
                  child: CustomPaint(painter: widget.painter),
                ),
                const SizedBox(height: 4),
                Text(
                  widget.label,
                  textAlign: TextAlign.center,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: WordTheme.ribbonLabel.copyWith(fontSize: 9),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

enum _ChartGlyph { column, bar, line, pie }

class _ChartPreviewPainter extends CustomPainter {
  const _ChartPreviewPainter(this.glyph);

  final _ChartGlyph glyph;

  static const _blue = Color(0xFF4472C4);
  static const _orange = Color(0xFFED7D31);
  static const _axis = Color(0xFF595959);

  @override
  void paint(Canvas canvas, Size size) {
    final axis = Paint()
      ..color = _axis
      ..strokeWidth = 1;
    switch (glyph) {
      case _ChartGlyph.column:
        canvas.drawLine(Offset(6, 4), Offset(6, size.height - 4), axis);
        canvas.drawLine(
          Offset(6, size.height - 4),
          Offset(size.width - 4, size.height - 4),
          axis,
        );
        final bars = [0.45, 0.75, 0.55, 0.9];
        final slot = (size.width - 14) / bars.length;
        for (var i = 0; i < bars.length; i++) {
          final h = (size.height - 10) * bars[i];
          canvas.drawRect(
            Rect.fromLTWH(
              8 + i * slot + 2,
              size.height - 4 - h,
              slot * 0.35,
              h,
            ),
            Paint()..color = _blue,
          );
          canvas.drawRect(
            Rect.fromLTWH(
              8 + i * slot + slot * 0.4,
              size.height - 4 - h * 0.7,
              slot * 0.35,
              h * 0.7,
            ),
            Paint()..color = _orange,
          );
        }
      case _ChartGlyph.bar:
        canvas.drawLine(Offset(6, 4), Offset(6, size.height - 4), axis);
        canvas.drawLine(
          Offset(6, size.height - 4),
          Offset(size.width - 4, size.height - 4),
          axis,
        );
        final widths = [0.5, 0.8, 0.4, 0.7];
        final slot = (size.height - 12) / widths.length;
        for (var i = 0; i < widths.length; i++) {
          final w = (size.width - 14) * widths[i];
          canvas.drawRect(
            Rect.fromLTWH(8, 6 + i * slot, w * 0.55, slot * 0.35),
            Paint()..color = _blue,
          );
          canvas.drawRect(
            Rect.fromLTWH(8, 6 + i * slot + slot * 0.4, w * 0.4, slot * 0.35),
            Paint()..color = _orange,
          );
        }
      case _ChartGlyph.line:
        canvas.drawLine(Offset(6, 4), Offset(6, size.height - 4), axis);
        canvas.drawLine(
          Offset(6, size.height - 4),
          Offset(size.width - 4, size.height - 4),
          axis,
        );
        final pts = [
          Offset(10, size.height * 0.65),
          Offset(size.width * 0.35, size.height * 0.35),
          Offset(size.width * 0.6, size.height * 0.55),
          Offset(size.width - 8, size.height * 0.25),
        ];
        final line = Paint()
          ..color = _blue
          ..strokeWidth = 1.8
          ..style = PaintingStyle.stroke;
        final path = Path()..moveTo(pts.first.dx, pts.first.dy);
        for (final p in pts.skip(1)) {
          path.lineTo(p.dx, p.dy);
        }
        canvas.drawPath(path, line);
        for (final p in pts) {
          canvas.drawCircle(p, 2.2, Paint()..color = _blue);
        }
      case _ChartGlyph.pie:
        final center = Offset(size.width * 0.45, size.height * 0.52);
        final radius = size.shortestSide * 0.36;
        var start = -math.pi / 2;
        final sweeps = [0.35, 0.25, 0.2, 0.2];
        final colors = [_blue, _orange, const Color(0xFFA5A5A5), const Color(0xFFFFC000)];
        for (var i = 0; i < sweeps.length; i++) {
          final sweep = sweeps[i] * math.pi * 2;
          canvas.drawArc(
            Rect.fromCircle(center: center, radius: radius),
            start,
            sweep,
            true,
            Paint()..color = colors[i],
          );
          start += sweep;
        }
    }
  }

  @override
  bool shouldRepaint(covariant _ChartPreviewPainter oldDelegate) =>
      oldDelegate.glyph != glyph;
}

enum _SmartArtGlyph { process, hierarchy, cycle }

class _SmartArtPreviewPainter extends CustomPainter {
  const _SmartArtPreviewPainter(this.glyph);

  final _SmartArtGlyph glyph;

  static const _fill = Color(0xFF5B9BD5);
  static const _stroke = Color(0xFF2F5496);

  @override
  void paint(Canvas canvas, Size size) {
    final fill = Paint()..color = _fill;
    final stroke = Paint()
      ..color = _stroke
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2;

    switch (glyph) {
      case _SmartArtGlyph.process:
        for (var i = 0; i < 3; i++) {
          final x = 6.0 + i * (size.width / 3.1);
          final r = RRect.fromRectAndRadius(
            Rect.fromLTWH(x, size.height * 0.28, size.width * 0.22, size.height * 0.44),
            const Radius.circular(2),
          );
          canvas.drawRRect(r, fill);
          canvas.drawRRect(r, stroke);
          if (i < 2) {
            final ax = x + size.width * 0.24;
            final ay = size.height * 0.5;
            canvas.drawLine(Offset(ax, ay), Offset(ax + 8, ay), stroke);
            canvas.drawLine(Offset(ax + 5, ay - 3), Offset(ax + 8, ay), stroke);
            canvas.drawLine(Offset(ax + 5, ay + 3), Offset(ax + 8, ay), stroke);
          }
        }
      case _SmartArtGlyph.hierarchy:
        final top = RRect.fromRectAndRadius(
          Rect.fromLTWH(size.width * 0.32, 4, size.width * 0.36, 14),
          const Radius.circular(2),
        );
        canvas.drawRRect(top, fill);
        canvas.drawRRect(top, stroke);
        canvas.drawLine(
          Offset(size.width * 0.5, 18),
          Offset(size.width * 0.5, 26),
          stroke,
        );
        canvas.drawLine(
          Offset(size.width * 0.22, 26),
          Offset(size.width * 0.78, 26),
          stroke,
        );
        for (final x in [0.1, 0.38, 0.66]) {
          canvas.drawLine(
            Offset(size.width * (x + 0.12), 26),
            Offset(size.width * (x + 0.12), 32),
            stroke,
          );
          final r = RRect.fromRectAndRadius(
            Rect.fromLTWH(size.width * x, 32, size.width * 0.24, 12),
            const Radius.circular(2),
          );
          canvas.drawRRect(r, fill);
          canvas.drawRRect(r, stroke);
        }
      case _SmartArtGlyph.cycle:
        final c = Offset(size.width / 2, size.height / 2);
        final r = size.shortestSide * 0.28;
        canvas.drawCircle(c, r, stroke);
        for (var i = 0; i < 3; i++) {
          final t = -math.pi / 2 + i * (math.pi * 2 / 3);
          final p = Offset(c.dx + r * math.cos(t), c.dy + r * math.sin(t));
          canvas.drawCircle(p, 7, fill);
          canvas.drawCircle(p, 7, stroke);
        }
    }
  }

  @override
  bool shouldRepaint(covariant _SmartArtPreviewPainter oldDelegate) =>
      oldDelegate.glyph != glyph;
}

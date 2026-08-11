import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Word-style Insert → Shapes gallery (icon grid).
class ShapePicker extends StatelessWidget {
  const ShapePicker({super.key, required this.onSelected});

  final ValueChanged<int> onSelected;

  static const _items = <({int type, String label, _ShapeGlyph glyph})>[
    (
      type: EditorController.shapeRectangle,
      label: 'Rectangle',
      glyph: _ShapeGlyph.rectangle,
    ),
    (
      type: EditorController.shapeEllipse,
      label: 'Ellipse',
      glyph: _ShapeGlyph.ellipse,
    ),
    (
      type: EditorController.shapeLine,
      label: 'Line',
      glyph: _ShapeGlyph.line,
    ),
  ];

  /// Shows an anchored popup under [anchorContext] and returns the chosen shape
  /// type, or `null` if dismissed.
  static Future<int?> show(BuildContext anchorContext) async {
    final box = anchorContext.findRenderObject() as RenderBox?;
    if (box == null) return null;
    final overlay =
        Overlay.of(anchorContext).context.findRenderObject() as RenderBox;
    final origin = box.localToGlobal(Offset.zero, ancestor: overlay);
    final size = box.size;

    return showGeneralDialog<int>(
      context: anchorContext,
      barrierDismissible: true,
      barrierLabel: 'Dismiss',
      barrierColor: Colors.transparent,
      pageBuilder: (context, animation, secondaryAnimation) {
        return Stack(
          children: [
            Positioned(
              left: origin.dx.clamp(8.0, overlay.size.width - 220),
              top: origin.dy + size.height + 2,
              child: Material(
                elevation: 6,
                color: Colors.white,
                borderRadius: BorderRadius.circular(4),
                child: ShapePicker(
                  onSelected: (value) => Navigator.pop(context, value),
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(10, 8, 10, 10),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Shapes', style: WordTheme.ribbonLabel),
          const SizedBox(height: 8),
          Wrap(
            spacing: 6,
            runSpacing: 6,
            children: [
              for (final item in _items)
                _ShapeTile(
                  key: Key('shape_picker_${item.label.toLowerCase()}'),
                  label: item.label,
                  glyph: item.glyph,
                  onTap: () => onSelected(item.type),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

enum _ShapeGlyph { rectangle, ellipse, line }

class _ShapeTile extends StatefulWidget {
  const _ShapeTile({
    super.key,
    required this.label,
    required this.glyph,
    required this.onTap,
  });

  final String label;
  final _ShapeGlyph glyph;
  final VoidCallback onTap;

  @override
  State<_ShapeTile> createState() => _ShapeTileState();
}

class _ShapeTileState extends State<_ShapeTile> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Tooltip(
          message: widget.label,
          child: Container(
            width: 56,
            height: 56,
            decoration: BoxDecoration(
              color: _hovered ? const Color(0xFFCDE6F7) : Colors.white,
              border: Border.all(
                color: _hovered
                    ? const Color(0xFF2B579A)
                    : const Color(0xFFB0B0B0),
              ),
              borderRadius: BorderRadius.circular(3),
            ),
            child: CustomPaint(painter: _ShapeIconPainter(widget.glyph)),
          ),
        ),
      ),
    );
  }
}

class _ShapeIconPainter extends CustomPainter {
  const _ShapeIconPainter(this.glyph);

  final _ShapeGlyph glyph;

  @override
  void paint(Canvas canvas, Size size) {
    final stroke = Paint()
      ..color = const Color(0xFF2B579A)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;
    final fill = Paint()
      ..color = const Color(0xFF5B9BD5)
      ..style = PaintingStyle.fill;
    final inset = Rect.fromLTWH(12, 12, size.width - 24, size.height - 24);

    switch (glyph) {
      case _ShapeGlyph.rectangle:
        canvas.drawRect(inset, fill);
        canvas.drawRect(inset, stroke);
      case _ShapeGlyph.ellipse:
        canvas.drawOval(inset, fill);
        canvas.drawOval(inset, stroke);
      case _ShapeGlyph.line:
        canvas.drawLine(
          Offset(inset.left, inset.bottom),
          Offset(inset.right, inset.top),
          stroke..strokeWidth = 2.5,
        );
    }
  }

  @override
  bool shouldRepaint(covariant _ShapeIconPainter oldDelegate) =>
      oldDelegate.glyph != glyph;
}

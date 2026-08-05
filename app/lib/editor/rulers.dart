import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Horizontal and vertical rulers synced to page margins.
class DocumentRulers extends StatelessWidget {
  const DocumentRulers({
    super.key,
    required this.controller,
    required this.child,
  });

  final EditorController controller;
  final Widget child;

  static const double _rulerSize = 20;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(width: _rulerSize),
        Expanded(
          child: Column(
            children: [
              SizedBox(
                height: _rulerSize,
                child: CustomPaint(
                  painter: _HorizontalRulerPainter(
                    pageWidth: controller.pageWidth,
                    marginLeft: 72,
                    marginRight: 72,
                  ),
                  size: Size.infinite,
                ),
              ),
              Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SizedBox(
                      width: _rulerSize,
                      child: CustomPaint(
                        painter: _VerticalRulerPainter(
                          pageHeight: controller.pageHeight,
                          marginTop: 72,
                        ),
                        size: Size.infinite,
                      ),
                    ),
                    Expanded(child: child),
                  ],
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _HorizontalRulerPainter extends CustomPainter {
  _HorizontalRulerPainter({
    required this.pageWidth,
    required this.marginLeft,
    required this.marginRight,
  });

  final double pageWidth;
  final double marginLeft;
  final double marginRight;

  @override
  void paint(Canvas canvas, Size size) {
    final bg = Paint()..color = const Color(0xFFF0F0F0);
    canvas.drawRect(Offset.zero & size, bg);
    final tick = Paint()..color = Colors.grey;
    for (var i = 0; i <= pageWidth; i += 36) {
      canvas.drawLine(Offset(i.toDouble(), 14), Offset(i.toDouble(), 20), tick);
    }
    final marginPaint = Paint()..color = Colors.blue.withValues(alpha: 0.4);
    canvas.drawRect(Rect.fromLTWH(0, 0, marginLeft, size.height), marginPaint);
    canvas.drawRect(
      Rect.fromLTWH(pageWidth - marginRight, 0, marginRight, size.height),
      marginPaint,
    );
  }

  @override
  bool shouldRepaint(covariant _HorizontalRulerPainter oldDelegate) => false;
}

class _VerticalRulerPainter extends CustomPainter {
  _VerticalRulerPainter({required this.pageHeight, required this.marginTop});

  final double pageHeight;
  final double marginTop;

  @override
  void paint(Canvas canvas, Size size) {
    final bg = Paint()..color = const Color(0xFFF0F0F0);
    canvas.drawRect(Offset.zero & size, bg);
    final tick = Paint()..color = Colors.grey;
    for (var i = 0; i <= pageHeight; i += 36) {
      canvas.drawLine(Offset(14, i.toDouble()), Offset(20, i.toDouble()), tick);
    }
    final marginPaint = Paint()..color = Colors.blue.withValues(alpha: 0.4);
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, marginTop), marginPaint);
  }

  @override
  bool shouldRepaint(covariant _VerticalRulerPainter oldDelegate) => false;
}

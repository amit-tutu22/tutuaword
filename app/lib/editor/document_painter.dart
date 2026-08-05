import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:tutuaword/editor/display_list.dart';

/// Paints Rust display list via drawRawAtlas + rect/path batches (ADR-0003).
class DocumentPainter extends CustomPainter {
  DocumentPainter({
    required this.snapshot,
    this.atlasImage,
    this.images = const {},
  });

  final DisplayListSnapshot snapshot;
  final ui.Image? atlasImage;

  /// Decoded document images keyed by asset id.
  final Map<String, ui.Image> images;

  @override
  void paint(Canvas canvas, Size size) {
    _paintRects(canvas);
    _paintImages(canvas);
    _paintPaths(canvas);
    _paintGlyphs(canvas);
  }

  void _paintImages(Canvas canvas) {
    final count = snapshot.imageTransforms.length ~/ 2;
    if (count == 0) return;

    final paint = Paint()
      ..isAntiAlias = true
      ..filterQuality = FilterQuality.medium;
    final placeholderBorder = Paint()
      ..color = const Color(0xFF999999)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    for (var i = 0; i < count; i++) {
      final x = snapshot.imageTransforms[i * 2];
      final y = snapshot.imageTransforms[i * 2 + 1];
      final w = snapshot.imageSizes[i * 2];
      final h = snapshot.imageSizes[i * 2 + 1];
      final dst = Rect.fromLTWH(x, y, w, h);

      final image = i < snapshot.imageAssetIds.length
          ? images[snapshot.imageAssetIds[i]]
          : null;
      if (image == null) {
        // Rust already filled the grey block; just outline the empty frame.
        canvas.drawRect(dst, placeholderBorder);
        continue;
      }
      final src = Rect.fromLTWH(
        0,
        0,
        image.width.toDouble(),
        image.height.toDouble(),
      );
      canvas.drawImageRect(image, src, dst, paint);
    }
  }

  void _paintRects(Canvas canvas) {
    for (var i = 0; i < snapshot.rectColors.length; i++) {
      final base = i * 4;
      if (base + 3 >= snapshot.rectBatch.length) break;
      final x = snapshot.rectBatch[base];
      final y = snapshot.rectBatch[base + 1];
      final w = snapshot.rectBatch[base + 2];
      final h = snapshot.rectBatch[base + 3];
      final color = Color(snapshot.rectColors[i]);
      canvas.drawRect(Rect.fromLTWH(x, y, w, h), Paint()..color = color);
    }
  }

  void _paintPaths(Canvas canvas) {
    final paint = Paint()
      ..strokeWidth = 0.5
      ..style = PaintingStyle.stroke;
    for (var i = 0; i < snapshot.pathColors.length; i++) {
      final base = i * 4;
      if (base + 3 >= snapshot.pathPoints.length) break;
      paint.color = Color(snapshot.pathColors[i]);
      canvas.drawLine(
        Offset(snapshot.pathPoints[base], snapshot.pathPoints[base + 1]),
        Offset(snapshot.pathPoints[base + 2], snapshot.pathPoints[base + 3]),
        paint,
      );
    }
  }

  void _paintGlyphs(Canvas canvas) {
    if (atlasImage == null || snapshot.glyphOffsets.isEmpty) return;

    final count = snapshot.glyphOffsets.length ~/ 2;
    // drawRawAtlas expects 4 floats per glyph: [scos, ssin, tx, ty].
    final rstTransforms = Float32List(count * 4);
    // Source rects in atlas are LTRB, not XYWH.
    final srcRects = Float32List(count * 4);

    for (var i = 0; i < count; i++) {
      final x = snapshot.glyphOffsets[i * 2];
      final y = snapshot.glyphOffsets[i * 2 + 1];
      rstTransforms[i * 4] = 1.0;
      rstTransforms[i * 4 + 1] = 0.0;
      rstTransforms[i * 4 + 2] = x;
      rstTransforms[i * 4 + 3] = y;

      final atlasX = snapshot.glyphSrcRects[i * 4];
      final atlasY = snapshot.glyphSrcRects[i * 4 + 1];
      final atlasW = snapshot.glyphSrcRects[i * 4 + 2];
      final atlasH = snapshot.glyphSrcRects[i * 4 + 3];
      srcRects[i * 4] = atlasX;
      srcRects[i * 4 + 1] = atlasY;
      srcRects[i * 4 + 2] = atlasX + atlasW;
      srcRects[i * 4 + 3] = atlasY + atlasH;
    }

    // Modulate multiplies the run color into the premultiplied glyph mask,
    // which preserves antialiasing; srcOver would flood the whole sprite rect.
    canvas.drawRawAtlas(
      atlasImage!,
      rstTransforms,
      srcRects,
      snapshot.glyphColors,
      ui.BlendMode.modulate,
      null,
      Paint(),
    );
  }

  @override
  bool shouldRepaint(covariant DocumentPainter oldDelegate) {
    return oldDelegate.snapshot.version != snapshot.version ||
        oldDelegate.atlasImage != atlasImage ||
        oldDelegate.images.length != images.length;
  }
}

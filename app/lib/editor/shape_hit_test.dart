import 'dart:ui';

import 'package:tutuaword/editor/display_list.dart';

class ShapeBounds {
  const ShapeBounds({
    required this.shapeId,
    required this.index,
    required this.rect,
  });

  final String shapeId;
  final int index;
  final Rect rect;
}

ShapeBounds? hitTestShape(DisplayListSnapshot snapshot, Offset point) {
  final count = snapshot.shapeIds.length;
  for (var i = count - 1; i >= 0; i--) {
    final rect = _shapeRect(snapshot, i);
    if (rect.contains(point)) {
      return ShapeBounds(shapeId: snapshot.shapeIds[i], index: i, rect: rect);
    }
  }
  return null;
}

Rect _shapeRect(DisplayListSnapshot snapshot, int index) {
  final x = snapshot.shapeRects[index * 4];
  final y = snapshot.shapeRects[index * 4 + 1];
  final w = snapshot.shapeRects[index * 4 + 2];
  final h = snapshot.shapeRects[index * 4 + 3];
  return Rect.fromLTWH(x, y, w, h);
}

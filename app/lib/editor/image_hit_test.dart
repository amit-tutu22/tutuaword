import 'dart:ui';

import 'package:tutuaword/editor/display_list.dart';

/// Which resize handle on a selected image is active.
enum ImageResizeHandle {
  topLeft,
  topRight,
  bottomLeft,
  bottomRight,
}

class ImageBounds {
  const ImageBounds({
    required this.imageId,
    required this.index,
    required this.rect,
  });

  final String imageId;
  final int index;
  final Rect rect;
}

const _handleHitRadius = 8.0;

/// Hit-test images on a page display list snapshot.
ImageBounds? hitTestImage(DisplayListSnapshot snapshot, Offset point) {
  final count = snapshot.imageIds.isNotEmpty
      ? snapshot.imageIds.length
      : snapshot.imageAssetIds.length;
  for (var i = count - 1; i >= 0; i--) {
    final rect = _imageRect(snapshot, i);
    if (rect.contains(point)) {
      final id = i < snapshot.imageIds.length && snapshot.imageIds[i].isNotEmpty
          ? snapshot.imageIds[i]
          : snapshot.imageAssetIds[i];
      return ImageBounds(imageId: id, index: i, rect: rect);
    }
  }
  return null;
}

ImageResizeHandle? hitTestImageHandle(Rect bounds, Offset point) {
  final handles = <ImageResizeHandle, Offset>{
    ImageResizeHandle.topLeft: bounds.topLeft,
    ImageResizeHandle.topRight: bounds.topRight,
    ImageResizeHandle.bottomLeft: bounds.bottomLeft,
    ImageResizeHandle.bottomRight: bounds.bottomRight,
  };
  for (final entry in handles.entries) {
    if ((entry.value - point).distance <= _handleHitRadius) {
      return entry.key;
    }
  }
  return null;
}

Rect resizeImageWithHandle({
  required Rect start,
  required ImageResizeHandle handle,
  required Offset current,
  required Offset origin,
  required bool lockAspectRatio,
}) {
  var left = start.left;
  var top = start.top;
  var right = start.right;
  var bottom = start.bottom;

  switch (handle) {
    case ImageResizeHandle.topLeft:
      left = current.dx;
      top = current.dy;
    case ImageResizeHandle.topRight:
      right = current.dx;
      top = current.dy;
    case ImageResizeHandle.bottomLeft:
      left = current.dx;
      bottom = current.dy;
    case ImageResizeHandle.bottomRight:
      right = current.dx;
      bottom = current.dy;
  }

  if (lockAspectRatio && start.width > 0 && start.height > 0) {
    final aspect = start.width / start.height;
    final newW = (right - left).abs();
    final newH = (bottom - top).abs();
    if (newW / newH > aspect) {
      final h = newW / aspect;
      if (handle == ImageResizeHandle.topLeft || handle == ImageResizeHandle.topRight) {
        top = bottom - h;
      } else {
        bottom = top + h;
      }
    } else {
      final w = newH * aspect;
      if (handle == ImageResizeHandle.topLeft || handle == ImageResizeHandle.bottomLeft) {
        left = right - w;
      } else {
        right = left + w;
      }
    }
  }

  final width = (right - left).abs().clamp(8.0, 2000.0);
  final height = (bottom - top).abs().clamp(8.0, 2000.0);
  return Rect.fromLTWH(
    left < right ? left : right,
    top < bottom ? top : bottom,
    width,
    height,
  );
}

Rect _imageRect(DisplayListSnapshot snapshot, int index) {
  final x = snapshot.imageTransforms[index * 2];
  final y = snapshot.imageTransforms[index * 2 + 1];
  final w = snapshot.imageSizes[index * 2];
  final h = snapshot.imageSizes[index * 2 + 1];
  return Rect.fromLTWH(x, y, w, h);
}

List<Offset> imageHandlePoints(Rect bounds) {
  return [
    bounds.topLeft,
    bounds.topRight,
    bounds.bottomLeft,
    bounds.bottomRight,
  ];
}

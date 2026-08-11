import 'package:flutter/material.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_painter.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Page thumbnail strip for multi-page navigation (F19.S1).
class PageNavigator extends StatelessWidget {
  const PageNavigator({
    super.key,
    required this.controller,
    required this.currentPage,
    required this.onPageSelected,
  });

  final EditorController controller;
  final int currentPage;
  final ValueChanged<int> onPageSelected;

  @override
  Widget build(BuildContext context) {
    final pageCount = controller.pageCount;
    return ListView.builder(
      padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
      itemCount: pageCount,
      itemBuilder: (context, index) {
        final selected = index == currentPage;
        return Padding(
          padding: const EdgeInsets.only(bottom: 10),
          child: _PageThumbnailTile(
            key: ValueKey('page-thumbnail-$index'),
            controller: controller,
            pageIndex: index,
            selected: selected,
            onTap: () => onPageSelected(index),
          ),
        );
      },
    );
  }
}

class _PageThumbnailTile extends StatelessWidget {
  const _PageThumbnailTile({
    super.key,
    required this.controller,
    required this.pageIndex,
    required this.selected,
    required this.onTap,
  });

  final EditorController controller;
  final int pageIndex;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final pageW = controller.pageWidth.clamp(1.0, 10000.0);
    final pageH = controller.pageHeight.clamp(1.0, 10000.0);
    final aspect = pageW / pageH;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(2),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 120),
            decoration: BoxDecoration(
              color: Colors.white,
              border: Border.all(
                color: selected
                    ? theme.colorScheme.primary
                    : Colors.grey.shade400,
                width: selected ? 2 : 1,
              ),
              boxShadow: selected
                  ? [
                      BoxShadow(
                        color: Colors.black.withValues(alpha: 0.12),
                        blurRadius: 4,
                        offset: const Offset(0, 1),
                      ),
                    ]
                  : null,
            ),
            child: AspectRatio(
              aspectRatio: aspect,
              child: _PageThumbnailPreview(
                controller: controller,
                pageIndex: pageIndex,
              ),
            ),
          ),
        ),
        const SizedBox(height: 4),
        Text(
          '${pageIndex + 1}',
          textAlign: TextAlign.center,
          style: TextStyle(
            fontSize: 11,
            fontWeight: selected ? FontWeight.w700 : FontWeight.w500,
            color: selected
                ? theme.colorScheme.primary
                : theme.colorScheme.onSurfaceVariant,
          ),
        ),
      ],
    );
  }
}

class _PageThumbnailPreview extends StatelessWidget {
  const _PageThumbnailPreview({
    required this.controller,
    required this.pageIndex,
  });

  final EditorController controller;
  final int pageIndex;

  @override
  Widget build(BuildContext context) {
    final bytes = controller.displayListForPage(pageIndex);
    final pageW = controller.pageWidth.clamp(1.0, 10000.0);
    final pageH = controller.pageHeight.clamp(1.0, 10000.0);

    if (bytes.isEmpty) {
      return _ThumbnailPlaceholder(pageIndex: pageIndex);
    }

    final snapshot = DisplayListSnapshot.fromBytes(bytes);
    if (!snapshot.hasPaintableContent) {
      return _ThumbnailPlaceholder(pageIndex: pageIndex);
    }

    return ClipRect(
      child: FittedBox(
        fit: BoxFit.contain,
        alignment: Alignment.topCenter,
        child: SizedBox(
          width: pageW,
          height: pageH,
          child: CustomPaint(
            size: Size(pageW, pageH),
            painter: DocumentPainter(snapshot: snapshot),
          ),
        ),
      ),
    );
  }
}

class _ThumbnailPlaceholder extends StatelessWidget {
  const _ThumbnailPlaceholder({required this.pageIndex});

  final int pageIndex;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: Colors.white,
      child: CustomPaint(
        painter: _MarginGuidePainter(),
        child: Center(
          child: Text(
            '${pageIndex + 1}',
            style: TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.w600,
              color: Colors.grey.shade500,
            ),
          ),
        ),
      ),
    );
  }
}

/// Light margin guides so empty / loading thumbnails still read as pages.
class _MarginGuidePainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xFFE8E8E8)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    final inset = size.shortestSide * 0.12;
    canvas.drawRect(
      Rect.fromLTWH(inset, inset, size.width - inset * 2, size.height - inset * 2),
      paint,
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

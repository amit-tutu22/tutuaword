import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

class WordStatusBar extends StatelessWidget {
  const WordStatusBar({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    final viewPadding = MediaQuery.paddingOf(context);
    final phone = WordTheme.phoneChrome(context);
    return Tooltip(
      message: controller.statusText,
      child: Container(
        color: WordTheme.statusBarSurface,
        padding: EdgeInsets.only(
          bottom: viewPadding.bottom,
          left: viewPadding.left + 8,
          right: viewPadding.right + 8,
        ),
        child: SizedBox(
          height: WordTheme.statusBarHeight,
          child: phone ? _buildPhoneBar(context) : _buildDesktopBar(context),
        ),
      ),
    );
  }

  Widget _buildPhoneBar(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Text(
            'Page ${controller.currentPage + 1} of ${controller.pageCount} · ${controller.wordCount} words',
            style: WordTheme.statusBarText,
            overflow: TextOverflow.ellipsis,
            maxLines: 1,
          ),
        ),
        IconButton(
          key: const Key('status_view_menu'),
          icon: const Icon(Icons.more_horiz, size: 16),
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
          onPressed: () => _showViewMenu(context),
          tooltip: 'View options',
        ),
        IconButton(
          icon: const Icon(Icons.remove, size: 14),
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
          onPressed: controller.zoomOut,
          tooltip: 'Zoom out',
        ),
        GestureDetector(
          onTap: () => _showZoomMenu(context),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4),
            child: Text(
              '${(controller.zoom * 100).round()}%',
              style: WordTheme.statusBarText,
            ),
          ),
        ),
        IconButton(
          icon: const Icon(Icons.add, size: 14),
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
          onPressed: controller.zoomIn,
          tooltip: 'Zoom in',
        ),
      ],
    );
  }

  void _showViewMenu(BuildContext context) {
    showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.article_outlined, size: 20),
              title: const Text('Read Mode'),
              trailing: controller.isReadMode ? const Icon(Icons.check, size: 18) : null,
              onTap: () {
                controller.setReadMode();
                Navigator.pop(context);
              },
            ),
            ListTile(
              leading: const Icon(Icons.print_outlined, size: 20),
              title: const Text('Print Layout'),
              trailing: !controller.printPreview &&
                      !controller.isReadMode &&
                      !controller.isWebLayout
                  ? const Icon(Icons.check, size: 18)
                  : null,
              onTap: () {
                controller.setPrintLayout();
                Navigator.pop(context);
              },
            ),
            ListTile(
              leading: const Icon(Icons.preview_outlined, size: 20),
              title: const Text('Print Preview'),
              trailing: controller.printPreview ? const Icon(Icons.check, size: 18) : null,
              onTap: () {
                controller.setPrintPreviewMode();
                Navigator.pop(context);
              },
            ),
            ListTile(
              leading: const Icon(Icons.web, size: 20),
              title: const Text('Web Layout'),
              trailing: controller.isWebLayout ? const Icon(Icons.check, size: 18) : null,
              onTap: () {
                controller.setWebLayout();
                Navigator.pop(context);
              },
            ),
            const Divider(height: 1),
            ListTile(
              leading: const Icon(Icons.check_circle_outline, size: 20),
              title: Text(controller.accessibilityStatusLabel),
              dense: true,
            ),
            ListTile(
              leading: const Icon(Icons.language, size: 20),
              title: const Text('English (India)'),
              dense: true,
            ),
          ],
        ),
      ),
    );
  }

  void _showZoomMenu(BuildContext context) {
    showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'Zoom ${(controller.zoom * 100).round()}%',
                style: WordTheme.statusBarText.copyWith(fontWeight: FontWeight.w600),
              ),
              Slider(
                value: controller.zoom.clamp(0.5, 3.0),
                min: 0.5,
                max: 3.0,
                onChanged: controller.setZoom,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildDesktopBar(BuildContext context) {
    return Row(
      children: [
        Flexible(
          child: SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                _StatusItem('Page ${controller.currentPage + 1} of ${controller.pageCount}'),
                _divider(),
                _StatusItem('${controller.wordCount} words'),
                _divider(),
                const Icon(Icons.check_circle_outline, size: 12, color: Color(0xFF555555)),
                _divider(),
                _StatusItem('English (India)'),
                _divider(),
                _StatusItem(controller.accessibilityStatusLabel),
              ],
            ),
          ),
        ),
        _ViewModeButton(
          icon: Icons.article_outlined,
          tooltip: 'Read Mode',
          selected: controller.isReadMode,
          onPressed: controller.setReadMode,
        ),
        _ViewModeButton(
          icon: Icons.print_outlined,
          tooltip: 'Print Layout',
          selected: !controller.printPreview &&
              !controller.isReadMode &&
              !controller.isWebLayout,
          onPressed: controller.setPrintLayout,
        ),
        _ViewModeButton(
          icon: Icons.preview_outlined,
          tooltip: 'Print Preview',
          selected: controller.printPreview,
          onPressed: controller.setPrintPreviewMode,
        ),
        _ViewModeButton(
          icon: Icons.web,
          tooltip: 'Web Layout',
          selected: controller.isWebLayout,
          onPressed: controller.setWebLayout,
        ),
        _divider(),
        IconButton(
          icon: const Icon(Icons.remove, size: 14),
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
          onPressed: controller.zoomOut,
          tooltip: 'Zoom out',
        ),
        LayoutBuilder(
          builder: (context, constraints) {
            // Parent Row may not give a bounded maxWidth; cap slider anyway.
            return SizedBox(
              width: 100,
              child: Slider(
                value: controller.zoom.clamp(0.5, 3.0),
                min: 0.5,
                max: 3.0,
                onChanged: controller.setZoom,
              ),
            );
          },
        ),
        IconButton(
          icon: const Icon(Icons.add, size: 14),
          padding: EdgeInsets.zero,
          constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
          onPressed: controller.zoomIn,
          tooltip: 'Zoom in',
        ),
        _StatusItem('${(controller.zoom * 100).round()}%'),
        const SizedBox(width: 4),
      ],
    );
  }

  Widget _divider() => Container(
        width: 1,
        height: 14,
        margin: const EdgeInsets.symmetric(horizontal: 8),
        color: WordTheme.groupDivider,
      );
}

class _StatusItem extends StatelessWidget {
  const _StatusItem(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Text(text, style: WordTheme.statusBarText);
  }
}

class _ViewModeButton extends StatefulWidget {
  const _ViewModeButton({
    required this.icon,
    required this.tooltip,
    this.selected = false,
    this.onPressed,
  });

  final IconData icon;
  final String tooltip;
  final bool selected;
  final VoidCallback? onPressed;

  @override
  State<_ViewModeButton> createState() => _ViewModeButtonState();
}

class _ViewModeButtonState extends State<_ViewModeButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            width: 22,
            height: 20,
            margin: const EdgeInsets.symmetric(horizontal: 1),
            decoration: BoxDecoration(
              color: widget.selected
                  ? WordTheme.ribbonSelected
                  : (_hovered ? WordTheme.ribbonHover : Colors.transparent),
              borderRadius: BorderRadius.circular(2),
            ),
            child: Icon(
              widget.icon,
              size: 14,
              color: WordTheme.ribbonText,
            ),
          ),
        ),
      ),
    );
  }
}

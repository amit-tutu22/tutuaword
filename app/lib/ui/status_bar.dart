import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

class WordStatusBar extends StatelessWidget {
  const WordStatusBar({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    // The home indicator sits over the app, so the surface extends past it
    // while the controls stay above.
    final viewPadding = MediaQuery.paddingOf(context);
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
          child: Row(
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
                      _StatusItem('Accessibility: Good to go'),
                    ],
                  ),
                ),
              ),
              _ViewModeButton(icon: Icons.article_outlined, tooltip: 'Read Mode', onPressed: null),
              _ViewModeButton(
                icon: Icons.print_outlined,
                tooltip: 'Print Layout',
                selected: !controller.printPreview,
                onPressed: () {
                  if (controller.printPreview) controller.togglePrintPreview();
                },
              ),
              _ViewModeButton(
                icon: Icons.preview_outlined,
                tooltip: 'Print Preview',
                selected: controller.printPreview,
                onPressed: () {
                  if (!controller.printPreview) controller.togglePrintPreview();
                },
              ),
              _ViewModeButton(icon: Icons.web, tooltip: 'Web Layout', onPressed: null),
              _divider(),
              IconButton(
                icon: const Icon(Icons.remove, size: 14),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 20, minHeight: 20),
                onPressed: controller.zoomOut,
                tooltip: 'Zoom out',
              ),
              SizedBox(
                width: 100,
                child: Slider(
                  value: controller.zoom,
                  min: 0.5,
                  max: 2.0,
                  onChanged: controller.setZoom,
                ),
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
          ),
        ),
      ),
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

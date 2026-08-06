import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class WordTitleBar extends StatelessWidget {
  const WordTitleBar({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: WordTheme.titleBarHeight,
      color: WordTheme.titleBarBlue,
      child: Row(
        children: [
          const SizedBox(width: WordTheme.trafficLightInset),
          _QuickAccessIcon(
            icon: Icons.home_outlined,
            tooltip: kComingSoonTooltip,
            onPressed: null,
          ),
          _QuickAccessIcon(
            icon: Icons.save_outlined,
            tooltip: 'Save',
            onPressed: () => controller.saveDocument(),
          ),
          _QuickAccessIcon(
            icon: Icons.undo,
            tooltip: 'Undo',
            onPressed: controller.undo,
          ),
          _QuickAccessIcon(
            icon: Icons.redo,
            tooltip: 'Redo',
            onPressed: controller.redo,
          ),
          _QuickAccessIcon(
            icon: Icons.print_outlined,
            tooltip: 'Print',
            onPressed: controller.togglePrintPreview,
          ),
          _QuickAccessIcon(
            icon: Icons.more_horiz,
            tooltip: 'More',
            onPressed: null,
          ),
          Expanded(
            child: Center(
              child: Text(
                controller.documentTitle,
                style: WordTheme.titleBarTitle,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ),
          _QuickAccessIcon(
            icon: Icons.search,
            tooltip: 'Search',
            onPressed: null,
          ),
          const SizedBox(width: 12),
        ],
      ),
    );
  }
}

class _QuickAccessIcon extends StatefulWidget {
  const _QuickAccessIcon({
    required this.icon,
    required this.tooltip,
    this.onPressed,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback? onPressed;

  @override
  State<_QuickAccessIcon> createState() => _QuickAccessIconState();
}

class _QuickAccessIconState extends State<_QuickAccessIcon> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            width: 28,
            height: 28,
            margin: const EdgeInsets.symmetric(horizontal: 1),
            decoration: BoxDecoration(
              color: _hovered && enabled
                  ? Colors.white.withValues(alpha: 0.15)
                  : Colors.transparent,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Icon(
              widget.icon,
              size: 16,
              color: enabled ? WordTheme.titleBarIcon : WordTheme.titleBarIcon.withValues(alpha: 0.4),
            ),
          ),
        ),
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Default tooltip for disabled ribbon controls that are not yet wired.
const kComingSoonTooltip = 'Coming soon';

/// Resolves the tooltip shown for a ribbon control.
String? effectiveRibbonTooltip({String? tooltip, required bool enabled}) {
  if (tooltip != null) return tooltip;
  if (!enabled) return kComingSoonTooltip;
  return null;
}

Widget wrapRibbonTooltip(String? tooltip, Widget child) {
  if (tooltip == null) return child;
  return Tooltip(message: tooltip, child: child);
}

class RibbonGroup extends StatelessWidget {
  const RibbonGroup({
    super.key,
    required this.label,
    required this.child,
    this.showDivider = true,
  });

  final String label;
  final Widget child;
  final bool showDivider;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 6),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(child: Center(child: child)),
              Padding(
                padding: const EdgeInsets.only(bottom: 2, top: 2),
                child: Text(label, style: WordTheme.ribbonGroupLabel),
              ),
            ],
          ),
        ),
        if (showDivider)
          Container(
            width: 1,
            margin: const EdgeInsets.symmetric(vertical: 4),
            color: WordTheme.groupDivider,
          ),
      ],
    );
  }
}

class RibbonIconButton extends StatefulWidget {
  const RibbonIconButton({
    super.key,
    required this.icon,
    this.label,
    this.tooltip,
    this.onPressed,
    this.iconSize = WordTheme.iconSize,
  });

  final IconData icon;
  final String? label;
  final String? tooltip;
  final VoidCallback? onPressed;
  final double iconSize;

  @override
  State<RibbonIconButton> createState() => _RibbonIconButtonState();
}

class _RibbonIconButtonState extends State<RibbonIconButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final color = enabled ? WordTheme.ribbonText : WordTheme.ribbonTextDisabled;
    final bg = _hovered && enabled ? WordTheme.ribbonHover : Colors.transparent;

    final effectiveTooltip =
        effectiveRibbonTooltip(tooltip: widget.tooltip, enabled: enabled);

    return wrapRibbonTooltip(
      effectiveTooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: BoxDecoration(
              color: bg,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(widget.icon, size: widget.iconSize, color: color),
                if (widget.label != null) ...[
                  const SizedBox(height: 2),
                  Text(
                    widget.label!,
                    style: WordTheme.ribbonLabel.copyWith(
                      color: color,
                      fontSize: 10,
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class RibbonToggleButton extends StatefulWidget {
  const RibbonToggleButton({
    super.key,
    required this.icon,
    this.label,
    this.tooltip,
    this.selected = false,
    this.onPressed,
    this.iconSize = WordTheme.iconSize,
  });

  final IconData icon;
  final String? label;
  final String? tooltip;
  final bool selected;
  final VoidCallback? onPressed;
  final double iconSize;

  @override
  State<RibbonToggleButton> createState() => _RibbonToggleButtonState();
}

class _RibbonToggleButtonState extends State<RibbonToggleButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final color = enabled ? WordTheme.ribbonText : WordTheme.ribbonTextDisabled;
    Color bg = Colors.transparent;
    if (widget.selected) {
      bg = WordTheme.ribbonSelected;
    } else if (_hovered && enabled) {
      bg = WordTheme.ribbonHover;
    }

    final effectiveTooltip =
        effectiveRibbonTooltip(tooltip: widget.tooltip, enabled: enabled);

    return wrapRibbonTooltip(
      effectiveTooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: BoxDecoration(
              color: bg,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(widget.icon, size: widget.iconSize, color: color),
                if (widget.label != null) ...[
                  const SizedBox(height: 2),
                  Text(
                    widget.label!,
                    style: WordTheme.ribbonLabel.copyWith(
                      color: color,
                      fontSize: 10,
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class RibbonLargeButton extends StatefulWidget {
  const RibbonLargeButton({
    super.key,
    required this.icon,
    required this.label,
    this.tooltip,
    this.onPressed,
    this.onDropdown,
  });

  final IconData icon;
  final String label;
  final String? tooltip;
  final VoidCallback? onPressed;
  final VoidCallback? onDropdown;

  @override
  State<RibbonLargeButton> createState() => _RibbonLargeButtonState();
}

class _RibbonLargeButtonState extends State<RibbonLargeButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final color = enabled ? WordTheme.ribbonText : WordTheme.ribbonTextDisabled;
    final effectiveTooltip =
        effectiveRibbonTooltip(tooltip: widget.tooltip, enabled: enabled);

    return wrapRibbonTooltip(
      effectiveTooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            GestureDetector(
              onTap: widget.onPressed,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
                decoration: BoxDecoration(
                  color: _hovered && enabled ? WordTheme.ribbonHover : Colors.transparent,
                  borderRadius: const BorderRadius.horizontal(left: Radius.circular(3)),
                ),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(widget.icon, size: WordTheme.largeIconSize, color: color),
                    const SizedBox(height: 2),
                    Text(widget.label, style: WordTheme.ribbonLabel.copyWith(color: color)),
                  ],
                ),
              ),
            ),
            GestureDetector(
              onTap: widget.onDropdown ?? widget.onPressed,
              child: Container(
                width: 14,
                height: 52,
                decoration: BoxDecoration(
                  color: _hovered && enabled ? WordTheme.ribbonHover : Colors.transparent,
                  borderRadius: const BorderRadius.horizontal(right: Radius.circular(3)),
                ),
                child: Icon(Icons.arrow_drop_down, size: 14, color: color),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class RibbonDropdown extends StatefulWidget {
  const RibbonDropdown({
    super.key,
    required this.value,
    required this.width,
    required this.items,
    required this.onSelected,
    this.itemStyle,
  });

  final String value;
  final double width;
  final List<String> items;
  final ValueChanged<String> onSelected;
  final TextStyle Function(String item)? itemStyle;

  @override
  State<RibbonDropdown> createState() => _RibbonDropdownState();
}

class _RibbonDropdownState extends State<RibbonDropdown> {
  bool _hovered = false;

  Future<void> _openMenu() async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null || !box.hasSize) return;

    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final origin = box.localToGlobal(Offset.zero, ancestor: overlay);
    final selected = await showMenu<String>(
      context: context,
      position: RelativeRect.fromLTRB(
        origin.dx,
        origin.dy + box.size.height,
        origin.dx + box.size.width,
        origin.dy + box.size.height + 4,
      ),
      constraints: BoxConstraints(
        minWidth: widget.width,
        maxWidth: widget.width + 80,
        maxHeight: 280,
      ),
      items: widget.items
          .map(
            (item) => PopupMenuItem<String>(
              value: item,
              height: 28,
              child: Text(
                item,
                style: (widget.itemStyle?.call(item) ?? WordTheme.ribbonLabel).copyWith(
                  fontWeight: item == widget.value ? FontWeight.w600 : FontWeight.normal,
                ),
              ),
            ),
          )
          .toList(),
    );
    if (selected != null && selected != widget.value) {
      widget.onSelected(selected);
    }
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: _openMenu,
        child: Container(
          width: widget.width,
          height: 22,
          padding: const EdgeInsets.symmetric(horizontal: 6),
          decoration: BoxDecoration(
            color: _hovered ? WordTheme.ribbonHover : Colors.white,
            border: Border.all(color: WordTheme.groupDivider),
            borderRadius: BorderRadius.circular(2),
          ),
          child: Row(
            children: [
              Expanded(
                child: FittedBox(
                  fit: BoxFit.scaleDown,
                  alignment: Alignment.centerLeft,
                  child: Text(
                    widget.value,
                    style: WordTheme.ribbonLabel.copyWith(fontSize: 11),
                    maxLines: 1,
                  ),
                ),
              ),
              const Icon(Icons.arrow_drop_down, size: 14, color: WordTheme.ribbonText),
            ],
          ),
        ),
      ),
    );
  }
}

/// Common Word font list for the Home tab ribbon.
const kRibbonFontFamilies = [
  'Calibri',
  'Arial',
  'Cambria',
  'Times New Roman',
  'Georgia',
  'Verdana',
  'Tahoma',
  'Trebuchet MS',
  'Courier New',
  'Consolas',
];

/// Point size label for the Home tab font-size dropdown.
String formatRibbonFontSize(double size) {
  return size.round().clamp(1, 999).toString();
}

/// Common Word point sizes for the Home tab ribbon.
const kRibbonFontSizes = [
  '8',
  '9',
  '10',
  '11',
  '12',
  '14',
  '16',
  '18',
  '20',
  '22',
  '24',
  '26',
  '28',
  '36',
  '48',
  '72',
];

class StyleGalleryCard extends StatefulWidget {
  const StyleGalleryCard({
    super.key,
    required this.label,
    this.previewStyle,
    this.selected = false,
    this.tooltip,
    this.onPressed,
  });

  final String label;
  final TextStyle? previewStyle;
  final bool selected;
  final String? tooltip;
  final VoidCallback? onPressed;

  @override
  State<StyleGalleryCard> createState() => _StyleGalleryCardState();
}

class _StyleGalleryCardState extends State<StyleGalleryCard> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    Color borderColor = WordTheme.groupDivider;
    if (widget.selected) {
      borderColor = WordTheme.activeTabUnderline;
    } else if (_hovered && enabled) {
      borderColor = WordTheme.ribbonText;
    }

    final effectiveTooltip =
        effectiveRibbonTooltip(tooltip: widget.tooltip, enabled: enabled);

    return wrapRibbonTooltip(
      effectiveTooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            width: 72,
            height: 52,
            margin: const EdgeInsets.symmetric(horizontal: 2),
            padding: const EdgeInsets.all(4),
            decoration: BoxDecoration(
              color: widget.selected ? WordTheme.ribbonSelected : Colors.white,
              border: Border.all(color: borderColor),
              borderRadius: BorderRadius.circular(2),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'AaBbCcDd',
                  style: (widget.previewStyle ?? WordTheme.ribbonLabel).copyWith(fontSize: 9),
                  overflow: TextOverflow.ellipsis,
                ),
                const Spacer(),
                Text(
                  widget.label,
                  style: WordTheme.ribbonLabel.copyWith(fontSize: 9),
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class RibbonTextButton extends StatefulWidget {
  const RibbonTextButton({
    super.key,
    required this.label,
    this.icon,
    this.tooltip,
    this.onPressed,
  });

  final String label;
  final IconData? icon;
  final String? tooltip;
  final VoidCallback? onPressed;

  @override
  State<RibbonTextButton> createState() => _RibbonTextButtonState();
}

class _RibbonTextButtonState extends State<RibbonTextButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final color = enabled ? WordTheme.ribbonText : WordTheme.ribbonTextDisabled;
    final effectiveTooltip =
        effectiveRibbonTooltip(tooltip: widget.tooltip, enabled: enabled);

    return wrapRibbonTooltip(
      effectiveTooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: _hovered && enabled ? WordTheme.ribbonHover : Colors.transparent,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (widget.icon != null) ...[
                  Icon(widget.icon, size: WordTheme.iconSize, color: color),
                  const SizedBox(width: 4),
                ],
                Text(widget.label, style: WordTheme.ribbonLabel.copyWith(color: color)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

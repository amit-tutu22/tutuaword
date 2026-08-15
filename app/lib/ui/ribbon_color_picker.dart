import 'package:flutter/material.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Selection from the Word color palette — optional theme slot for theme-linked colors.
class WordColorSelection {
  const WordColorSelection(this.color, {this.themeSlot, this.themeVariant});

  final Color color;
  final String? themeSlot;
  final int? themeVariant;

  bool get isThemeColor => themeSlot != null && themeVariant != null;
}

/// Maps theme-color grid coordinates to engine theme slots (columns 0,1,4,5).
WordColorSelection? themeColorSelectionForPicker({
  required int column,
  required int row,
  required Color color,
}) {
  final slot = switch (column) {
    0 => 'Background1',
    1 => 'Text1',
    4 => 'Accent1',
    5 => 'Accent2',
    _ => null,
  };
  if (slot == null) return null;
  return WordColorSelection(color, themeSlot: slot, themeVariant: row.clamp(0, 5));
}

/// Office default theme accent colors (10 columns).
const kWordThemeBaseColors = <Color>[
  Color(0xFFFFFFFF),
  Color(0xFF000000),
  Color(0xFFE7E6E6),
  Color(0xFF44546A),
  Color(0xFF4472C4),
  Color(0xFFED7D31),
  Color(0xFFA5A5A5),
  Color(0xFFFFC000),
  Color(0xFF5B9BD5),
  Color(0xFF70AD47),
];

/// Word standard color row.
const kWordStandardColors = <Color>[
  Color(0xFFC00000),
  Color(0xFFFF0000),
  Color(0xFFFFC000),
  Color(0xFFFFFF00),
  Color(0xFF92D050),
  Color(0xFF00B050),
  Color(0xFF00B0F0),
  Color(0xFF0070C0),
  Color(0xFF002060),
  Color(0xFF7030A0),
];

/// Legacy exports used by tests and highlight mapping.
const kRibbonFontColors = kWordStandardColors;
const kRibbonHighlightColors = <Color>[
  Color(0xFFFFFF00),
  Color(0xFF00FF00),
  Color(0xFF00FFFF),
  Color(0xFFFF00FF),
  Color(0xFFFF0000),
  Color(0xFF0000FF),
];

Color _blend(Color base, Color target, double amount) {
  amount = amount.clamp(0.0, 1.0);
  int ch(double a, double b) => (a + (b - a) * amount).round().clamp(0, 255);
  return Color.fromARGB(
    255,
    ch(base.r * 255, target.r * 255),
    ch(base.g * 255, target.g * 255),
    ch(base.b * 255, target.b * 255),
  );
}

/// Six theme rows per column: three tints, base, two shades.
List<List<Color>> buildWordThemeColorGrid() {
  return List.generate(6, (row) {
    return List.generate(kWordThemeBaseColors.length, (column) {
      final base = kWordThemeBaseColors[column];
      return switch (row) {
        0 => _blend(base, Colors.white, 0.80),
        1 => _blend(base, Colors.white, 0.55),
        2 => _blend(base, Colors.white, 0.30),
        3 => base,
        4 => _blend(base, Colors.black, 0.25),
        _ => _blend(base, Colors.black, 0.50),
      };
    });
  });
}

/// Word-style theme + standard color palette panel.
class WordColorPalettePanel extends StatelessWidget {
  const WordColorPalettePanel({
    super.key,
    required this.onColorSelected,
    this.onAutomatic,
    this.onClear,
    this.automaticColor = Colors.black,
    this.automaticLabel = 'Automatic',
    this.clearLabel = 'No Color',
  });

  final ValueChanged<WordColorSelection> onColorSelected;
  final VoidCallback? onAutomatic;
  final VoidCallback? onClear;
  final Color automaticColor;
  final String automaticLabel;
  final String clearLabel;

  static const _cellSize = 15.0;
  static const _cellGap = 2.0;

  @override
  Widget build(BuildContext context) {
    final themeGrid = buildWordThemeColorGrid();
    return Material(
      elevation: 8,
      color: Colors.white,
      borderRadius: BorderRadius.circular(2),
      clipBehavior: Clip.antiAlias,
      child: Container(
        width: (_cellSize + _cellGap) * kWordThemeBaseColors.length + 16,
        padding: const EdgeInsets.fromLTRB(8, 6, 8, 6),
        decoration: BoxDecoration(
          border: Border.all(color: WordTheme.groupDivider),
          borderRadius: BorderRadius.circular(2),
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (onAutomatic != null) ...[
              _AutomaticRow(
                label: automaticLabel,
                barColor: automaticColor,
                onTap: onAutomatic!,
              ),
              const SizedBox(height: 4),
              _sectionDivider(),
            ],
            if (onClear != null) ...[
              _AutomaticRow(
                label: clearLabel,
                barColor: Colors.transparent,
                onTap: onClear!,
                showBar: false,
              ),
              const SizedBox(height: 4),
              _sectionDivider(),
            ],
            _sectionLabel('Theme Colors'),
            const SizedBox(height: 4),
            for (var rowIndex = 0; rowIndex < themeGrid.length; rowIndex++) ...[
              _themeColorRow(themeGrid[rowIndex], rowIndex),
              const SizedBox(height: _cellGap),
            ],
            const SizedBox(height: 4),
            _sectionDivider(),
            const SizedBox(height: 4),
            _sectionLabel('Standard Colors'),
            const SizedBox(height: 4),
            _colorRow(kWordStandardColors, onSelected: (color) {
              onColorSelected(WordColorSelection(color));
            }),
          ],
        ),
      ),
    );
  }

  Widget _sectionLabel(String text) {
    return Text(
      text,
      style: WordTheme.ribbonLabel.copyWith(
        fontSize: 10,
        color: const Color(0xFF666666),
      ),
    );
  }

  Widget _sectionDivider() {
    return Container(height: 1, color: WordTheme.groupDivider);
  }

  Widget _colorRow(List<Color> colors, {required ValueChanged<Color> onSelected}) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var i = 0; i < colors.length; i++) ...[
          if (i > 0) const SizedBox(width: _cellGap),
          _ColorSwatch(
            color: colors[i],
            size: _cellSize,
            onSelected: () => onSelected(colors[i]),
          ),
        ],
      ],
    );
  }

  Widget _themeColorRow(List<Color> colors, int rowIndex) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (var column = 0; column < colors.length; column++) ...[
          if (column > 0) const SizedBox(width: _cellGap),
          _ColorSwatch(
            color: colors[column],
            size: _cellSize,
            onSelected: () {
              final themed = themeColorSelectionForPicker(
                column: column,
                row: rowIndex,
                color: colors[column],
              );
              onColorSelected(
                themed ?? WordColorSelection(colors[column]),
              );
            },
          ),
        ],
      ],
    );
  }
}

class _AutomaticRow extends StatefulWidget {
  const _AutomaticRow({
    required this.label,
    required this.barColor,
    required this.onTap,
    this.showBar = true,
  });

  final String label;
  final Color barColor;
  final VoidCallback onTap;
  final bool showBar;

  @override
  State<_AutomaticRow> createState() => _AutomaticRowState();
}

class _AutomaticRowState extends State<_AutomaticRow> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        behavior: HitTestBehavior.opaque,
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 3),
          color: _hovered ? WordTheme.chrome(context).ribbonHover : Colors.transparent,
          child: Row(
            children: [
              SizedBox(
                width: 18,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      'A',
                      style: WordTheme.ribbonLabel.copyWith(
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                        height: 1.0,
                      ),
                    ),
                    if (widget.showBar)
                      Container(
                        width: 16,
                        height: 3,
                        color: widget.barColor,
                      )
                    else
                      Container(
                        width: 16,
                        height: 3,
                        decoration: BoxDecoration(
                          border: Border.all(color: WordTheme.groupDivider),
                        ),
                      ),
                  ],
                ),
              ),
              const SizedBox(width: 6),
              Text(widget.label, style: WordTheme.ribbonLabel.copyWith(fontSize: 11)),
            ],
          ),
        ),
      ),
    );
  }
}

class _ColorSwatch extends StatefulWidget {
  const _ColorSwatch({
    required this.color,
    required this.size,
    required this.onSelected,
  });

  final Color color;
  final double size;
  final VoidCallback onSelected;

  @override
  State<_ColorSwatch> createState() => _ColorSwatchState();
}

class _ColorSwatchState extends State<_ColorSwatch> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final isLight = widget.color.computeLuminance() > 0.85;
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onSelected,
        child: Container(
          width: widget.size,
          height: widget.size,
          decoration: BoxDecoration(
            color: widget.color,
            border: Border.all(
              color: _hovered
                  ? const Color(0xFF333333)
                  : (isLight ? WordTheme.groupDivider : widget.color),
              width: _hovered ? 1.5 : 1,
            ),
          ),
        ),
      ),
    );
  }
}

/// Font-color and text-highlight picker buttons for the Home ribbon.
class RibbonColorButton extends StatefulWidget {
  const RibbonColorButton({
    super.key,
    required this.icon,
    required this.tooltip,
    required this.barColor,
    required this.onColorSelected,
    this.onAutomatic,
    this.onClear,
    this.automaticColor = Colors.black,
  });

  final IconData icon;
  final String tooltip;
  final Color barColor;
  final ValueChanged<WordColorSelection> onColorSelected;
  final VoidCallback? onAutomatic;
  final VoidCallback? onClear;
  final Color automaticColor;

  @override
  State<RibbonColorButton> createState() => _RibbonColorButtonState();
}

class _RibbonColorButtonState extends State<RibbonColorButton> {
  bool _hovered = false;

  Future<void> _openPalette() async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    final origin = box.localToGlobal(Offset.zero);
    final panelBottom = origin.dy + box.size.height + 4;

    await showGeneralDialog<void>(
      context: context,
      barrierDismissible: true,
      barrierLabel: widget.tooltip,
      barrierColor: Colors.transparent,
      transitionDuration: Duration.zero,
      pageBuilder: (dialogContext, _, __) {
        return Stack(
          children: [
            Positioned(
              left: origin.dx,
              top: panelBottom,
              child: WordColorPalettePanel(
                automaticColor: widget.automaticColor,
                onColorSelected: (selection) {
                  Navigator.of(dialogContext).pop();
                  widget.onColorSelected(selection);
                },
                onAutomatic: widget.onAutomatic == null
                    ? null
                    : () {
                        Navigator.of(dialogContext).pop();
                        widget.onAutomatic!();
                      },
                onClear: widget.onClear == null
                    ? null
                    : () {
                        Navigator.of(dialogContext).pop();
                        widget.onClear!();
                      },
              ),
            ),
          ],
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final bg = _hovered ? WordTheme.chrome(context).ribbonHover : Colors.transparent;
    return wrapRibbonTooltip(
      widget.tooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: _openPalette,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: BoxDecoration(
              color: bg,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(widget.icon, size: WordTheme.iconSize, color: WordTheme.ribbonText),
                const SizedBox(height: 1),
                Container(
                  width: 16,
                  height: 3,
                  color: widget.barColor,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

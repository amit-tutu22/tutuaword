import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Paragraph shading + uniform borders (F04.S4).
class ParagraphBordersValues {
  const ParagraphBordersValues({
    this.shading,
    this.borderWidth = 0,
    this.borderColor = Colors.black,
    this.clearShading = false,
    this.clearBorders = false,
  });

  final Color? shading;
  final double borderWidth;
  final Color borderColor;
  final bool clearShading;
  final bool clearBorders;
}

class ParagraphBordersDialog extends StatefulWidget {
  const ParagraphBordersDialog({super.key, required this.initial});

  final ParagraphBordersValues initial;

  static Future<ParagraphBordersValues?> show(
    BuildContext context, {
    required ParagraphBordersValues initial,
  }) {
    return showDialog<ParagraphBordersValues>(
      context: context,
      builder: (context) => ParagraphBordersDialog(initial: initial),
    );
  }

  @override
  State<ParagraphBordersDialog> createState() => _ParagraphBordersDialogState();
}

class _ParagraphBordersDialogState extends State<ParagraphBordersDialog> {
  Color? _shading;
  late final TextEditingController _widthController;
  Color _borderColor = Colors.black;
  bool _hasBorder = false;

  @override
  void initState() {
    super.initState();
    _shading = widget.initial.shading;
    _borderColor = widget.initial.borderColor;
    _hasBorder = widget.initial.borderWidth > 0;
    _widthController = TextEditingController(
      text: widget.initial.borderWidth > 0
          ? widget.initial.borderWidth.toStringAsFixed(1)
          : '1',
    );
  }

  @override
  void dispose() {
    _widthController.dispose();
    super.dispose();
  }

  void _submit() {
    final width = double.tryParse(_widthController.text.trim()) ?? 1.0;
    Navigator.of(context).pop(
      ParagraphBordersValues(
        shading: _shading,
        borderWidth: _hasBorder ? width.clamp(0.5, 12) : 0,
        borderColor: _borderColor,
      ),
    );
  }

  void _clearAll() {
    Navigator.of(context).pop(
      const ParagraphBordersValues(clearShading: true, clearBorders: true),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Borders and Shading'),
      content: SizedBox(
        width: 380,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text('Shading', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final color in kWordThemeBaseColors.take(8))
                  _ColorSwatch(
                    color: color,
                    selected: _shading == color,
                    onTap: () => setState(() => _shading = color),
                  ),
                _ColorSwatch(
                  label: 'None',
                  selected: _shading == null,
                  onTap: () => setState(() => _shading = null),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text('Borders', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            CheckboxListTile(
              key: const Key('para_border_enabled'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Border on all sides'),
              value: _hasBorder,
              onChanged: (value) => setState(() => _hasBorder = value ?? false),
              controlAffinity: ListTileControlAffinity.leading,
            ),
            if (_hasBorder) ...[
              TextField(
                key: const Key('para_border_width'),
                controller: _widthController,
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                inputFormatters: [
                  FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
                ],
                decoration: const InputDecoration(
                  labelText: 'Width',
                  suffixText: 'pt',
                  isDense: true,
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 8),
              Wrap(
                spacing: 4,
                children: [
                  for (final color in kWordStandardColors.take(6))
                    _ColorSwatch(
                      color: color,
                      selected: _borderColor == color,
                      onTap: () => setState(() => _borderColor = color),
                    ),
                ],
              ),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('borders_dialog_clear'),
          onPressed: _clearAll,
          child: const Text('Clear'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('borders_dialog_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }
}

class _ColorSwatch extends StatelessWidget {
  const _ColorSwatch({
    this.color,
    this.label,
    required this.selected,
    required this.onTap,
  });

  final Color? color;
  final String? label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        width: 28,
        height: 28,
        decoration: BoxDecoration(
          color: color ?? Colors.transparent,
          border: Border.all(
            color: selected ? WordTheme.activeTabUnderline : Colors.black26,
            width: selected ? 2 : 1,
          ),
        ),
        child: label != null
            ? Center(
                child: Text(label!, style: const TextStyle(fontSize: 8)),
              )
            : null,
      ),
    );
  }
}

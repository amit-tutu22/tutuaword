import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Table border, cell shading, column width, and AutoFit (F09.S4).
class TableDesignValues {
  const TableDesignValues({
    this.tableBorderWidth = 0,
    this.tableBorderColor = Colors.black,
    this.cellShading,
    this.columnWidth,
    this.autofitToWindow = false,
    this.clearTableBorder = false,
    this.clearCellShading = false,
  });

  final double tableBorderWidth;
  final Color tableBorderColor;
  final Color? cellShading;
  final double? columnWidth;
  final bool autofitToWindow;
  final bool clearTableBorder;
  final bool clearCellShading;
}

class TableDesignDialog extends StatefulWidget {
  const TableDesignDialog({super.key, required this.initial});

  final TableDesignValues initial;

  static Future<TableDesignValues?> show(
    BuildContext context, {
    required TableDesignValues initial,
  }) {
    return showDialog<TableDesignValues>(
      context: context,
      builder: (context) => TableDesignDialog(initial: initial),
    );
  }

  @override
  State<TableDesignDialog> createState() => _TableDesignDialogState();
}

class _TableDesignDialogState extends State<TableDesignDialog> {
  Color? _cellShading;
  late final TextEditingController _borderWidthController;
  late final TextEditingController _columnWidthController;
  Color _borderColor = Colors.black;
  bool _hasBorder = false;

  @override
  void initState() {
    super.initState();
    _cellShading = widget.initial.cellShading;
    _borderColor = widget.initial.tableBorderColor;
    _hasBorder = widget.initial.tableBorderWidth > 0;
    _borderWidthController = TextEditingController(
      text: widget.initial.tableBorderWidth > 0
          ? widget.initial.tableBorderWidth.toStringAsFixed(1)
          : '1',
    );
    _columnWidthController = TextEditingController(
      text: widget.initial.columnWidth?.toStringAsFixed(1) ?? '100',
    );
  }

  @override
  void dispose() {
    _borderWidthController.dispose();
    _columnWidthController.dispose();
    super.dispose();
  }

  void _submit() {
    final borderWidth = double.tryParse(_borderWidthController.text.trim()) ?? 1.0;
    final columnText = _columnWidthController.text.trim();
    final columnWidth = columnText.isEmpty ? null : double.tryParse(columnText);
    Navigator.of(context).pop(
      TableDesignValues(
        tableBorderWidth: _hasBorder ? borderWidth.clamp(0.5, 12) : 0,
        tableBorderColor: _borderColor,
        cellShading: _cellShading,
        columnWidth: columnWidth,
      ),
    );
  }

  void _autofit() {
    Navigator.of(context).pop(
      const TableDesignValues(autofitToWindow: true),
    );
  }

  void _clearAll() {
    Navigator.of(context).pop(
      const TableDesignValues(clearTableBorder: true, clearCellShading: true),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Table Design'),
      content: SizedBox(
        width: 400,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
            Text('Cell shading', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final color in kWordThemeBaseColors.take(8))
                  _ColorSwatch(
                    color: color,
                    selected: _cellShading == color,
                    onTap: () => setState(() => _cellShading = color),
                  ),
                _ColorSwatch(
                  label: 'None',
                  selected: _cellShading == null,
                  onTap: () => setState(() => _cellShading = null),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text('Table border', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            CheckboxListTile(
              key: const Key('table_border_enabled'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Border on all sides'),
              value: _hasBorder,
              onChanged: (v) => setState(() => _hasBorder = v ?? false),
            ),
            if (_hasBorder) ...[
              TextField(
                key: const Key('table_border_width'),
                controller: _borderWidthController,
                decoration: const InputDecoration(labelText: 'Border width (pt)'),
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                inputFormatters: [
                  FilteringTextInputFormatter.allow(RegExp(r'[\d.]')),
                ],
              ),
              const SizedBox(height: 8),
              Wrap(
                spacing: 4,
                runSpacing: 4,
                children: [
                  for (final color in kWordThemeBaseColors.take(6))
                    _ColorSwatch(
                      color: color,
                      selected: _borderColor == color,
                      onTap: () => setState(() => _borderColor = color),
                    ),
                ],
              ),
            ],
            const SizedBox(height: 16),
            Text('Column width', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            TextField(
              key: const Key('table_column_width'),
              controller: _columnWidthController,
              decoration: const InputDecoration(labelText: 'Width (pt) for current column'),
              keyboardType: const TextInputType.numberWithOptions(decimal: true),
              inputFormatters: [
                FilteringTextInputFormatter.allow(RegExp(r'[\d.]')),
              ],
            ),
            const SizedBox(height: 16),
            OutlinedButton(
              key: const Key('table_autofit_window'),
              onPressed: _autofit,
              child: const Text('AutoFit to Window'),
            ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(onPressed: _clearAll, child: const Text('Clear')),
        TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
        FilledButton(
          key: const Key('table_design_ok'),
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
    this.selected = false,
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
            color: selected ? Theme.of(context).colorScheme.primary : Colors.grey,
            width: selected ? 2 : 1,
          ),
        ),
        alignment: Alignment.center,
        child: label != null
            ? Text(label!, style: const TextStyle(fontSize: 8))
            : null,
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Values for Design/Layout → Page Borders.
class PageBordersValues {
  const PageBordersValues({
    this.enabled = false,
    this.width = 1.0,
    this.color = Colors.black,
    this.clear = false,
  });

  final bool enabled;
  final double width;
  final Color color;
  final bool clear;
}

/// Configure a uniform page box border.
class PageBordersDialog extends StatefulWidget {
  const PageBordersDialog({super.key, required this.initial});

  final PageBordersValues initial;

  static Future<PageBordersValues?> show(
    BuildContext context, {
    required PageBordersValues initial,
  }) {
    return showDialog<PageBordersValues>(
      context: context,
      builder: (context) => PageBordersDialog(initial: initial),
    );
  }

  @override
  State<PageBordersDialog> createState() => _PageBordersDialogState();
}

class _PageBordersDialogState extends State<PageBordersDialog> {
  late bool _enabled;
  late final TextEditingController _widthController;
  late Color _color;

  @override
  void initState() {
    super.initState();
    _enabled = widget.initial.enabled;
    _color = widget.initial.color;
    _widthController = TextEditingController(
      text: widget.initial.width > 0
          ? widget.initial.width.toStringAsFixed(1)
          : '1.0',
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
      PageBordersValues(
        enabled: _enabled,
        width: width.clamp(0.5, 12),
        color: _color,
        clear: !_enabled,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('page_borders_dialog'),
      title: const Text('Page Borders'),
      content: SizedBox(
        width: 380,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            CheckboxListTile(
              key: const Key('page_borders_enabled'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Box border on all sides'),
              value: _enabled,
              onChanged: (v) => setState(() => _enabled = v ?? false),
            ),
            const SizedBox(height: 8),
            TextField(
              key: const Key('page_borders_width'),
              controller: _widthController,
              enabled: _enabled,
              keyboardType:
                  const TextInputType.numberWithOptions(decimal: true),
              inputFormatters: [
                FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
              ],
              decoration: const InputDecoration(
                labelText: 'Width (pt)',
                isDense: true,
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            Text(
              'Color',
              style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600),
            ),
            const SizedBox(height: 8),
            Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final color in kWordThemeBaseColors.take(8))
                  _Swatch(
                    color: color,
                    selected: _color == color,
                    enabled: _enabled,
                    onTap: () => setState(() => _color = color),
                  ),
              ],
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('page_borders_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        TextButton(
          key: const Key('page_borders_none'),
          onPressed: () => Navigator.of(context).pop(
            const PageBordersValues(clear: true),
          ),
          child: const Text('None'),
        ),
        FilledButton(
          key: const Key('page_borders_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }
}

class _Swatch extends StatelessWidget {
  const _Swatch({
    required this.color,
    required this.selected,
    required this.enabled,
    required this.onTap,
  });

  final Color color;
  final bool selected;
  final bool enabled;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: enabled ? onTap : null,
      child: Container(
        width: 22,
        height: 22,
        decoration: BoxDecoration(
          color: color,
          border: Border.all(
            color: selected ? WordTheme.activeTabUnderline : WordTheme.groupDivider,
            width: selected ? 2 : 1,
          ),
        ),
      ),
    );
  }
}

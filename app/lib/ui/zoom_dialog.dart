import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Zoom percentage picker for View → Zoom.
class ZoomDialog extends StatefulWidget {
  const ZoomDialog({super.key, required this.initialPercent});

  final int initialPercent;

  static Future<double?> show(BuildContext context, {required double currentZoom}) {
    return showDialog<double>(
      context: context,
      builder: (context) => ZoomDialog(
        initialPercent: (currentZoom * 100).round(),
      ),
    );
  }

  @override
  State<ZoomDialog> createState() => _ZoomDialogState();
}

class _ZoomDialogState extends State<ZoomDialog> {
  static const _presets = [50, 75, 100, 125, 150, 200];
  late final TextEditingController _custom;
  late int _selected;

  @override
  void initState() {
    super.initState();
    _selected = widget.initialPercent;
    _custom = TextEditingController(text: '${widget.initialPercent}');
  }

  @override
  void dispose() {
    _custom.dispose();
    super.dispose();
  }

  void _submit([int? percent]) {
    final value = percent ?? int.tryParse(_custom.text.trim());
    if (value == null) return;
    final clamped = value.clamp(50, 300);
    Navigator.of(context).pop(clamped / 100.0);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('zoom_dialog'),
      title: const Text('Zoom'),
      content: SizedBox(
        width: 280,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (final p in _presets)
              RadioListTile<int>(
                key: Key('zoom_preset_$p'),
                dense: true,
                title: Text('$p%'),
                value: p,
                groupValue: _presets.contains(_selected) ? _selected : null,
                onChanged: (v) {
                  if (v == null) return;
                  setState(() {
                    _selected = v;
                    _custom.text = '$v';
                  });
                },
              ),
            const SizedBox(height: 8),
            TextField(
              key: const Key('zoom_custom'),
              controller: _custom,
              keyboardType: TextInputType.number,
              inputFormatters: [FilteringTextInputFormatter.digitsOnly],
              decoration: const InputDecoration(
                labelText: 'Percent',
                suffixText: '%',
                border: OutlineInputBorder(),
              ),
              onSubmitted: (_) => _submit(),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('zoom_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('zoom_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }
}

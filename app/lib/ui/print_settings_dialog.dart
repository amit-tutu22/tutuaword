import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Scale + margins + print scope chooser shown before the OS print dialog (F25.S2/S3).
class PrintSettingsDialog extends StatefulWidget {
  const PrintSettingsDialog({
    super.key,
    this.initial = PrintLayoutSettings.defaults,
    this.selectionAvailable = false,
  });

  final PrintLayoutSettings initial;
  final bool selectionAvailable;

  static Future<PrintLayoutSettings?> show(
    BuildContext context, {
    PrintLayoutSettings initial = PrintLayoutSettings.defaults,
    bool selectionAvailable = false,
  }) {
    return showDialog<PrintLayoutSettings>(
      context: context,
      barrierDismissible: true,
      builder: (context) => PrintSettingsDialog(
        initial: initial,
        selectionAvailable: selectionAvailable,
      ),
    );
  }

  @override
  State<PrintSettingsDialog> createState() => _PrintSettingsDialogState();
}

class _PrintSettingsDialogState extends State<PrintSettingsDialog> {
  late PrintScaleMode _mode;
  late double _scalePercent;
  late String _marginPreset;
  late double _customMargin;
  late PrintScope _scope;
  late PrintDuplexMode _duplex;
  late int _pagesPerSheet;
  late bool _booklet;

  static const _marginPresets = <String, double>{
    'None': 0,
    'Narrow': 18,
    'Normal': 36,
    'Wide': 72,
  };

  static const _nupChoices = <int>[1, 2, 4, 6, 9, 16];

  @override
  void initState() {
    super.initState();
    _mode = widget.initial.scaleMode;
    _scalePercent = widget.initial.scalePercent;
    final m = widget.initial.marginLeft;
    String? matched;
    for (final e in _marginPresets.entries) {
      if ((e.value - m).abs() < 0.5) {
        matched = e.key;
        break;
      }
    }
    _marginPreset = matched ?? 'Custom';
    _customMargin = m;
    _scope = widget.selectionAvailable &&
            widget.initial.scope == PrintScope.selection
        ? PrintScope.selection
        : PrintScope.document;
    _duplex = widget.initial.duplex;
    _pagesPerSheet =
        PrintLayoutSettings.normalizePagesPerSheet(widget.initial.pagesPerSheet);
    _booklet = widget.initial.booklet;
  }

  PrintLayoutSettings _build() {
    final margin = _marginPreset == 'Custom'
        ? _customMargin
        : (_marginPresets[_marginPreset] ?? 0);
    return PrintLayoutSettings(
      scaleMode: _mode,
      scalePercent: _scalePercent,
      marginLeft: margin,
      marginRight: margin,
      marginTop: margin,
      marginBottom: margin,
      scope: _scope,
      duplex: _duplex,
      pagesPerSheet: _pagesPerSheet,
      booklet: _booklet,
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('print_settings_dialog'),
      title: const Text('Print'),
      content: SizedBox(
        width: 360,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
            const Text('Pages', style: WordTheme.ribbonGroupLabel),
            const SizedBox(height: 6),
            SegmentedButton<PrintScope>(
              key: const Key('print_scope'),
              segments: [
                const ButtonSegment(
                  value: PrintScope.document,
                  label: Text('Document'),
                ),
                ButtonSegment(
                  value: PrintScope.selection,
                  label: const Text('Selection'),
                  enabled: widget.selectionAvailable,
                ),
              ],
              selected: {_scope},
              onSelectionChanged: (next) {
                if (next.isEmpty) return;
                setState(() => _scope = next.first);
              },
            ),
            if (!widget.selectionAvailable) ...[
              const SizedBox(height: 4),
              Text(
                'Select text to enable Print Selection',
                key: const Key('print_scope_hint'),
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
            const SizedBox(height: 14),
            const Text('Duplex', style: WordTheme.ribbonGroupLabel),
            const SizedBox(height: 6),
            DropdownButtonFormField<PrintDuplexMode>(
              key: const Key('print_duplex'),
              value: _booklet ? PrintDuplexMode.longEdge : _duplex,
              decoration: const InputDecoration(
                isDense: true,
                border: OutlineInputBorder(),
              ),
              items: const [
                DropdownMenuItem(
                  value: PrintDuplexMode.simplex,
                  child: Text('One-sided'),
                ),
                DropdownMenuItem(
                  value: PrintDuplexMode.longEdge,
                  child: Text('Long edge'),
                ),
                DropdownMenuItem(
                  value: PrintDuplexMode.shortEdge,
                  child: Text('Short edge'),
                ),
              ],
              onChanged: _booklet
                  ? null
                  : (value) {
                      if (value == null) return;
                      setState(() => _duplex = value);
                    },
            ),
            const SizedBox(height: 14),
            const Text('Pages per sheet', style: WordTheme.ribbonGroupLabel),
            const SizedBox(height: 6),
            DropdownButtonFormField<int>(
              key: const Key('print_pages_per_sheet'),
              value: _booklet ? 2 : _pagesPerSheet,
              decoration: const InputDecoration(
                isDense: true,
                border: OutlineInputBorder(),
              ),
              items: [
                for (final n in _nupChoices)
                  DropdownMenuItem(value: n, child: Text('$n')),
              ],
              onChanged: _booklet
                  ? null
                  : (value) {
                      if (value == null) return;
                      setState(() => _pagesPerSheet = value);
                    },
            ),
            const SizedBox(height: 8),
            CheckboxListTile(
              key: const Key('print_booklet'),
              contentPadding: EdgeInsets.zero,
              dense: true,
              title: const Text('Booklet'),
              value: _booklet,
              onChanged: (value) {
                setState(() {
                  _booklet = value ?? false;
                  if (_booklet) {
                    _duplex = PrintDuplexMode.longEdge;
                    _pagesPerSheet = 2;
                  }
                });
              },
            ),
            const SizedBox(height: 14),
            const Text('Scale', style: WordTheme.ribbonGroupLabel),
            const SizedBox(height: 6),
            SegmentedButton<PrintScaleMode>(
              key: const Key('print_scale_mode'),
              segments: const [
                ButtonSegment(
                  value: PrintScaleMode.actualSize,
                  label: Text('100%'),
                ),
                ButtonSegment(
                  value: PrintScaleMode.fitToMargins,
                  label: Text('Fit'),
                ),
                ButtonSegment(
                  value: PrintScaleMode.customPercent,
                  label: Text('Custom'),
                ),
              ],
              selected: {_mode},
              onSelectionChanged: (next) {
                if (next.isEmpty) return;
                setState(() => _mode = next.first);
              },
            ),
            if (_mode == PrintScaleMode.customPercent) ...[
              const SizedBox(height: 10),
              Row(
                children: [
                  const Text('Percent'),
                  Expanded(
                    child: Slider(
                      key: const Key('print_scale_slider'),
                      value: _scalePercent.clamp(10, 400),
                      min: 10,
                      max: 400,
                      divisions: 39,
                      label: '${_scalePercent.round()}%',
                      onChanged: (v) => setState(() => _scalePercent = v),
                    ),
                  ),
                  SizedBox(
                    width: 48,
                    child: Text(
                      key: const Key('print_scale_label'),
                      '${_scalePercent.round()}%',
                      textAlign: TextAlign.end,
                    ),
                  ),
                ],
              ),
            ],
            const SizedBox(height: 14),
            const Text('Margins', style: WordTheme.ribbonGroupLabel),
            const SizedBox(height: 6),
            DropdownButtonFormField<String>(
              key: const Key('print_margin_preset'),
              value: _marginPreset,
              decoration: const InputDecoration(
                isDense: true,
                border: OutlineInputBorder(),
              ),
              items: [
                for (final name in [..._marginPresets.keys, 'Custom'])
                  DropdownMenuItem(value: name, child: Text(name)),
              ],
              onChanged: (value) {
                if (value == null) return;
                setState(() => _marginPreset = value);
              },
            ),
            if (_marginPreset == 'Custom') ...[
              const SizedBox(height: 10),
              Row(
                children: [
                  const Text('Points'),
                  Expanded(
                    child: Slider(
                      key: const Key('print_margin_slider'),
                      value: _customMargin.clamp(0, 144),
                      min: 0,
                      max: 144,
                      divisions: 24,
                      label: _customMargin.round().toString(),
                      onChanged: (v) => setState(() => _customMargin = v),
                    ),
                  ),
                ],
              ),
            ],
          ],
          ),
        ),
      ),
      actions: [
        TextButton(
          key: const Key('print_settings_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        TextButton(
          key: const Key('print_settings_print'),
          onPressed: () => Navigator.of(context).pop(_build()),
          child: const Text('Print'),
        ),
      ],
    );
  }
}

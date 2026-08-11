import 'package:flutter/material.dart';
import 'package:tutuaword/editor/equation_omml.dart';
import 'package:tutuaword/editor/latex_omml.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Word-like Insert Equation dialog with symbol palette (F14.S3).
class EquationDialog extends StatefulWidget {
  const EquationDialog({super.key, required this.initial});

  final EquationModel initial;

  static Future<EquationModel?> show(
    BuildContext context, {
    required EquationModel initial,
  }) {
    return showDialog<EquationModel>(
      context: context,
      barrierDismissible: false,
      builder: (context) => EquationDialog(initial: initial),
    );
  }

  @override
  State<EquationDialog> createState() => _EquationDialogState();
}

class _EquationDialogState extends State<EquationDialog> {
  late EquationKind _kind;
  late TextEditingController _text;
  late TextEditingController _numerator;
  late TextEditingController _denominator;
  late TextEditingController _base;
  late TextEditingController _superscript;
  late TextEditingController _subscript;
  late TextEditingController _latex;
  late bool _display;
  String? _error;

  @override
  void initState() {
    super.initState();
    _kind = widget.initial.kind;
    _display = widget.initial.display;
    _text = TextEditingController(text: widget.initial.text);
    _numerator = TextEditingController(text: widget.initial.numerator);
    _denominator = TextEditingController(text: widget.initial.denominator);
    _base = TextEditingController(text: widget.initial.base);
    _superscript = TextEditingController(text: widget.initial.superscript);
    _subscript = TextEditingController(text: widget.initial.subscript);
    _latex = TextEditingController(text: widget.initial.latex);
  }

  @override
  void dispose() {
    _text.dispose();
    _numerator.dispose();
    _denominator.dispose();
    _base.dispose();
    _superscript.dispose();
    _subscript.dispose();
    _latex.dispose();
    super.dispose();
  }

  EquationModel currentModel() {
    return widget.initial.withFields(
      kind: _kind,
      text: _text.text,
      numerator: _numerator.text,
      denominator: _denominator.text,
      base: _base.text,
      superscript: _superscript.text,
      subscript: _subscript.text,
      latex: _latex.text,
      display: _display,
    );
  }

  void _insertSymbol(String symbol) {
    if (_kind == EquationKind.latex) {
      final insert = EquationOmml.latexCommandForSymbol(symbol) ?? symbol;
      final controller = _latex;
      final selection = controller.selection;
      final value = controller.text;
      final start = selection.start >= 0 ? selection.start : value.length;
      final end = selection.end >= 0 ? selection.end : value.length;
      final next = value.replaceRange(start, end, insert);
      controller
        ..text = next
        ..selection = TextSelection.collapsed(offset: start + insert.length);
      setState(() => _error = null);
      return;
    }
    final controller = switch (_kind) {
      EquationKind.plain || EquationKind.squareRoot => _text,
      EquationKind.fraction => _numerator,
      EquationKind.superscript || EquationKind.subscript => _base,
      EquationKind.latex => _latex,
    };
    final selection = controller.selection;
    final value = controller.text;
    final start = selection.start >= 0 ? selection.start : value.length;
    final end = selection.end >= 0 ? selection.end : value.length;
    final next = value.replaceRange(start, end, symbol);
    controller
      ..text = next
      ..selection = TextSelection.collapsed(offset: start + symbol.length);
    setState(() => _error = null);
  }

  void _submit() {
    final model = currentModel();
    if (!model.isValid) {
      setState(() {
        _error = model.kind == EquationKind.latex
            ? (LatexOmml.lastError ?? 'Invalid LaTeX')
            : 'Fill in all equation fields.';
      });
      return;
    }
    Navigator.pop(context, model);
  }

  @override
  Widget build(BuildContext context) {
    final preview = EquationOmml.previewText(currentModel());
    return AlertDialog(
      key: const Key('equation_dialog'),
      title: const Text('Insert Equation'),
      content: SizedBox(
        width: 420,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
            SegmentedButton<EquationKind>(
              segments: const [
                ButtonSegment(
                  value: EquationKind.plain,
                  label: Text('Text'),
                  icon: Icon(Icons.text_fields, size: 16),
                ),
                ButtonSegment(
                  value: EquationKind.fraction,
                  label: Text('Fraction'),
                  icon: Icon(Icons.horizontal_split, size: 16),
                ),
                ButtonSegment(
                  value: EquationKind.superscript,
                  label: Text('Sup'),
                  icon: Icon(Icons.superscript, size: 16),
                ),
                ButtonSegment(
                  value: EquationKind.subscript,
                  label: Text('Sub'),
                  icon: Icon(Icons.subscript, size: 16),
                ),
                ButtonSegment(
                  value: EquationKind.squareRoot,
                  label: Text('√'),
                ),
                ButtonSegment(
                  value: EquationKind.latex,
                  label: Text('LaTeX'),
                  icon: Icon(Icons.code, size: 16),
                ),
              ],
              selected: {_kind},
              onSelectionChanged: (next) {
                setState(() {
                  _kind = next.first;
                  _error = null;
                });
              },
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [
                for (final entry in EquationOmml.palette)
                  OutlinedButton(
                    key: Key('equation_symbol_${entry.label}'),
                    style: OutlinedButton.styleFrom(
                      minimumSize: const Size(36, 32),
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                    ),
                    onPressed: () => _insertSymbol(entry.insert),
                    child: Text(entry.label, style: const TextStyle(fontSize: 16)),
                  ),
              ],
            ),
            const SizedBox(height: 12),
            ..._structureFields(),
            SwitchListTile(
              key: const Key('equation_display_toggle'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Display equation (centered block)'),
              value: _display,
              onChanged: (value) => setState(() => _display = value),
            ),
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: const Color(0xFFE8EEF7),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: WordTheme.groupDivider),
              ),
              child: Text(
                preview.isEmpty ? 'Preview' : preview,
                key: const Key('equation_preview'),
                style: const TextStyle(
                  fontSize: 18,
                  fontStyle: FontStyle.italic,
                ),
                textAlign: _display ? TextAlign.center : TextAlign.start,
              ),
            ),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(
                _error!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
          ],
        ),
      ),
      ),
      actions: [
        TextButton(
          key: const Key('equation_cancel'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('equation_ok'),
          onPressed: _submit,
          child: const Text('Insert'),
        ),
      ],
    );
  }

  List<Widget> _structureFields() {
    switch (_kind) {
      case EquationKind.plain:
        return [
          TextField(
            key: const Key('equation_text'),
            controller: _text,
            decoration: const InputDecoration(
              labelText: 'Equation',
              hintText: 'E=mc²',
            ),
            autofocus: true,
          ),
        ];
      case EquationKind.fraction:
        return [
          TextField(
            key: const Key('equation_numerator'),
            controller: _numerator,
            decoration: const InputDecoration(labelText: 'Numerator'),
          ),
          const SizedBox(height: 8),
          TextField(
            key: const Key('equation_denominator'),
            controller: _denominator,
            decoration: const InputDecoration(labelText: 'Denominator'),
          ),
        ];
      case EquationKind.superscript:
        return [
          TextField(
            key: const Key('equation_base'),
            controller: _base,
            decoration: const InputDecoration(labelText: 'Base'),
          ),
          const SizedBox(height: 8),
          TextField(
            key: const Key('equation_superscript'),
            controller: _superscript,
            decoration: const InputDecoration(labelText: 'Superscript'),
          ),
        ];
      case EquationKind.subscript:
        return [
          TextField(
            key: const Key('equation_base'),
            controller: _base,
            decoration: const InputDecoration(labelText: 'Base'),
          ),
          const SizedBox(height: 8),
          TextField(
            key: const Key('equation_subscript'),
            controller: _subscript,
            decoration: const InputDecoration(labelText: 'Subscript'),
          ),
        ];
      case EquationKind.squareRoot:
        return [
          TextField(
            key: const Key('equation_radical'),
            controller: _text,
            decoration: const InputDecoration(labelText: 'Radicand'),
          ),
        ];
      case EquationKind.latex:
        return [
          TextField(
            key: const Key('equation_latex'),
            controller: _latex,
            decoration: const InputDecoration(
              labelText: 'LaTeX',
              hintText: r'\frac{\pi}{2}',
            ),
            maxLines: 3,
            autofocus: true,
          ),
        ];
    }
  }
}

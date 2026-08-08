import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/controllers/formatting_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Result of [ParagraphSpacingDialog] — applied via [FormattingController.applySpacing].
class ParagraphSpacingValues {
  const ParagraphSpacingValues({
    required this.lineSpacing,
    required this.exactPoints,
    required this.spaceBefore,
    required this.spaceAfter,
    required this.keepTogether,
    required this.keepWithNext,
    required this.widowOrphanControl,
  });

  final LineSpacingMode lineSpacing;
  final double exactPoints;
  final double spaceBefore;
  final double spaceAfter;
  final bool keepTogether;
  final bool keepWithNext;
  final bool widowOrphanControl;
}

/// Paragraph spacing: line rule (Single / 1.5 / Double / Exact) and space before/after.
class ParagraphSpacingDialog extends StatefulWidget {
  const ParagraphSpacingDialog({
    super.key,
    required this.initial,
  });

  final ParagraphSpacingValues initial;

  static Future<ParagraphSpacingValues?> show(
    BuildContext context, {
    required ParagraphSpacingValues initial,
  }) {
    return showDialog<ParagraphSpacingValues>(
      context: context,
      builder: (context) => ParagraphSpacingDialog(initial: initial),
    );
  }

  @override
  State<ParagraphSpacingDialog> createState() => _ParagraphSpacingDialogState();
}

class _ParagraphSpacingDialogState extends State<ParagraphSpacingDialog> {
  late LineSpacingMode _mode;
  late final TextEditingController _exactController;
  late final TextEditingController _beforeController;
  late final TextEditingController _afterController;
  late bool _keepTogether;
  late bool _keepWithNext;
  late bool _widowOrphanControl;

  @override
  void initState() {
    super.initState();
    _mode = widget.initial.lineSpacing;
    _keepTogether = widget.initial.keepTogether;
    _keepWithNext = widget.initial.keepWithNext;
    _widowOrphanControl = widget.initial.widowOrphanControl;
    _exactController = TextEditingController(
      text: _formatPt(widget.initial.exactPoints),
    );
    _beforeController = TextEditingController(
      text: _formatPt(widget.initial.spaceBefore),
    );
    _afterController = TextEditingController(
      text: _formatPt(widget.initial.spaceAfter),
    );
  }

  @override
  void dispose() {
    _exactController.dispose();
    _beforeController.dispose();
    _afterController.dispose();
    super.dispose();
  }

  static String _formatPt(double value) {
    if (value == value.roundToDouble()) return value.round().toString();
    return value.toStringAsFixed(1);
  }

  double _parsePt(TextEditingController c, double fallback) {
    final parsed = double.tryParse(c.text.trim());
    if (parsed == null || parsed.isNaN || parsed.isInfinite) return fallback;
    return parsed.clamp(0, 240).toDouble();
  }

  void _submit() {
    Navigator.of(context).pop(
      ParagraphSpacingValues(
        lineSpacing: _mode,
        exactPoints: _parsePt(_exactController, widget.initial.exactPoints),
        spaceBefore: _parsePt(_beforeController, widget.initial.spaceBefore),
        spaceAfter: _parsePt(_afterController, widget.initial.spaceAfter),
        keepTogether: _keepTogether,
        keepWithNext: _keepWithNext,
        widowOrphanControl: _widowOrphanControl,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Paragraph Spacing'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text('Line spacing', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 6),
            DropdownButtonFormField<LineSpacingMode>(
              key: const Key('line_spacing_mode'),
              initialValue: _mode,
              items: const [
                DropdownMenuItem(value: LineSpacingMode.single, child: Text('Single')),
                DropdownMenuItem(value: LineSpacingMode.oneAndHalf, child: Text('1.5 lines')),
                DropdownMenuItem(value: LineSpacingMode.double_, child: Text('Double')),
                DropdownMenuItem(value: LineSpacingMode.exact, child: Text('Exactly')),
              ],
              onChanged: (value) {
                if (value == null) return;
                setState(() => _mode = value);
              },
            ),
            if (_mode == LineSpacingMode.exact) ...[
              const SizedBox(height: 12),
              Text('At', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
              const SizedBox(height: 6),
              TextField(
                key: const Key('exact_points'),
                controller: _exactController,
                keyboardType: const TextInputType.numberWithOptions(decimal: true),
                inputFormatters: [
                  FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
                ],
                decoration: const InputDecoration(
                  suffixText: 'pt',
                  isDense: true,
                  border: OutlineInputBorder(),
                ),
              ),
            ],
            const SizedBox(height: 16),
            Text('Spacing', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('space_before'),
                    controller: _beforeController,
                    keyboardType: const TextInputType.numberWithOptions(decimal: true),
                    inputFormatters: [
                      FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
                    ],
                    decoration: const InputDecoration(
                      labelText: 'Before',
                      suffixText: 'pt',
                      isDense: true,
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    key: const Key('space_after'),
                    controller: _afterController,
                    keyboardType: const TextInputType.numberWithOptions(decimal: true),
                    inputFormatters: [
                      FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
                    ],
                    decoration: const InputDecoration(
                      labelText: 'After',
                      suffixText: 'pt',
                      isDense: true,
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text('Pagination', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 4),
            CheckboxListTile(
              key: const Key('keep_together'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Keep lines together'),
              value: _keepTogether,
              onChanged: (value) => setState(() => _keepTogether = value ?? false),
              controlAffinity: ListTileControlAffinity.leading,
            ),
            CheckboxListTile(
              key: const Key('keep_with_next'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Keep with next'),
              value: _keepWithNext,
              onChanged: (value) => setState(() => _keepWithNext = value ?? false),
              controlAffinity: ListTileControlAffinity.leading,
            ),
            CheckboxListTile(
              key: const Key('widow_orphan'),
              contentPadding: EdgeInsets.zero,
              title: const Text('Widow/orphan control'),
              value: _widowOrphanControl,
              onChanged: (value) => setState(() => _widowOrphanControl = value ?? true),
              controlAffinity: ListTileControlAffinity.leading,
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('spacing_dialog_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }
}

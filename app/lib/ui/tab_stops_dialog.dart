import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// One explicit paragraph tab stop (points from the left margin).
class TabStopValue {
  const TabStopValue({
    required this.position,
    this.alignment = 'Left',
  });

  final double position;

  /// Serde name: Left / Center / Right / Decimal / Bar.
  final String alignment;

  Map<String, dynamic> toJson() => {
        'position': position,
        'alignment': alignment,
      };

  static TabStopValue? fromJson(dynamic raw) {
    if (raw is! Map) return null;
    final pos = raw['position'];
    if (pos is! num) return null;
    final align = raw['alignment'];
    return TabStopValue(
      position: pos.toDouble(),
      alignment: align is String && align.isNotEmpty ? align : 'Left',
    );
  }
}

/// Add / remove explicit tab stops for the current paragraph (F04.S3).
class TabStopsDialog extends StatefulWidget {
  const TabStopsDialog({super.key, required this.initial});

  final List<TabStopValue> initial;

  static Future<List<TabStopValue>?> show(
    BuildContext context, {
    required List<TabStopValue> initial,
  }) {
    return showDialog<List<TabStopValue>>(
      context: context,
      builder: (context) => TabStopsDialog(initial: initial),
    );
  }

  @override
  State<TabStopsDialog> createState() => _TabStopsDialogState();
}

class _TabStopsDialogState extends State<TabStopsDialog> {
  late List<TabStopValue> _stops;
  late final TextEditingController _positionController;
  String _alignment = 'Left';

  static const _alignments = ['Left', 'Center', 'Right', 'Decimal', 'Bar'];

  @override
  void initState() {
    super.initState();
    _stops = List<TabStopValue>.from(widget.initial)
      ..sort((a, b) => a.position.compareTo(b.position));
    _positionController = TextEditingController(text: '72');
  }

  @override
  void dispose() {
    _positionController.dispose();
    super.dispose();
  }

  void _addStop() {
    final parsed = double.tryParse(_positionController.text.trim());
    if (parsed == null || parsed.isNaN || parsed.isInfinite) return;
    final position = parsed.clamp(0, 1000).toDouble();
    setState(() {
      _stops.removeWhere((s) => (s.position - position).abs() < 0.01);
      _stops.add(TabStopValue(position: position, alignment: _alignment));
      _stops.sort((a, b) => a.position.compareTo(b.position));
    });
  }

  void _removeSelected(TabStopValue stop) {
    setState(() => _stops.removeWhere((s) => identical(s, stop) ||
        ((s.position - stop.position).abs() < 0.01 && s.alignment == stop.alignment)));
  }

  void _clearAll() => setState(() => _stops.clear());

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Tabs'),
      content: SizedBox(
        width: 380,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text('Tab stops', style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            SizedBox(
              height: 140,
              child: DecoratedBox(
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.black26),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: _stops.isEmpty
                    ? Center(
                        child: Text(
                          'No custom tab stops (default grid)',
                          style: WordTheme.ribbonLabel.copyWith(color: Colors.black54),
                        ),
                      )
                    : ListView.builder(
                        key: const Key('tab_stops_list'),
                        itemCount: _stops.length,
                        itemBuilder: (context, index) {
                          final stop = _stops[index];
                          final label = stop.position == stop.position.roundToDouble()
                              ? '${stop.position.round()} pt'
                              : '${stop.position.toStringAsFixed(1)} pt';
                          return ListTile(
                            dense: true,
                            title: Text('$label — ${stop.alignment}'),
                            trailing: IconButton(
                              key: Key('remove_tab_stop_$index'),
                              icon: const Icon(Icons.close, size: 18),
                              tooltip: 'Remove',
                              onPressed: () => _removeSelected(stop),
                            ),
                          );
                        },
                      ),
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('tab_stop_position'),
                    controller: _positionController,
                    keyboardType: const TextInputType.numberWithOptions(decimal: true),
                    inputFormatters: [
                      FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
                    ],
                    decoration: const InputDecoration(
                      labelText: 'Position',
                      suffixText: 'pt',
                      isDense: true,
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: DropdownButtonFormField<String>(
                    key: const Key('tab_stop_alignment'),
                    initialValue: _alignment,
                    items: [
                      for (final a in _alignments)
                        DropdownMenuItem(value: a, child: Text(a)),
                    ],
                    onChanged: (value) {
                      if (value == null) return;
                      setState(() => _alignment = value);
                    },
                    decoration: const InputDecoration(
                      labelText: 'Alignment',
                      isDense: true,
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                OutlinedButton(
                  key: const Key('tab_stop_add'),
                  onPressed: _addStop,
                  child: const Text('Set'),
                ),
                const SizedBox(width: 8),
                TextButton(
                  key: const Key('tab_stop_clear'),
                  onPressed: _stops.isEmpty ? null : _clearAll,
                  child: const Text('Clear All'),
                ),
              ],
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
          key: const Key('tab_stops_dialog_ok'),
          onPressed: () => Navigator.of(context).pop(List<TabStopValue>.from(_stops)),
          child: const Text('OK'),
        ),
      ],
    );
  }
}

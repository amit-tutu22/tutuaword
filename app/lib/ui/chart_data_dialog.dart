import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/chart_data.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Word-like Edit Data sheet for a chart (categories × series).
class ChartDataDialog extends StatefulWidget {
  const ChartDataDialog({super.key, required this.initial});

  final ChartDataModel initial;

  static Future<ChartDataModel?> show(
    BuildContext context, {
    required ChartDataModel initial,
  }) {
    return showDialog<ChartDataModel>(
      context: context,
      barrierDismissible: false,
      builder: (context) => ChartDataDialog(initial: initial),
    );
  }

  @override
  State<ChartDataDialog> createState() => _ChartDataDialogState();
}

class _ChartDataDialogState extends State<ChartDataDialog> {
  late ChartDataModel _data;
  late List<TextEditingController> _categoryControllers;
  late List<TextEditingController> _seriesNameControllers;
  late List<List<TextEditingController>> _valueControllers;
  String? _error;

  @override
  void initState() {
    super.initState();
    _data = widget.initial.copy();
    _syncControllersFromData();
  }

  void _syncControllersFromData() {
    _categoryControllers = [
      for (final c in _data.categories) TextEditingController(text: c),
    ];
    _seriesNameControllers = [
      for (final s in _data.series) TextEditingController(text: s.name),
    ];
    _valueControllers = [
      for (final s in _data.series)
        [
          for (final v in s.values)
            TextEditingController(
              text: v == v.roundToDouble() ? v.toStringAsFixed(0) : v.toString(),
            ),
        ],
    ];
  }

  void _disposeControllers() {
    for (final c in _categoryControllers) {
      c.dispose();
    }
    for (final c in _seriesNameControllers) {
      c.dispose();
    }
    for (final row in _valueControllers) {
      for (final c in row) {
        c.dispose();
      }
    }
  }

  @override
  void dispose() {
    _disposeControllers();
    super.dispose();
  }

  void _rebuildControllers(VoidCallback mutate) {
    setState(() {
      _error = null;
      _readControllersIntoData();
      _disposeControllers();
      mutate();
      _syncControllersFromData();
    });
  }

  void _readControllersIntoData() {
    for (var i = 0; i < _categoryControllers.length; i++) {
      _data.categories[i] = _categoryControllers[i].text.trim();
    }
    for (var s = 0; s < _seriesNameControllers.length; s++) {
      _data.series[s].name = _seriesNameControllers[s].text.trim();
      if (_data.series[s].name.isEmpty) {
        _data.series[s].name = 'Series ${s + 1}';
      }
      for (var c = 0; c < _valueControllers[s].length; c++) {
        final parsed = double.tryParse(_valueControllers[s][c].text.trim());
        _data.series[s].values[c] = parsed ?? 0;
      }
    }
  }

  /// Always mutate growable copies — engine/JSON lists can be fixed-length on
  /// some platforms, and [List.filled] defaults to fixed-length too.
  void _addCategory() {
    _rebuildControllers(() {
      final categories = List<String>.from(_data.categories);
      final n = categories.length + 1;
      categories.add('Category $n');
      _data.categories = categories;
      for (final s in _data.series) {
        final values = List<double>.from(s.values)..add(0);
        s.values = values;
      }
    });
  }

  void _removeCategory(int index) {
    if (_data.categories.length <= 1) return;
    if (index < 0 || index >= _data.categories.length) return;
    _rebuildControllers(() {
      final categories = List<String>.from(_data.categories)..removeAt(index);
      _data.categories = categories;
      for (final s in _data.series) {
        if (index >= s.values.length) continue;
        s.values = List<double>.from(s.values)..removeAt(index);
      }
    });
  }

  void _addSeries() {
    _rebuildControllers(() {
      final series = List<ChartSeriesModel>.from(_data.series);
      final n = series.length + 1;
      series.add(
        ChartSeriesModel(
          name: 'Series $n',
          values: List<double>.generate(_data.categories.length, (_) => 0),
        ),
      );
      _data.series = series;
    });
  }

  void _removeSeries(int index) {
    if (_data.series.length <= 1) return;
    if (index < 0 || index >= _data.series.length) return;
    _rebuildControllers(() {
      _data.series = List<ChartSeriesModel>.from(_data.series)..removeAt(index);
    });
  }

  void _submit() {
    _readControllersIntoData();
    final error = _data.validate();
    if (error != null) {
      setState(() => _error = error);
      return;
    }
    for (var i = 0; i < _data.categories.length; i++) {
      if (_data.categories[i].isEmpty) {
        _data.categories[i] = 'Category ${i + 1}';
      }
    }
    Navigator.of(context).pop(_data);
  }

  @override
  Widget build(BuildContext context) {
    final seriesCount = _data.series.length;
    final catCount = _data.categories.length;

    return AlertDialog(
      key: const Key('chart_data_dialog'),
      title: const Text('Edit Data'),
      content: SizedBox(
        width: 520,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'Enter categories and series values for this chart.',
              style: TextStyle(fontSize: 12, color: Color(0xFF666666)),
            ),
            const SizedBox(height: 12),
            ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 320),
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: SingleChildScrollView(
                  child: Table(
                    defaultColumnWidth: const FixedColumnWidth(96),
                    border: TableBorder.all(color: WordTheme.groupDivider, width: 1),
                    children: [
                      TableRow(
                        decoration: const BoxDecoration(color: WordTheme.ribbonSurface),
                        children: [
                          _headerCell(''),
                          for (var s = 0; s < seriesCount; s++)
                            _seriesHeader(s),
                          _iconHeader(
                            key: const Key('chart_data_add_series'),
                            icon: Icons.add,
                            tooltip: 'Add series',
                            onTap: _addSeries,
                          ),
                        ],
                      ),
                      for (var c = 0; c < catCount; c++)
                        TableRow(
                          children: [
                            _categoryCell(c),
                            for (var s = 0; s < seriesCount; s++)
                              _valueCell(s, c),
                            _iconCell(
                              key: Key('chart_data_remove_category_$c'),
                              icon: Icons.remove,
                              tooltip: 'Remove category',
                              enabled: catCount > 1,
                              onTap: () => _removeCategory(c),
                            ),
                          ],
                        ),
                      TableRow(
                        children: [
                          _iconCell(
                            key: const Key('chart_data_add_category'),
                            icon: Icons.add,
                            tooltip: 'Add category',
                            onTap: _addCategory,
                          ),
                          for (var s = 0; s < seriesCount; s++)
                            _iconCell(
                              key: Key('chart_data_remove_series_$s'),
                              icon: Icons.remove,
                              tooltip: 'Remove series',
                              enabled: seriesCount > 1,
                              onTap: () => _removeSeries(s),
                            ),
                          const SizedBox.shrink(),
                        ],
                      ),
                    ],
                  ),
                ),
              ),
            ),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(
                _error!,
                style: const TextStyle(color: Colors.red, fontSize: 12),
              ),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('chart_data_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('chart_data_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }

  Widget _headerCell(String text) {
    return Padding(
      padding: const EdgeInsets.all(6),
      child: Text(
        text,
        style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
      ),
    );
  }

  Widget _seriesHeader(int index) {
    return Padding(
      padding: const EdgeInsets.all(2),
      child: TextField(
        key: Key('chart_data_series_name_$index'),
        controller: _seriesNameControllers[index],
        style: const TextStyle(fontSize: 11, fontWeight: FontWeight.w600),
        decoration: const InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.symmetric(horizontal: 4, vertical: 6),
        ),
      ),
    );
  }

  Widget _categoryCell(int index) {
    return Padding(
      padding: const EdgeInsets.all(2),
      child: TextField(
        key: Key('chart_data_category_$index'),
        controller: _categoryControllers[index],
        style: const TextStyle(fontSize: 11),
        decoration: const InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.symmetric(horizontal: 4, vertical: 6),
        ),
      ),
    );
  }

  Widget _valueCell(int seriesIndex, int categoryIndex) {
    return Padding(
      padding: const EdgeInsets.all(2),
      child: TextField(
        key: Key('chart_data_value_${seriesIndex}_$categoryIndex'),
        controller: _valueControllers[seriesIndex][categoryIndex],
        keyboardType: const TextInputType.numberWithOptions(decimal: true),
        inputFormatters: [
          FilteringTextInputFormatter.allow(RegExp(r'[-0-9.]')),
        ],
        style: const TextStyle(fontSize: 11),
        decoration: const InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.symmetric(horizontal: 4, vertical: 6),
        ),
      ),
    );
  }

  Widget _iconHeader({
    required Key key,
    required IconData icon,
    required String tooltip,
    required VoidCallback onTap,
  }) {
    return _iconCell(
      key: key,
      icon: icon,
      tooltip: tooltip,
      onTap: onTap,
    );
  }

  Widget _iconCell({
    required Key key,
    required IconData icon,
    required String tooltip,
    required VoidCallback onTap,
    bool enabled = true,
  }) {
    return SizedBox(
      height: 36,
      child: IconButton(
        key: key,
        tooltip: tooltip,
        iconSize: 16,
        padding: EdgeInsets.zero,
        onPressed: enabled ? onTap : null,
        icon: Icon(icon, size: 16),
      ),
    );
  }
}

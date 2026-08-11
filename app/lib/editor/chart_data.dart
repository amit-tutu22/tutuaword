/// Editable chart dataset matching Rust [`tw_model::ChartData`] JSON.
class ChartDataModel {
  ChartDataModel({
    required this.kind,
    required this.categories,
    required this.series,
  });

  /// Serde name: column / bar / line / pie.
  String kind;
  List<String> categories;
  List<ChartSeriesModel> series;

  static const kindNames = ['column', 'bar', 'line', 'pie'];

  static String kindFromType(int chartType) {
    if (chartType >= 0 && chartType < kindNames.length) {
      return kindNames[chartType];
    }
    return 'column';
  }

  factory ChartDataModel.sample({int chartType = 0}) {
    return ChartDataModel(
      kind: kindFromType(chartType),
      categories: const [
        'Category 1',
        'Category 2',
        'Category 3',
        'Category 4',
      ],
      series: [
        ChartSeriesModel(name: 'Series 1', values: [4.3, 2.5, 3.5, 4.5]),
        ChartSeriesModel(name: 'Series 2', values: [2.4, 4.4, 1.8, 2.8]),
      ],
    );
  }

  factory ChartDataModel.fromJson(Map<String, dynamic> json) {
    final categories = (json['categories'] as List? ?? const [])
        .map((e) => e?.toString() ?? '')
        .toList();
    final seriesRaw = json['series'] as List? ?? const [];
    final series = seriesRaw.map((raw) {
      final map = Map<String, dynamic>.from(raw as Map);
      final values = (map['values'] as List? ?? const [])
          .map((v) => (v as num?)?.toDouble() ?? 0.0)
          .toList();
      return ChartSeriesModel(
        name: map['name']?.toString() ?? 'Series',
        values: values,
      );
    }).toList();
    final kind = json['kind']?.toString();
    return ChartDataModel(
      kind: (kind != null && kind.isNotEmpty) ? kind : 'column',
      categories: categories,
      series: series,
    );
  }

  Map<String, dynamic> toJson() => {
        'kind': kind,
        'categories': categories,
        'series': series.map((s) => s.toJson()).toList(),
      };

  String? validate() {
    if (categories.isEmpty) return 'Add at least one category';
    if (series.isEmpty) return 'Add at least one series';
    for (final s in series) {
      if (s.values.length != categories.length) {
        return 'Series values must match category count';
      }
    }
    return null;
  }

  ChartDataModel copy() => ChartDataModel.fromJson(toJson());
}

class ChartSeriesModel {
  ChartSeriesModel({required this.name, required this.values});

  String name;
  List<double> values;

  Map<String, dynamic> toJson() => {
        'name': name,
        'values': values,
      };
}

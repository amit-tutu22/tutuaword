import 'package:flutter/material.dart';

/// Page setup presets for the Layout tab (F07.S1+).
class PageSetupPresets {
  static const marginNames = ['Normal', 'Narrow', 'Moderate', 'Wide'];
  static const pageSizeNames = ['Letter', 'A4', 'Legal'];

  static double? marginValue(String name) => switch (name) {
        'Normal' => 72,
        'Narrow' => 36,
        'Moderate' => 54,
        'Wide' => 108,
        _ => null,
      };

  static (double width, double height)? pageSizeDimensions(String name) =>
      switch (name) {
        'Letter' => (612, 792),
        'A4' => (595, 842),
        'Legal' => (612, 1008),
        _ => null,
      };

  static Map<String, dynamic> defaultSectionFormat() => {
        'page_width': 612.0,
        'page_height': 792.0,
        'margin_top': 72.0,
        'margin_bottom': 72.0,
        'margin_left': 72.0,
        'margin_right': 72.0,
        'columns': {'count': 1, 'gap': 12.0},
      };

  static Map<String, dynamic> mergeSectionFormat(
    Map<String, dynamic> current,
    Map<String, dynamic> patch,
  ) =>
      {...current, ...patch};

  static Map<String, dynamic> withMarginPreset(
    Map<String, dynamic> current,
    String name,
  ) {
    final margin = marginValue(name);
    if (margin == null) return current;
    return mergeSectionFormat(current, {
      'margin_top': margin,
      'margin_bottom': margin,
      'margin_left': margin,
      'margin_right': margin,
    });
  }

  static Map<String, dynamic> withOrientation(
    Map<String, dynamic> current, {
    required bool landscape,
  }) {
    final width = (current['page_width'] as num?)?.toDouble() ?? 612;
    final height = (current['page_height'] as num?)?.toDouble() ?? 792;
    final isLandscape = width > height;
    if (landscape == isLandscape) return current;
    return mergeSectionFormat(current, {
      'page_width': height,
      'page_height': width,
    });
  }

  static Map<String, dynamic> withPageSizePreset(
    Map<String, dynamic> current,
    String name,
  ) {
    final dims = pageSizeDimensions(name);
    if (dims == null) return current;
    var (width, height) = dims;
    final landscape = ((current['page_width'] as num?)?.toDouble() ?? 612) >
        ((current['page_height'] as num?)?.toDouble() ?? 792);
    if (landscape) {
      final swapped = width;
      width = height;
      height = swapped;
    }
    return mergeSectionFormat(current, {
      'page_width': width,
      'page_height': height,
    });
  }

  static const columnCountNames = ['One', 'Two', 'Three'];

  static Map<String, dynamic> withColumnCount(
    Map<String, dynamic> current,
    int count,
  ) =>
      mergeSectionFormat(current, {
        'columns': {'count': count.clamp(1, 3), 'gap': 12.0},
      });

  static int columnCount(Map<String, dynamic> format) {
    final columns = format['columns'];
    if (columns is Map) {
      return ((columns['count'] as num?)?.toInt() ?? 1).clamp(1, 3);
    }
    return 1;
  }

  static String columnCountLabel(Map<String, dynamic> format) {
    return switch (columnCount(format)) {
      2 => 'Two',
      3 => 'Three',
      _ => 'One',
    };
  }

  static bool isLandscape(Map<String, dynamic> format) {
    final width = (format['page_width'] as num?)?.toDouble() ?? 612;
    final height = (format['page_height'] as num?)?.toDouble() ?? 792;
    return width > height;
  }

  static String? matchingMarginPreset(Map<String, dynamic> format) {
    final top = (format['margin_top'] as num?)?.toDouble();
    final bottom = (format['margin_bottom'] as num?)?.toDouble();
    final left = (format['margin_left'] as num?)?.toDouble();
    final right = (format['margin_right'] as num?)?.toDouble();
    if (top == null || bottom == null || left == null || right == null) {
      return null;
    }
    if (top != bottom || top != left || top != right) return null;
    for (final name in marginNames) {
      if (marginValue(name) == top) return name;
    }
    return null;
  }

  static String? matchingPageSizePreset(Map<String, dynamic> format) {
    var width = (format['page_width'] as num?)?.toDouble() ?? 612;
    var height = (format['page_height'] as num?)?.toDouble() ?? 792;
    if (width > height) {
      final swapped = width;
      width = height;
      height = swapped;
    }
    for (final name in pageSizeNames) {
      final dims = pageSizeDimensions(name);
      if (dims == null) continue;
      if ((dims.$1 - width).abs() < 0.5 && (dims.$2 - height).abs() < 0.5) {
        return name;
      }
    }
    return null;
  }

  static Map<String, dynamic>? colorJson(Color color) => {
        'r': color.red,
        'g': color.green,
        'b': color.blue,
        'a': color.alpha,
      };

  static Color? colorFromJson(Object? json) {
    if (json is! Map) return null;
    final r = (json['r'] as num?)?.toInt();
    final g = (json['g'] as num?)?.toInt();
    final b = (json['b'] as num?)?.toInt();
    final a = (json['a'] as num?)?.toInt() ?? 255;
    if (r == null || g == null || b == null) return null;
    return Color.fromARGB(a, r, g, b);
  }

  static Map<String, dynamic> withPageColor(
    Map<String, dynamic> current,
    Color? color,
  ) {
    final next = Map<String, dynamic>.from(current);
    if (color == null) {
      next.remove('page_color');
    } else {
      next['page_color'] = colorJson(color);
    }
    return next;
  }

  static Color? pageColor(Map<String, dynamic> format) =>
      colorFromJson(format['page_color']);

  static const watermarkPresets = ['CONFIDENTIAL', 'DRAFT', 'SAMPLE'];
  static const removeWatermarkLabel = 'Remove Watermark';

  static const _defaultWatermarkColor = {
    'r': 192,
    'g': 192,
    'b': 192,
    'a': 160,
  };

  static Map<String, dynamic> withWatermark(
    Map<String, dynamic> current,
    String text,
  ) =>
      mergeSectionFormat(current, {
        'watermark': {'text': text, 'color': _defaultWatermarkColor},
      });

  static Map<String, dynamic> withoutWatermark(Map<String, dynamic> current) {
    final next = Map<String, dynamic>.from(current);
    next.remove('watermark');
    return next;
  }

  static String? watermarkText(Map<String, dynamic> format) {
    final wm = format['watermark'];
    if (wm is Map) return wm['text'] as String?;
    return null;
  }

  static Map<String, dynamic> withLineNumbers(
    Map<String, dynamic> current, {
    required bool enabled,
    int start = 1,
  }) =>
      mergeSectionFormat(current, {
        'line_numbers': {'enabled': enabled, 'start': start},
      });

  static bool lineNumbersEnabled(Map<String, dynamic> format) {
    final ln = format['line_numbers'];
    if (ln is Map) return ln['enabled'] == true;
    return false;
  }

  static Map<String, dynamic> withDifferentFirstPage(
    Map<String, dynamic> current,
    bool enabled,
  ) =>
      mergeSectionFormat(current, {'different_first_page': enabled});

  static bool differentFirstPage(Map<String, dynamic> format) =>
      format['different_first_page'] as bool? ?? false;
}

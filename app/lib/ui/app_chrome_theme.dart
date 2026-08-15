import 'package:flutter/material.dart';

/// Named accent for the app chrome (title bar, tabs, ribbon, status bar).
class ChromeAccentPreset {
  const ChromeAccentPreset({
    required this.id,
    required this.label,
    required this.color,
  });

  final String id;
  final String label;
  final Color color;
}

const kDefaultChromeAccent = Color(0xFF2B579A);

const kChromeAccentPresets = <ChromeAccentPreset>[
  ChromeAccentPreset(id: 'blue', label: 'Word Blue', color: kDefaultChromeAccent),
  ChromeAccentPreset(id: 'teal', label: 'Teal', color: Color(0xFF0F766E)),
  ChromeAccentPreset(id: 'forest', label: 'Forest', color: Color(0xFF3F6B4A)),
  ChromeAccentPreset(id: 'grape', label: 'Grape', color: Color(0xFF6D28D9)),
  ChromeAccentPreset(id: 'rose', label: 'Rose', color: Color(0xFFBE185D)),
  ChromeAccentPreset(id: 'coral', label: 'Coral', color: Color(0xFFC2410C)),
  ChromeAccentPreset(id: 'sky', label: 'Sky', color: Color(0xFF0369A1)),
  ChromeAccentPreset(id: 'slate', label: 'Slate', color: Color(0xFF334155)),
];

/// Derived chrome colors from a user-chosen accent.
@immutable
class TutuawordChrome extends ThemeExtension<TutuawordChrome> {
  const TutuawordChrome({
    required this.accent,
    required this.titleBar,
    required this.tabStrip,
    required this.ribbonSurface,
    required this.statusBar,
    required this.canvas,
    required this.activeTab,
    required this.ribbonHover,
    required this.ribbonSelected,
    required this.groupDivider,
  });

  factory TutuawordChrome.fromAccent(Color accent) {
    Color tint(Color base, double amount) => Color.lerp(base, accent, amount)!;
    return TutuawordChrome(
      accent: accent,
      titleBar: accent,
      tabStrip: tint(Colors.white, 0.06),
      ribbonSurface: tint(const Color(0xFFF5F5F5), 0.12),
      statusBar: tint(const Color(0xFFF5F5F5), 0.10),
      canvas: tint(const Color(0xFFE6E6E6), 0.10),
      activeTab: accent,
      ribbonHover: tint(const Color(0xFFE8E8E8), 0.18),
      ribbonSelected: tint(const Color(0xFFD0D0D0), 0.28),
      groupDivider: tint(const Color(0xFFDDDDDD), 0.20),
    );
  }

  static final wordBlue = TutuawordChrome.fromAccent(kDefaultChromeAccent);

  static TutuawordChrome of(BuildContext context) {
    return Theme.of(context).extension<TutuawordChrome>() ?? wordBlue;
  }

  final Color accent;
  final Color titleBar;
  final Color tabStrip;
  final Color ribbonSurface;
  final Color statusBar;
  final Color canvas;
  final Color activeTab;
  final Color ribbonHover;
  final Color ribbonSelected;
  final Color groupDivider;

  @override
  TutuawordChrome copyWith({
    Color? accent,
    Color? titleBar,
    Color? tabStrip,
    Color? ribbonSurface,
    Color? statusBar,
    Color? canvas,
    Color? activeTab,
    Color? ribbonHover,
    Color? ribbonSelected,
    Color? groupDivider,
  }) {
    return TutuawordChrome(
      accent: accent ?? this.accent,
      titleBar: titleBar ?? this.titleBar,
      tabStrip: tabStrip ?? this.tabStrip,
      ribbonSurface: ribbonSurface ?? this.ribbonSurface,
      statusBar: statusBar ?? this.statusBar,
      canvas: canvas ?? this.canvas,
      activeTab: activeTab ?? this.activeTab,
      ribbonHover: ribbonHover ?? this.ribbonHover,
      ribbonSelected: ribbonSelected ?? this.ribbonSelected,
      groupDivider: groupDivider ?? this.groupDivider,
    );
  }

  @override
  TutuawordChrome lerp(ThemeExtension<TutuawordChrome>? other, double t) {
    if (other is! TutuawordChrome) return this;
    return TutuawordChrome(
      accent: Color.lerp(accent, other.accent, t)!,
      titleBar: Color.lerp(titleBar, other.titleBar, t)!,
      tabStrip: Color.lerp(tabStrip, other.tabStrip, t)!,
      ribbonSurface: Color.lerp(ribbonSurface, other.ribbonSurface, t)!,
      statusBar: Color.lerp(statusBar, other.statusBar, t)!,
      canvas: Color.lerp(canvas, other.canvas, t)!,
      activeTab: Color.lerp(activeTab, other.activeTab, t)!,
      ribbonHover: Color.lerp(ribbonHover, other.ribbonHover, t)!,
      ribbonSelected: Color.lerp(ribbonSelected, other.ribbonSelected, t)!,
      groupDivider: Color.lerp(groupDivider, other.groupDivider, t)!,
    );
  }
}

ThemeData buildTutuawordTheme(Color accent) {
  final chrome = TutuawordChrome.fromAccent(accent);
  return ThemeData(
    useMaterial3: true,
    colorScheme: ColorScheme.fromSeed(seedColor: accent),
    extensions: <ThemeExtension<dynamic>>[chrome],
    dividerColor: chrome.groupDivider,
    scaffoldBackgroundColor: chrome.canvas,
    sliderTheme: SliderThemeData(
      activeTrackColor: accent,
      thumbColor: accent,
    ),
  );
}

String colorToHex(Color color) =>
    color.toARGB32().toRadixString(16).padLeft(8, '0');

Color? colorFromHex(String? value) {
  if (value == null || value.isEmpty) return null;
  var hex = value.replaceFirst('#', '');
  if (hex.length == 6) hex = 'ff$hex';
  if (hex.length != 8) return null;
  final parsed = int.tryParse(hex, radix: 16);
  if (parsed == null) return null;
  return Color(parsed);
}

bool colorsEqual(Color a, Color b) => a.toARGB32() == b.toARGB32();

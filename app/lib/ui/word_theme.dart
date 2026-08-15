import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/ui/app_chrome_theme.dart';

/// Microsoft Word for Mac design tokens.
abstract final class WordTheme {
  static const titleBarBlue = Color(0xFF2B579A);
  static const ribbonSurface = Color(0xFFF5F5F5);
  static const tabStripSurface = Colors.white;
  static const activeTabUnderline = Color(0xFF2B579A);
  static const groupDivider = Color(0xFFDDDDDD);
  static const canvasGray = Color(0xFFE6E6E6);
  static const statusBarSurface = Color(0xFFF5F5F5);
  static const infoBarAmber = Color(0xFFFFF4CE);
  static const ribbonText = Color(0xFF333333);
  static const ribbonTextDisabled = Color(0xFFAAAAAA);
  static const ribbonHover = Color(0xFFE8E8E8);
  static const ribbonSelected = Color(0xFFD0D0D0);
  static const titleBarText = Colors.white;
  static const titleBarIcon = Colors.white70;

  static const titleBarHeight = 38.0;
  static const tabStripHeight = 32.0;
  static const ribbonHeight = 92.0;
  static const ribbonHeightPhone = 64.0;
  static const statusBarHeight = 26.0;
  static const trafficLightInset = 78.0;

  static bool _isMobilePlatform() {
    switch (defaultTargetPlatform) {
      case TargetPlatform.iOS:
      case TargetPlatform.android:
        return true;
      default:
        return false;
    }
  }

  /// iPhone / Android phone: shortest side under 600 logical px.
  static bool phoneChrome(BuildContext context) {
    if (!_isMobilePlatform()) return false;
    return MediaQuery.sizeOf(context).shortestSide < 600;
  }

  /// iPad / Android tablet: mobile platform but not phone.
  static bool tabletChrome(BuildContext context) {
    return _isMobilePlatform() && !phoneChrome(context);
  }

  /// Any iOS or Android surface (phone or tablet).
  static bool mobileChrome(BuildContext context) => _isMobilePlatform();

  /// macOS traffic-light inset on desktop only; phones/tablets use a small pad.
  static double leadingChromeInset(BuildContext context) {
    if (mobileChrome(context)) return 8.0;
    if (defaultTargetPlatform == TargetPlatform.macOS) {
      return trafficLightInset;
    }
    return 8.0;
  }

  /// Ribbon body height: shorter on phones for more canvas space.
  static double ribbonHeightFor(BuildContext context) {
    return phoneChrome(context) ? ribbonHeightPhone : ribbonHeight;
  }

  /// @deprecated Use [phoneChrome] for phone-only layouts.
  static bool compactChrome(BuildContext context) => phoneChrome(context);

  static const iconSize = 16.0;
  static const largeIconSize = 26.0;

  static const TextStyle ribbonLabel = TextStyle(
    fontSize: 11,
    color: ribbonText,
    height: 1.2,
  );

  static const TextStyle ribbonGroupLabel = TextStyle(
    fontSize: 10,
    color: Color(0xFF666666),
    height: 1.0,
  );

  static const TextStyle tabLabel = TextStyle(
    fontSize: 12,
    color: ribbonText,
    fontWeight: FontWeight.w400,
  );

  static const TextStyle tabLabelActive = TextStyle(
    fontSize: 12,
    color: activeTabUnderline,
    fontWeight: FontWeight.w600,
  );

  static const TextStyle titleBarTitle = TextStyle(
    fontSize: 13,
    color: titleBarText,
    fontWeight: FontWeight.w400,
  );

  static const TextStyle statusBarText = TextStyle(
    fontSize: 11,
    color: Color(0xFF555555),
    height: 1.0,
  );

  /// Live chrome colors from the app theme (falls back to Word Blue).
  static TutuawordChrome chrome(BuildContext context) => TutuawordChrome.of(context);

  static TextStyle tabLabelActiveOf(BuildContext context) => tabLabelActive.copyWith(
        color: chrome(context).activeTab,
      );
}

import 'package:flutter/material.dart';

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
  static const statusBarHeight = 26.0;
  static const trafficLightInset = 78.0;

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
}

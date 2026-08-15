import 'package:flutter/material.dart';
import 'package:tutuaword/app_bootstrap.dart';
import 'package:tutuaword/ui/app_chrome_theme.dart';
import 'package:tutuaword/ui/app_theme_controller.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const TutuawordApp());
}

class TutuawordApp extends StatefulWidget {
  const TutuawordApp({super.key});

  @override
  State<TutuawordApp> createState() => _TutuawordAppState();
}

class _TutuawordAppState extends State<TutuawordApp> {
  late final AppThemeController _theme;

  @override
  void initState() {
    super.initState();
    _theme = AppThemeController.instance;
    _theme.addListener(_onTheme);
  }

  @override
  void dispose() {
    _theme.removeListener(_onTheme);
    super.dispose();
  }

  void _onTheme() => setState(() {});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'tutuaword',
      debugShowCheckedModeBanner: false,
      theme: buildTutuawordTheme(_theme.accent),
      home: const AppBootstrap(),
    );
  }
}

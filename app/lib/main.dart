import 'package:flutter/material.dart';
import 'package:tutuaword/app_bootstrap.dart';
import 'package:tutuaword/ui/word_theme.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const TutuawordApp());
}

class TutuawordApp extends StatelessWidget {
  const TutuawordApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'tutuaword',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: WordTheme.titleBarBlue),
        useMaterial3: true,
      ),
      home: const AppBootstrap(),
    );
  }
}

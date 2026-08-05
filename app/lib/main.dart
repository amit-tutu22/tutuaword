import 'package:flutter/material.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_menu.dart';
import 'package:tutuaword/ui/info_bar.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/title_bar.dart';
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
        fontFamily: '.AppleSystemUIFont',
      ),
      home: const EditorScreen(),
    );
  }
}

class EditorScreen extends StatefulWidget {
  const EditorScreen({super.key});

  @override
  State<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends State<EditorScreen> {
  late final EditorController _controller;
  final _ribbonKey = GlobalKey<WordRibbonState>();

  @override
  void initState() {
    super.initState();
    _controller = EditorController();
    _controller.addListener(_onUpdate);
  }

  void _onUpdate() => setState(() {});

  @override
  void dispose() {
    _controller.removeListener(_onUpdate);
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return EditorMenuBar(
      controller: _controller,
      onOpen: () => _controller.openDocument(),
      onSave: () => _controller.saveDocument(),
      onSaveAs: (ext) => _controller.saveDocumentAs(extension: ext),
      child: Material(
        color: WordTheme.tabStripSurface,
        child: Column(
          children: [
            WordTitleBar(controller: _controller),
            WordRibbon(key: _ribbonKey, controller: _controller),
            InfoBar(controller: _controller),
            Expanded(
              child: DocumentView(controller: _controller),
            ),
            WordStatusBar(controller: _controller),
          ],
        ),
      ),
    );
  }
}

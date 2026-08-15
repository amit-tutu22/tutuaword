import 'package:flutter/material.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Second in-app view of the same document (View → New Window).
class SecondaryDocumentWindow extends StatelessWidget {
  const SecondaryDocumentWindow({super.key, required this.controller});

  final EditorController controller;

  static Future<void> show(
    BuildContext context, {
    required EditorController controller,
  }) {
    return Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        fullscreenDialog: true,
        builder: (context) => SecondaryDocumentWindow(controller: controller),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return Material(
          key: const Key('secondary_document_window'),
          color: WordTheme.chrome(context).tabStrip,
          child: Column(
            children: [
              Container(
                height: WordTheme.titleBarHeight,
                color: WordTheme.chrome(context).titleBar,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        '${controller.documentTitle} — Window 2',
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    IconButton(
                      key: const Key('secondary_window_close'),
                      icon: const Icon(Icons.close, color: Colors.white, size: 18),
                      tooltip: 'Close window',
                      onPressed: () => Navigator.of(context).pop(),
                    ),
                  ],
                ),
              ),
              Expanded(child: DocumentView(controller: controller)),
              WordStatusBar(controller: controller),
            ],
          ),
        );
      },
    );
  }
}

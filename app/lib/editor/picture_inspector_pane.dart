import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Right-side pane for the selected picture's accessibility alt text (F21.S3).
class PictureInspectorPane extends StatefulWidget {
  const PictureInspectorPane({
    super.key,
    required this.controller,
  });

  final EditorController controller;

  @override
  State<PictureInspectorPane> createState() => _PictureInspectorPaneState();
}

class _PictureInspectorPaneState extends State<PictureInspectorPane> {
  late final TextEditingController _altController;
  final FocusNode _altFocus = FocusNode();
  String? _boundImageId;

  @override
  void initState() {
    super.initState();
    _altController = TextEditingController();
    widget.controller.addListener(_syncFromSelection);
    _syncFromSelection();
  }

  @override
  void didUpdateWidget(covariant PictureInspectorPane oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != widget.controller) {
      oldWidget.controller.removeListener(_syncFromSelection);
      widget.controller.addListener(_syncFromSelection);
      _syncFromSelection();
    }
  }

  @override
  void dispose() {
    widget.controller.removeListener(_syncFromSelection);
    _altController.dispose();
    _altFocus.dispose();
    super.dispose();
  }

  void _syncFromSelection() {
    final id = widget.controller.selectedImageId;
    final text = widget.controller.selectedImageAltText;
    if (id != _boundImageId) {
      _boundImageId = id;
      _altController.text = text;
      return;
    }
    if (!_altFocus.hasFocus && _altController.text != text) {
      _altController.text = text;
    }
  }

  Future<void> _commit() async {
    await widget.controller.setSelectedImageAltText(_altController.text);
  }

  @override
  Widget build(BuildContext context) {
    final hasImage = widget.controller.hasSelectedImage;
    return Container(
      key: const Key('picture_inspector_pane'),
      width: 220,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Padding(
            padding: EdgeInsets.fromLTRB(12, 10, 12, 6),
            child: Text(
              'Picture',
              style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
            ),
          ),
          const Divider(height: 1),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: hasImage
                  ? Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        const Text(
                          'Alt text',
                          style: TextStyle(fontSize: 11, fontWeight: FontWeight.w500),
                        ),
                        const SizedBox(height: 6),
                        TextField(
                          key: const Key('picture_alt_text_field'),
                          controller: _altController,
                          focusNode: _altFocus,
                          maxLines: 4,
                          style: const TextStyle(fontSize: 12),
                          decoration: const InputDecoration(
                            isDense: true,
                            hintText: 'Describe this picture',
                            border: OutlineInputBorder(),
                          ),
                          onSubmitted: (_) => _commit(),
                          onEditingComplete: _commit,
                        ),
                        const SizedBox(height: 8),
                        Align(
                          alignment: Alignment.centerRight,
                          child: TextButton(
                            key: const Key('picture_alt_text_apply'),
                            onPressed: _commit,
                            child: const Text('Apply'),
                          ),
                        ),
                      ],
                    )
                  : const Text(
                      'Select a picture to edit alt text.',
                      style: TextStyle(fontSize: 12, height: 1.4),
                    ),
            ),
          ),
        ],
      ),
    );
  }
}

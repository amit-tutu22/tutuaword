import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

class InfoBar extends StatelessWidget {
  const InfoBar({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    final message = controller.infoMessage ??
        (!controller.isEngineConnected ? 'Running in mock mode — build libtw_ffi to enable Rust engine' : null);

    if (message == null) return const SizedBox.shrink();

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      color: WordTheme.infoBarAmber,
      child: Row(
        children: [
          Expanded(
            child: Text(
              message,
              style: WordTheme.ribbonLabel.copyWith(fontSize: 11),
              maxLines: WordTheme.phoneChrome(context) ? 2 : null,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          if (controller.infoMessage != null)
            TextButton(
              onPressed: controller.clearInfoMessage,
              style: TextButton.styleFrom(
                padding: const EdgeInsets.symmetric(horizontal: 8),
                minimumSize: Size.zero,
                tapTargetSize: MaterialTapTargetSize.shrinkWrap,
              ),
              child: const Text('Dismiss', style: TextStyle(fontSize: 11)),
            ),
        ],
      ),
    );
  }
}

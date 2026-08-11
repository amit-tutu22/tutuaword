import 'dart:async';

import 'package:tutuaword/editor/editor_controller.dart';

abstract class WebKeyListener {
  void attach();
  void dispose();
}

WebKeyListener createWebKeyListener(EditorController controller) =>
    _NoopWebKeyListener();

class _NoopWebKeyListener implements WebKeyListener {
  @override
  void attach() {}

  @override
  void dispose() {}
}

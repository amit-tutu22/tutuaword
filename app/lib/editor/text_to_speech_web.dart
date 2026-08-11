import 'dart:async';
import 'dart:js_util' as js_util;

import 'package:tutuaword/editor/text_to_speech.dart';

/// Browser [SpeechSynthesis] backend (F21.S5).
TextToSpeechEngine createPlatformTextToSpeech() => WebTextToSpeech();

class WebTextToSpeech implements TextToSpeechEngine {
  bool _speaking = false;
  Completer<void>? _active;

  @override
  bool get isSpeaking => _speaking;

  Object get _synth => js_util.getProperty(js_util.globalThis, 'speechSynthesis');

  @override
  Future<void> speak(String text) async {
    await stop();
    if (text.trim().isEmpty) return;

    final utterCtor =
        js_util.getProperty(js_util.globalThis, 'SpeechSynthesisUtterance');
    final utter = js_util.callConstructor(utterCtor, [text]);
    final completer = Completer<void>();
    _active = completer;
    _speaking = true;

    void finish() {
      if (!completer.isCompleted) completer.complete();
      if (identical(_active, completer)) {
        _active = null;
        _speaking = false;
      }
    }

    js_util.setProperty(utter, 'onend', js_util.allowInterop((_) => finish()));
    js_util.setProperty(utter, 'onerror', js_util.allowInterop((_) => finish()));
    js_util.callMethod(_synth, 'speak', [utter]);
    await completer.future;
  }

  @override
  Future<void> stop() async {
    try {
      js_util.callMethod(_synth, 'cancel', []);
    } catch (_) {
      // speechSynthesis may be unavailable in headless tests.
    }
    _speaking = false;
    final pending = _active;
    _active = null;
    if (pending != null && !pending.isCompleted) {
      pending.complete();
    }
  }
}

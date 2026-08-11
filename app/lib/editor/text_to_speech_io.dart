import 'dart:io';

import 'package:flutter/services.dart';
import 'package:tutuaword/editor/text_to_speech.dart';

/// Native TTS via platform channel (macOS NSSpeechSynthesizer) — F21.S5.
TextToSpeechEngine createPlatformTextToSpeech() => IoTextToSpeech();

class IoTextToSpeech implements TextToSpeechEngine {
  static const _channel = MethodChannel('tutuaword/tts');
  bool _speaking = false;
  bool _pluginMissing = false;

  @override
  bool get isSpeaking => _speaking;

  @override
  Future<void> speak(String text) async {
    final trimmed = text.trim();
    if (trimmed.isEmpty) return;
    if (_pluginMissing) {
      await _speakViaProcess(trimmed);
      return;
    }
    try {
      _speaking = true;
      await _channel.invokeMethod<void>('speak', {'text': trimmed});
    } on MissingPluginException {
      _pluginMissing = true;
      await _speakViaProcess(trimmed);
    } on PlatformException {
      _speaking = false;
      rethrow;
    } finally {
      _speaking = false;
    }
  }

  @override
  Future<void> stop() async {
    _speaking = false;
    if (_pluginMissing) {
      // Best-effort: macOS `say` subprocess cannot be cancelled cleanly here.
      return;
    }
    try {
      await _channel.invokeMethod<void>('stop');
    } on MissingPluginException {
      _pluginMissing = true;
    } on PlatformException {
      // Ignore.
    }
  }

  /// Desktop fallback when the Flutter plugin is not registered (e.g. tests).
  Future<void> _speakViaProcess(String text) async {
    if (!Platform.isMacOS) {
      throw UnsupportedError('Text-to-speech plugin is not available');
    }
    _speaking = true;
    try {
      final result = await Process.run('say', [text]);
      if (result.exitCode != 0) {
        throw StateError('say failed: ${result.stderr}');
      }
    } finally {
      _speaking = false;
    }
  }
}

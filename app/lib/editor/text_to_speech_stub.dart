import 'package:tutuaword/editor/text_to_speech.dart';

/// Fallback when no platform TTS is available.
TextToSpeechEngine createPlatformTextToSpeech() => _UnavailableTextToSpeech();

class _UnavailableTextToSpeech implements TextToSpeechEngine {
  @override
  bool get isSpeaking => false;

  @override
  Future<void> speak(String text) async {
    throw UnsupportedError('Text-to-speech is not available on this platform');
  }

  @override
  Future<void> stop() async {}
}

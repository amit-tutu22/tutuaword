import 'dart:async';

/// Platform text-to-speech used by Read Aloud (F21.S5).
abstract class TextToSpeechEngine {
  /// Speaks [text]. Completes when utterance finishes or is stopped.
  Future<void> speak(String text);

  /// Cancels any in-progress speech.
  Future<void> stop();

  /// Whether an utterance is currently playing.
  bool get isSpeaking;
}

/// In-memory engine for tests — records spoken strings, no audio.
class RecordingTextToSpeech implements TextToSpeechEngine {
  RecordingTextToSpeech({this.completeImmediately = true});

  final List<String> spoken = <String>[];
  final bool completeImmediately;
  bool _speaking = false;
  Completer<void>? _hold;

  @override
  bool get isSpeaking => _speaking;

  @override
  Future<void> speak(String text) async {
    spoken.add(text);
    _speaking = true;
    if (completeImmediately) {
      _speaking = false;
      return;
    }
    _hold = Completer<void>();
    await _hold!.future;
    _speaking = false;
    _hold = null;
  }

  @override
  Future<void> stop() async {
    _speaking = false;
    final pending = _hold;
    _hold = null;
    if (pending != null && !pending.isCompleted) {
      pending.complete();
    }
  }

  /// Mark a non-immediate utterance finished (tests).
  void finish() {
    _speaking = false;
    final pending = _hold;
    _hold = null;
    if (pending != null && !pending.isCompleted) {
      pending.complete();
    }
  }
}

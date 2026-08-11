import 'package:tutuaword/editor/text_to_speech.dart';
import 'package:tutuaword/editor/text_to_speech_stub.dart'
    if (dart.library.html) 'package:tutuaword/editor/text_to_speech_web.dart'
    if (dart.library.io) 'package:tutuaword/editor/text_to_speech_io.dart'
    as impl;

/// Creates the platform TTS engine for the current runtime (F21.S5).
TextToSpeechEngine createPlatformTextToSpeech() =>
    impl.createPlatformTextToSpeech();

package com.manasai.tutuaword

import android.os.Build
import android.os.Bundle
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.util.Locale
import java.util.UUID

class MainActivity : FlutterActivity(), TextToSpeech.OnInitListener {
    private var tts: TextToSpeech? = null
    private var ttsReady = false
    private var pendingResult: MethodChannel.Result? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        // Android 12+ keeps the system splash until the first Flutter frame;
        // install the compat helper so it hands off cleanly.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            installSplashScreen()
        }
        super.onCreate(savedInstanceState)
    }

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        tts = TextToSpeech(this, this)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "tutuaword/tts")
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "speak" -> {
                        val text = call.argument<String>("text")?.trim().orEmpty()
                        if (text.isEmpty()) {
                            result.error("bad_args", "text required", null)
                            return@setMethodCallHandler
                        }
                        val engine = tts
                        if (!ttsReady || engine == null) {
                            result.error("not_ready", "Text-to-speech not ready", null)
                            return@setMethodCallHandler
                        }
                        // Complete any prior speak so Flutter does not hang on QUEUE_FLUSH.
                        pendingResult?.success(null)
                        pendingResult = result
                        val id = UUID.randomUUID().toString()
                        val status = engine.speak(text, TextToSpeech.QUEUE_FLUSH, null, id)
                        if (status != TextToSpeech.SUCCESS) {
                            pendingResult = null
                            result.error("speak_failed", "TTS speak failed", null)
                        }
                    }
                    "stop" -> {
                        tts?.stop()
                        pendingResult?.success(null)
                        pendingResult = null
                        result.success(null)
                    }
                    "isSpeaking" -> result.success(tts?.isSpeaking == true)
                    else -> result.notImplemented()
                }
            }
    }

    override fun onInit(status: Int) {
        ttsReady = status == TextToSpeech.SUCCESS
        if (ttsReady) {
            tts?.language = Locale.getDefault()
            tts?.setOnUtteranceProgressListener(object : UtteranceProgressListener() {
                override fun onStart(utteranceId: String?) {}
                override fun onDone(utteranceId: String?) {
                    runOnUiThread {
                        pendingResult?.success(null)
                        pendingResult = null
                    }
                }
                @Deprecated("Deprecated in Java")
                override fun onError(utteranceId: String?) {
                    runOnUiThread {
                        pendingResult?.error("speak_failed", "TTS error", null)
                        pendingResult = null
                    }
                }
            })
        }
    }

    override fun onDestroy() {
        tts?.stop()
        tts?.shutdown()
        tts = null
        super.onDestroy()
    }
}

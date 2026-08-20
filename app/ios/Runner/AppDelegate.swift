import AVFoundation
import Flutter
import UIKit

@main
@objc class AppDelegate: FlutterAppDelegate, FlutterImplicitEngineDelegate {
  override func application(
    _ application: UIApplication,
    didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
  ) -> Bool {
    return super.application(application, didFinishLaunchingWithOptions: launchOptions)
  }

  func didInitializeImplicitFlutterEngine(_ engineBridge: FlutterImplicitEngineBridge) {
    GeneratedPluginRegistrant.register(with: engineBridge.pluginRegistry)
    if let registrar = engineBridge.pluginRegistry.registrar(forPlugin: "IosTtsPlugin") {
      IosTtsPlugin.register(with: registrar)
    }
  }
}

/// Platform TTS for Read Aloud (F21.S5) using AVSpeechSynthesizer.
final class IosTtsPlugin: NSObject, FlutterPlugin, AVSpeechSynthesizerDelegate {
  private let synthesizer = AVSpeechSynthesizer()
  private var speakResult: FlutterResult?
  /// When replacing an in-flight utterance, `didCancel` must not complete the
  /// new `speak` FlutterResult while audio is still playing.
  private var ignoreNextCancel = false

  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: "tutuaword/tts",
      binaryMessenger: registrar.messenger()
    )
    let instance = IosTtsPlugin()
    instance.synthesizer.delegate = instance
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "speak":
      guard let args = call.arguments as? [String: Any],
            let text = args["text"] as? String else {
        result(
          FlutterError(code: "bad_args", message: "text required", details: nil)
        )
        return
      }
      if synthesizer.isSpeaking {
        if let pending = speakResult {
          speakResult = nil
          pending(nil)
        }
        ignoreNextCancel = true
        synthesizer.stopSpeaking(at: .immediate)
      }
      do {
        let session = AVAudioSession.sharedInstance()
        try session.setCategory(.playback, mode: .spokenAudio, options: [.duckOthers])
        try session.setActive(true)
      } catch {
        // Still attempt to speak; the synthesizer may work without a session.
      }
      let utterance = AVSpeechUtterance(string: text)
      utterance.rate = AVSpeechUtteranceDefaultSpeechRate
      speakResult = result
      synthesizer.speak(utterance)
    case "stop":
      if let pending = speakResult {
        speakResult = nil
        pending(nil)
      }
      ignoreNextCancel = synthesizer.isSpeaking
      synthesizer.stopSpeaking(at: .immediate)
      result(nil)
    case "isSpeaking":
      result(synthesizer.isSpeaking)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  func speechSynthesizer(
    _ synthesizer: AVSpeechSynthesizer,
    didFinish utterance: AVSpeechUtterance
  ) {
    if let pending = speakResult {
      speakResult = nil
      pending(nil)
    }
  }

  func speechSynthesizer(
    _ synthesizer: AVSpeechSynthesizer,
    didCancel utterance: AVSpeechUtterance
  ) {
    if ignoreNextCancel {
      ignoreNextCancel = false
      return
    }
    if let pending = speakResult {
      speakResult = nil
      pending(nil)
    }
  }
}

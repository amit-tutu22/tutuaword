import ApplicationServices
import Cocoa
import FlutterMacOS
import PDFKit

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    MacosFileAccessPlugin.register(
      with: flutterViewController.registrar(forPlugin: "MacosFileAccessPlugin")
    )
    MacosTtsPlugin.register(
      with: flutterViewController.registrar(forPlugin: "MacosTtsPlugin")
    )
    MacosPrintPlugin.register(
      with: flutterViewController.registrar(forPlugin: "MacosPrintPlugin")
    )

    self.styleMask.insert(.fullSizeContentView)
    self.titlebarAppearsTransparent = true
    self.titleVisibility = .hidden
    self.isMovableByWindowBackground = true

    super.awakeFromNib()
  }
}

class MacosFileAccessPlugin: NSObject, FlutterPlugin {
  private var accessedUrls: [String: URL] = [:]

  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: "tutuaword/macos_file_access",
      binaryMessenger: registrar.messenger
    )
    let instance = MacosFileAccessPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "createBookmark":
      createBookmark(call, result: result)
    case "startAccess":
      startAccess(call, result: result)
    case "stopAccess":
      stopAccess(call, result: result)
    case "stopAllAccess":
      stopAllAccess(result: result)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  private func createBookmark(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let path = args["path"] as? String else {
      result(FlutterError(code: "bad_args", message: "path required", details: nil))
      return
    }

    let url = URL(fileURLWithPath: path)
    do {
      let data = try url.bookmarkData(
        options: .withSecurityScope,
        includingResourceValuesForKeys: nil,
        relativeTo: nil
      )
      result(data.base64EncodedString())
    } catch {
      result(nil)
    }
  }

  private func startAccess(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let path = args["path"] as? String else {
      result(FlutterError(code: "bad_args", message: "path required", details: nil))
      return
    }

    if let existing = accessedUrls[path] {
      existing.stopAccessingSecurityScopedResource()
      accessedUrls.removeValue(forKey: path)
    }

    var resolvedUrl: URL?
    if let bookmarkBase64 = args["bookmark"] as? String,
       let data = Data(base64Encoded: bookmarkBase64) {
      var isStale = false
      do {
        resolvedUrl = try URL(
          resolvingBookmarkData: data,
          options: .withSecurityScope,
          relativeTo: nil,
          bookmarkDataIsStale: &isStale
        )
      } catch {
        resolvedUrl = nil
      }
    }

    if resolvedUrl == nil {
      resolvedUrl = URL(fileURLWithPath: path)
    }

    guard let url = resolvedUrl else {
      result(false)
      return
    }

    let ok = url.startAccessingSecurityScopedResource()
    if ok {
      accessedUrls[path] = url
    }
    result(ok)
  }

  private func stopAccess(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let path = args["path"] as? String else {
      result(FlutterError(code: "bad_args", message: "path required", details: nil))
      return
    }

    if let url = accessedUrls.removeValue(forKey: path) {
      url.stopAccessingSecurityScopedResource()
    }
    result(nil)
  }

  private func stopAllAccess(result: @escaping FlutterResult) {
    for (_, url) in accessedUrls {
      url.stopAccessingSecurityScopedResource()
    }
    accessedUrls.removeAll()
    result(nil)
  }
}

/// Platform TTS for Read Aloud (F21.S5) using NSSpeechSynthesizer.
class MacosTtsPlugin: NSObject, FlutterPlugin, NSSpeechSynthesizerDelegate {
  private let synthesizer = NSSpeechSynthesizer()
  private var speakResult: FlutterResult?
  private var ignoreNextFinish = false

  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: "tutuaword/tts",
      binaryMessenger: registrar.messenger
    )
    let instance = MacosTtsPlugin()
    instance.synthesizer.delegate = instance
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "speak":
      guard let args = call.arguments as? [String: Any],
            let text = args["text"] as? String else {
        result(FlutterError(code: "bad_args", message: "text required", details: nil))
        return
      }
      if synthesizer.isSpeaking {
        if let pending = speakResult {
          speakResult = nil
          pending(nil)
        }
        // NSSpeechSynthesizer finishes the stopped utterance asynchronously;
        // ignore that callback so it does not complete the replacement speak.
        ignoreNextFinish = true
        synthesizer.stopSpeaking()
      }
      speakResult = result
      if !synthesizer.startSpeaking(text) {
        speakResult = nil
        ignoreNextFinish = false
        result(FlutterError(code: "speak_failed", message: "NSSpeechSynthesizer failed", details: nil))
      }
      // Result completed in speechSynthesizer(_:didFinishSpeaking:).
    case "stop":
      if let pending = speakResult {
        speakResult = nil
        pending(nil)
      }
      ignoreNextFinish = synthesizer.isSpeaking
      synthesizer.stopSpeaking()
      result(nil)
    case "isSpeaking":
      result(synthesizer.isSpeaking)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  func speechSynthesizer(_ sender: NSSpeechSynthesizer, didFinishSpeaking finishedSpeaking: Bool) {
    if ignoreNextFinish {
      ignoreNextFinish = false
      return
    }
    if let pending = speakResult {
      speakResult = nil
      pending(nil)
    }
  }
}

/// OS print dialog for PDF bytes (F25.S1) via PDFKit + NSPrintOperation.
class MacosPrintPlugin: NSObject, FlutterPlugin {
  static func register(with registrar: FlutterPluginRegistrar) {
    let channel = FlutterMethodChannel(
      name: "tutuaword/print",
      binaryMessenger: registrar.messenger
    )
    let instance = MacosPrintPlugin()
    registrar.addMethodCallDelegate(instance, channel: channel)
  }

  func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "presentPrintDialog":
      presentPrintDialog(call, result: result)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  private func presentPrintDialog(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let pdfBase64 = args["pdfBase64"] as? String,
          let data = Data(base64Encoded: pdfBase64),
          let document = PDFDocument(data: data) else {
      result(FlutterError(code: "bad_args", message: "pdfBase64 required", details: nil))
      return
    }

    let jobName = (args["jobName"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines)
    let attributes = args["attributes"] as? [String: Any] ?? [:]
    let printInfo = NSPrintInfo.shared.copy() as! NSPrintInfo
    printInfo.horizontalPagination = .fit
    printInfo.verticalPagination = .automatic
    printInfo.isHorizontallyCentered = true
    printInfo.isVerticallyCentered = true

    // F25.S4 — duplex via Core Printing (NSPrintInfo has no duplex property on macOS).
    if let duplex = attributes["duplex"] as? String {
      applyDuplex(printInfo, duplex: duplex)
    }

    // PDFPrintScalingMode cases: .pageScaleNone / .pageScaleToFit / .pageScaleDownToFit.
    guard let operation = document.printOperation(
      for: printInfo,
      scalingMode: .pageScaleToFit,
      autoRotate: true
    ) else {
      result("unsupported")
      return
    }
    if let jobName, !jobName.isEmpty {
      operation.jobTitle = jobName
    }

    // Run modal print panel on the main thread.
    DispatchQueue.main.async {
      let accepted = operation.run()
      result(accepted ? "presented" : "cancelled")
    }
  }

  /// Map Flutter duplex names onto PMDuplexMode and sync back into NSPrintInfo.
  private func applyDuplex(_ printInfo: NSPrintInfo, duplex: String) {
    let mode: PMDuplexMode
    switch duplex {
    case "longEdge":
      mode = PMDuplexMode(kPMDuplexNoTumble)
    case "shortEdge":
      mode = PMDuplexMode(kPMDuplexTumble)
    default:
      mode = PMDuplexMode(kPMDuplexNone)
    }
    let settings = PMPrintSettings(printInfo.pmPrintSettings())
    if PMSetDuplex(settings, mode) == noErr {
      printInfo.updateFromPMPrintSettings()
    }
  }
}

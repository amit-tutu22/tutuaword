import Cocoa
import FlutterMacOS

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

import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/services.dart';

/// macOS sandbox file access via security-scoped bookmarks.
class MacOSFileAccess {
  MacOSFileAccess._();

  static const _channel = MethodChannel('tutuaword/macos_file_access');
  static bool _pluginMissing = false;

  /// Create a security-scoped bookmark while the app still has picker-granted access.
  static Future<String?> createBookmark(String path) async {
    if (!Platform.isMacOS || _pluginMissing) return null;
    try {
      return await _channel.invokeMethod<String>('createBookmark', {'path': path});
    } on MissingPluginException {
      _pluginMissing = true;
      return null;
    } on PlatformException {
      return null;
    }
  }

  /// Begin security-scoped access for [path], resolving [bookmark] when present.
  static Future<bool> startAccess(String path, {String? bookmark}) async {
    if (!Platform.isMacOS || _pluginMissing) return true;
    try {
      final ok = await _channel.invokeMethod<bool>('startAccess', {
        'path': path,
        'bookmark': bookmark,
      });
      return ok ?? false;
    } on MissingPluginException {
      _pluginMissing = true;
      return true;
    } on PlatformException {
      return false;
    }
  }

  static Future<void> stopAccess(String path) async {
    if (!Platform.isMacOS || _pluginMissing) return;
    try {
      await _channel.invokeMethod<void>('stopAccess', {'path': path});
    } on MissingPluginException {
      _pluginMissing = true;
    } on PlatformException {
      // Ignore — access may already be stopped.
    }
  }

  static Future<void> stopAllAccess() async {
    if (!Platform.isMacOS || _pluginMissing) return;
    try {
      await _channel.invokeMethod<void>('stopAllAccess');
    } on MissingPluginException {
      _pluginMissing = true;
    } on PlatformException {
      // Ignore.
    }
  }

  static Future<Uint8List> readFileBytes(String path, {String? bookmark}) async {
    final ok = await startAccess(path, bookmark: bookmark);
    if (!ok) {
      throw FileSystemException('Could not access file', path);
    }
    return File(path).readAsBytes();
  }
}

#!/usr/bin/env bash
# Sync iOS Flutter + CocoaPods state before opening or building in Xcode.
#
# Plugins (file_picker, …) require iOS 14. This project uses CocoaPods with
# `platform :ios, '14.0'` and disables Flutter SPM so Xcode is not stuck on the
# generated FlutterGeneratedPluginSwiftPackage default of iOS 13.0.
#
# Usage (from repo root):
#   scripts/ios-xcode-sync.sh
set -euo pipefail
cd "$(dirname "$0")/.."

step() { echo ""; echo "==> $1"; }

step "flutter pub get"
(cd app && flutter pub get)

step "pod install (iOS 14.0)"
(cd app/ios && pod install)

step "pod install (macOS)"
(cd app/macos && pod install)

echo ""
echo "Ready for Xcode."
echo "  iOS:   app/ios/Runner.xcworkspace"
echo "  macOS: app/macos/Runner.xcworkspace"
echo "(Use the .xcworkspace, not the bare .xcodeproj, so CocoaPods is linked.)"

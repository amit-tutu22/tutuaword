#!/usr/bin/env bash
# Legacy no-op: iOS plugins are installed via CocoaPods (platform :ios, '14.0').
# Flutter SPM is disabled in app/pubspec.yaml, so the old 13.0 vs 14.0
# FlutterGeneratedPluginSwiftPackage mismatch no longer applies.
#
# Prefer: scripts/ios-xcode-sync.sh
set -euo pipefail
echo "skip: iOS uses CocoaPods at 14.0 (see app/ios/Podfile); SPM patch not needed"
exit 0

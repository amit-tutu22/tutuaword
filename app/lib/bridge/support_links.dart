import 'package:flutter/foundation.dart';

/// Developer support contact.
const kSupportEmail = 'amit.blr76@gmail.com';

const kAndroidApplicationId = 'com.manasai.tutuaword';

/// Public privacy policy (GitHub Pages) — use this URL in App Store Connect.
const kPrivacyPolicyUrl =
    'https://amit-tutu22.github.io/privacy-policy/tutuaword.html';

Uri get privacyPolicyUri => Uri.parse(kPrivacyPolicyUrl);

/// Numeric Apple App Store ID (App Store Connect → App Information → Apple ID).
/// Set after the iOS or Mac App Store listing is created.
const String? kAppleAppStoreId = null;

Uri get mailtoSupportUri => Uri(
      scheme: 'mailto',
      path: kSupportEmail,
      query: _encodeQueryParameters(<String, String>{
        'subject': 'Tutuaword Support',
      }),
    );

Uri get mailtoFeedbackUri => Uri(
      scheme: 'mailto',
      path: kSupportEmail,
      query: _encodeQueryParameters(<String, String>{
        'subject': 'Tutuaword Feedback',
      }),
    );

Uri get googlePlayListingUri => Uri.https(
      'play.google.com',
      '/store/apps/details',
      <String, String>{'id': kAndroidApplicationId},
    );

Uri? get appleAppStoreReviewUri {
  final id = kAppleAppStoreId;
  if (id == null || id.trim().isEmpty) return null;
  return Uri.https(
    'apps.apple.com',
    '/app/id$id',
    <String, String>{'action': 'write-review'},
  );
}

Uri get appleAppStoreSearchUri => Uri.https(
      'apps.apple.com',
      '/search',
      <String, String>{'term': 'tutuaword'},
    );

Uri get appleAppStoreFeedbackUri =>
    appleAppStoreReviewUri ?? appleAppStoreSearchUri;

bool get showGooglePlayFeedback {
  if (kIsWeb) return false;
  return defaultTargetPlatform == TargetPlatform.android;
}

bool get showAppleStoreFeedback {
  if (kIsWeb) return false;
  return defaultTargetPlatform == TargetPlatform.iOS ||
      defaultTargetPlatform == TargetPlatform.macOS;
}

String? _encodeQueryParameters(Map<String, String> params) {
  if (params.isEmpty) return null;
  return params.entries
      .map(
        (e) =>
            '${Uri.encodeComponent(e.key)}=${Uri.encodeComponent(e.value)}',
      )
      .join('&');
}

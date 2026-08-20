import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

/// Schemes allowed when following hyperlinks from document content.
const Set<String> kAllowedDocumentLinkSchemes = {
  'http',
  'https',
  'mailto',
  'tel',
};

/// Whether [uri] is safe to open from document-originated hyperlinks.
bool isAllowedDocumentLinkUri(Uri uri) {
  final scheme = uri.scheme.toLowerCase();
  if (!kAllowedDocumentLinkSchemes.contains(scheme)) return false;
  if (scheme == 'http' || scheme == 'https') {
    return uri.host.isNotEmpty;
  }
  if (scheme == 'mailto') {
    return uri.path.isNotEmpty || (uri.queryParameters['to']?.isNotEmpty ?? false);
  }
  if (scheme == 'tel') {
    return uri.path.isNotEmpty;
  }
  return false;
}

Future<bool> openExternalUri(Uri uri) async {
  if (!isAllowedDocumentLinkUri(uri)) return false;
  if (uri.scheme.toLowerCase() == 'mailto') {
    return openEmailUri(uri);
  }
  if (!await canLaunchUrl(uri)) return false;
  return launchUrl(uri, mode: LaunchMode.externalApplication);
}

Future<bool> openEmailUri(Uri uri) async {
  if (uri.scheme.toLowerCase() != 'mailto') return false;
  if (!isAllowedDocumentLinkUri(uri)) return false;
  if (!await canLaunchUrl(uri)) return false;
  return launchUrl(uri);
}

/// Ask the user before opening a document link in an external app/browser.
Future<bool> confirmOpenDocumentLink(
  BuildContext context,
  Uri uri,
) async {
  final display = uri.toString();
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (ctx) => AlertDialog(
      key: const Key('confirm_open_link_dialog'),
      title: const Text('Open link?'),
      content: Text(
        'Open this link outside the app?\n\n$display',
        key: const Key('confirm_open_link_url'),
      ),
      actions: [
        TextButton(
          key: const Key('confirm_open_link_cancel'),
          onPressed: () => Navigator.pop(ctx, false),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('confirm_open_link_ok'),
          onPressed: () => Navigator.pop(ctx, true),
          child: const Text('Open'),
        ),
      ],
    ),
  );
  return confirmed == true;
}

Future<bool> openSupportEmail({String subject = 'Tutuaword Support'}) {
  return openEmailUri(
    Uri(
      scheme: 'mailto',
      path: 'amit.blr76@gmail.com',
      query: subject.isEmpty
          ? null
          : 'subject=${Uri.encodeComponent(subject)}',
    ),
  );
}

Future<bool> openFeedbackEmail() =>
    openSupportEmail(subject: 'Tutuaword Feedback');

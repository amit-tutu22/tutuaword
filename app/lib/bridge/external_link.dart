import 'package:url_launcher/url_launcher.dart';

Future<bool> openExternalUri(Uri uri) async {
  if (!await canLaunchUrl(uri)) return false;
  return launchUrl(uri, mode: LaunchMode.externalApplication);
}

Future<bool> openEmailUri(Uri uri) async {
  if (!await canLaunchUrl(uri)) return false;
  return launchUrl(uri);
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

/// Document canvas presentation modes (View ribbon).
enum DocumentViewLayout {
  /// Paginated editing view (default).
  printLayout,

  /// Distraction-free read-only view.
  readMode,

  /// Continuous editing canvas without page chrome.
  webLayout,
}

extension DocumentViewLayoutX on DocumentViewLayout {
  String get statusLabel => switch (this) {
        DocumentViewLayout.printLayout => 'Print layout',
        DocumentViewLayout.readMode => 'Read mode',
        DocumentViewLayout.webLayout => 'Web layout',
      };
}

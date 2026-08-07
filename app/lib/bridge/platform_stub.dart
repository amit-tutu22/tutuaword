class Platform {
  static bool isMacOS = false;
  static bool isAndroid = false;
  static bool isIOS = false;
  static bool isLinux = false;
  static String pathSeparator = '/';
  static Map<String, String> environment = {};
}

class File {
  File(String path);

  Future<List<int>> readAsBytes() {
    throw UnsupportedError('filesystem unavailable on web');
  }

  Future<void> writeAsBytes(List<int> bytes, {bool flush = false}) {
    throw UnsupportedError('filesystem unavailable on web');
  }

  bool existsSync() => false;
}

class FileSystemException implements Exception {
  FileSystemException(String message, String path);

  @override
  String toString() => 'FileSystemException';
}

class Directory {
  Directory(String path);

  String get path => '';

  Future<Directory> create({bool recursive = false}) async => this;
}

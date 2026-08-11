import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/document_print_io.dart'
    if (dart.library.html) 'package:tutuaword/bridge/document_print_web.dart'
    as impl;

/// Creates the platform print host for the current runtime (F25.S1).
DocumentPrintHost createPlatformPrintHost() => impl.createPlatformPrintHost();

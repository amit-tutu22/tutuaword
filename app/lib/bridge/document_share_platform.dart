import 'package:tutuaword/bridge/document_share.dart';
import 'package:tutuaword/bridge/document_share_io.dart'
    if (dart.library.html) 'package:tutuaword/bridge/document_share_web.dart'
    as impl;

/// Creates the platform share host for the current runtime.
DocumentShareHost createPlatformShareHost() => impl.createPlatformShareHost();

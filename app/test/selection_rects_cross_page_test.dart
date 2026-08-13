import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/engine_types.dart';

void main() {
  group('DocumentEngineSelection.projectSelectionOntoPage', () {
    final start = CaretGeometry(x: 100, y: 200, height: 14);
    final end = CaretGeometry(x: 140, y: 80, height: 14);

    test('same-page range uses both carets', () {
      final projected = DocumentEngineSelection.projectSelectionOntoPage(
        page: 1,
        startPage: 1,
        startGeom: start,
        endPage: 1,
        endGeom: end,
      );
      expect(projected, (100.0, 200.0, 140.0, 80.0));
    });

    test('focus page of cross-page range starts at top', () {
      final projected = DocumentEngineSelection.projectSelectionOntoPage(
        page: 1,
        startPage: 0,
        startGeom: start,
        endPage: 1,
        endGeom: end,
      );
      expect(projected, isNotNull);
      expect(projected!.$1, 0);
      expect(projected.$2, 0);
      expect(projected.$3, end.x);
      expect(projected.$4, end.y);
    });

    test('anchor page of cross-page range extends through bottom', () {
      final projected = DocumentEngineSelection.projectSelectionOntoPage(
        page: 0,
        startPage: 0,
        startGeom: start,
        endPage: 1,
        endGeom: end,
      );
      expect(projected, isNotNull);
      expect(projected!.$1, start.x);
      expect(projected.$2, start.y);
      expect(projected.$4, greaterThan(1e8));
    });

    test('pages outside the span yield null', () {
      expect(
        DocumentEngineSelection.projectSelectionOntoPage(
          page: 2,
          startPage: 0,
          startGeom: start,
          endPage: 1,
          endGeom: end,
        ),
        isNull,
      );
    });
  });
}

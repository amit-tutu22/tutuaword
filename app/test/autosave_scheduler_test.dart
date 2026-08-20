import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/autosave_scheduler.dart';

void main() {
  group('AutosaveScheduler', () {
    test('stop cancels pending arm and stale ticks cannot re-arm', () async {
      var ticks = 0;
      final pending = <void Function()>[];
      final scheduler = AutosaveScheduler(
        interval: const Duration(milliseconds: 10),
        onTick: () async {
          ticks++;
        },
        schedule: (_, callback) => pending.add(callback),
      );

      scheduler.start();
      expect(pending, hasLength(1));

      scheduler.stop();
      // Stale callback from the cancelled generation must not tick or re-arm.
      pending.single();
      await Future<void>.delayed(Duration.zero);
      expect(ticks, 0);
      expect(pending, hasLength(1));
    });

    test('setInterval bumps generation so prior callback is ignored', () async {
      var ticks = 0;
      final pending = <void Function()>[];
      final scheduler = AutosaveScheduler(
        interval: const Duration(milliseconds: 10),
        onTick: () async {
          ticks++;
        },
        schedule: (_, callback) => pending.add(callback),
      );

      scheduler.start();
      final first = pending.single;
      scheduler.setInterval(const Duration(milliseconds: 20));
      expect(pending, hasLength(2));

      first();
      await Future<void>.delayed(Duration.zero);
      expect(ticks, 0);

      pending.last();
      await Future<void>.delayed(Duration.zero);
      expect(ticks, 1);
    });
  });
}

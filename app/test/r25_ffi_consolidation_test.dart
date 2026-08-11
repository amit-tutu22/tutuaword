import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/command_codec.dart';
import 'package:tutuaword/bridge/format_codec.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/bridge/native_event_router.dart';

import 'native_ffi_test_helpers.dart';

void main() {
  group('format_codec', () {
    test('parses unknown char format fields without codegen', () {
      const json =
          '{"char_format":{"bold":true,"character_spacing":2.25,"future_field":"x"},'
          '"para_format":{"alignment":"Left"},"style_name":"Normal"}';
      final state = FormatCodec.fromCaretFormatJson(json);
      expect(state.charFormat.bold, isTrue);
      expect(state.charFormat.characterSpacing, 2.25);
      expect(state.charFormat['future_field'], 'x');
    });

    test('charFormatPatch accepts open map entries', () {
      final patch = FormatCodec.charFormatPatch({
        'character_spacing': 4.0,
        'experimental_flag': true,
      });
      expect(patch['character_spacing'], 4.0);
      expect(patch['experimental_flag'], isTrue);
    });
  });

  group('R2.5 FFI consolidation', () {
    test('tw_dispatch applies character_spacing via JSON patch', () async {
      final engine = NativeEngine.load();
      if (engine == null) return;
      if (!await nativeFfiEventsAvailable()) return;

      final runId = nativeDefaultRunId(engine)!;
      // Character formatting applies to a span of text, so an empty range would
      // dispatch cleanly and change nothing.
      expect(engine.tryInsertText(runId, 0, 'A'), isTrue);
      final patch = FormatCodec.charFormatPatch({'character_spacing': 5.5});
      final commands = CommandCodec.charFormatPatchCommands(
        startRunId: runId,
        startOffset: 0,
        endRunId: runId,
        endOffset: 1,
        patch: patch,
      );
      expect(commands, isNotEmpty);

      for (final command in commands) {
        expect(engine.dispatchCommand(command), 0);
      }
      // Dispatch only enqueues, so the format is not readable back until the
      // worker reports the resulting relayout.
      await NativeEventRouter.instance.waitFor(
        engine.lastRequestId(),
        timeout: const Duration(seconds: 5),
      );

      final json = engine.fetchCaretFormat(runId);
      expect(json, isNotNull);
      final state = FormatCodec.fromCaretFormatJson(json!);
      expect(state.charFormat.characterSpacing, 5.5);
    });

    test('tw_dispatch insert text command', () async {
      final engine = NativeEngine.load();
      if (engine == null) return;

      final runId = nativeDefaultRunId(engine)!;
      final ok = engine.tryInsertText(runId, 1, 'Z');
      expect(ok, isTrue);
    });
  });
}

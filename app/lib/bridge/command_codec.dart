import 'dart:convert';
import 'dart:typed_data';

/// Builds JSON wire payloads for [`tw_dispatch`] (R2.5).
///
/// Field names match Rust `Command` serde (`type` + snake_case fields).
class CommandCodec {
  CommandCodec._();

  /// End-of-run sentinel used for collapsed caret char-format (matches Rust `usize::MAX`).
  static const int runEndSentinel = 9007199254740991;

  static Uint8List encode(Map<String, dynamic> command) {
    return Uint8List.fromList(utf8.encode(jsonEncode(command)));
  }

  static Map<String, dynamic> docPosition(String runId, int charOffset) => {
        'run_id': runId,
        'char_offset': charOffset,
      };

  static Map<String, dynamic> docRange({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
  }) =>
      {
        'start': docPosition(startRunId, startOffset),
        'end': docPosition(endRunId, endOffset),
      };

  static Map<String, dynamic> insertText({
    required String runId,
    required int offset,
    required String text,
  }) =>
      {
        'type': 'InsertText',
        'run_id': runId,
        'offset': offset,
        'text': text,
      };

  static Map<String, dynamic> deleteRange({
    required String runId,
    required int start,
    required int end,
  }) =>
      {
        'type': 'DeleteRange',
        'run_id': runId,
        'start': start,
        'end': end,
      };

  static Map<String, dynamic> deleteDocRange({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
  }) =>
      {
        'type': 'DeleteDocRange',
        'range': docRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        ),
      };

  static Map<String, dynamic> splitParagraphAt({
    required String runId,
    required int offset,
  }) =>
      {
        'type': 'SplitParagraphAt',
        'run_id': runId,
        'offset': offset,
      };

  static Map<String, dynamic> setCharFormat({
    required String runId,
    required int start,
    required int end,
    required Map<String, dynamic> format,
    bool merge = true,
  }) =>
      {
        'type': 'SetCharFormat',
        'run_id': runId,
        'start': start,
        'end': end,
        'format': format,
        'merge': merge,
      };

  static Map<String, dynamic> setCharFormatRange({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required Map<String, dynamic> format,
    bool merge = true,
  }) =>
      {
        'type': 'SetCharFormatRange',
        'range': docRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        ),
        'format': format,
        'merge': merge,
      };

  static Map<String, dynamic> clearCharFormatFields({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    bool clearColor = false,
    bool clearHighlight = false,
  }) =>
      {
        'type': 'ClearCharFormatFields',
        'range': docRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        ),
        'clear_color': clearColor,
        'clear_highlight': clearHighlight,
      };

  static Map<String, dynamic> setParaFormatRange({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required Map<String, dynamic> format,
    bool merge = true,
  }) =>
      {
        'type': 'SetParaFormatRange',
        'range': docRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        ),
        'format': format,
        'merge': merge,
      };

  static Map<String, dynamic> setChartData({
    required String shapeId,
    required Map<String, dynamic> chartData,
  }) =>
      {
        'type': 'SetChartData',
        'shape_id': shapeId,
        'chart_data': chartData,
      };

  static Map<String, dynamic> ensureShapeText({required String shapeId}) => {
        'type': 'EnsureShapeText',
        'shape_id': shapeId,
      };

  static Map<String, dynamic> insertOfficeMath({
    required String runId,
    required int offset,
    required String xml,
  }) =>
      {
        'type': 'InsertOfficeMath',
        'run_id': runId,
        'offset': offset,
        'xml': xml,
      };

  static Map<String, dynamic> insertOfficeMathDisplay({
    required String afterBlockId,
    required String xml,
  }) =>
      {
        'type': 'InsertOfficeMathDisplay',
        'after_block_id': afterBlockId,
        'xml': xml,
      };

  static Map<String, dynamic> setOfficeMath({
    required String runId,
    required String xml,
  }) =>
      {
        'type': 'SetOfficeMath',
        'run_id': runId,
        'xml': xml,
      };

  /// Mirror `tw_apply_char_format` command selection for a JSON patch map.
  static List<Map<String, dynamic>> charFormatPatchCommands({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required Map<String, dynamic> patch,
  }) {
    final commands = <Map<String, dynamic>>[];
    final clearColor = patch['clear_color'] == true;
    final clearHighlight = patch['clear_highlight'] == true;
    if (clearColor || clearHighlight) {
      commands.add(clearCharFormatFields(
        startRunId: startRunId,
        startOffset: startOffset,
        endRunId: endRunId,
        endOffset: endOffset,
        clearColor: clearColor,
        clearHighlight: clearHighlight,
      ));
    }

    final formatFields = Map<String, dynamic>.from(patch)
      ..remove('clear_color')
      ..remove('clear_highlight');
    if (formatFields.isEmpty) {
      return commands;
    }

    final collapsed = startRunId == endRunId && startOffset == endOffset;
    if (collapsed) {
      commands.add(setCharFormat(
        runId: startRunId,
        start: startOffset,
        end: runEndSentinel,
        format: formatFields,
      ));
    } else if (startRunId == endRunId) {
      commands.add(setCharFormat(
        runId: startRunId,
        start: startOffset,
        end: endOffset,
        format: formatFields,
      ));
    } else {
      commands.add(setCharFormatRange(
        startRunId: startRunId,
        startOffset: startOffset,
        endRunId: endRunId,
        endOffset: endOffset,
        format: formatFields,
      ));
    }
    return commands;
  }

  static Map<String, dynamic> findReplace({
    required String startRunId,
    required int startOffset,
    required String endRunId,
    required int endOffset,
    required String find,
    required String replace,
    required bool matchCase,
    bool useRegex = false,
    bool useWildcards = false,
  }) =>
      {
        'type': 'FindReplace',
        'range': docRange(
          startRunId: startRunId,
          startOffset: startOffset,
          endRunId: endRunId,
          endOffset: endOffset,
        ),
        'find': find,
        'replace': replace,
        'match_case': matchCase,
        'use_regex': useRegex,
        'use_wildcards': useWildcards,
      };
}

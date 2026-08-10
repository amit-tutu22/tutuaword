/// Pure helpers for F28.S2 rewrite → DeleteRange / InsertText (mirrors tw-edit).

class AiReplaceCommand {
  const AiReplaceCommand.delete({
    required this.runId,
    required this.start,
    required this.end,
  })  : text = null,
        offset = null;

  const AiReplaceCommand.insert({
    required this.runId,
    required this.offset,
    required this.text,
  })  : start = null,
        end = null;

  final String runId;
  final int? start;
  final int? end;
  final int? offset;
  final String? text;

  bool get isDelete => text == null;
  bool get isInsert => text != null;
}

/// Build DeleteRange then InsertText for a same-run selection replace.
List<AiReplaceCommand> replaceRunRangeCommands({
  required String runId,
  required int start,
  required int end,
  required String text,
}) {
  final cmds = <AiReplaceCommand>[];
  if (start < end) {
    cmds.add(AiReplaceCommand.delete(runId: runId, start: start, end: end));
  }
  if (text.isNotEmpty) {
    cmds.add(AiReplaceCommand.insert(runId: runId, offset: start, text: text));
  }
  return cmds;
}

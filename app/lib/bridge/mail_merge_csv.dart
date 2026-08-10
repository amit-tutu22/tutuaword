/// Lightweight CSV parse for mail-merge data sources (F26.S2).
class MailMergeDataSource {
  const MailMergeDataSource({
    required this.headers,
    required this.rows,
  });

  final List<String> headers;
  final List<Map<String, String>> rows;

  Map<String, String>? rowAt(int index) {
    if (index < 0 || index >= rows.length) return null;
    return rows[index];
  }
}

/// Parse comma-separated CSV with optional quoted fields.
MailMergeDataSource parseMailMergeCsv(String csv) {
  final records = _parseCsvRecords(csv);
  if (records.isEmpty) {
    throw const FormatException('CSV is empty');
  }
  final headers = records.first.map((h) => h.trim()).toList();
  if (headers.isEmpty || headers.every((h) => h.isEmpty)) {
    throw const FormatException('CSV has no header row');
  }
  final rows = <Map<String, String>>[];
  for (var i = 1; i < records.length; i++) {
    final record = records[i];
    if (record.length != headers.length) {
      throw FormatException(
        'CSV row $i has ${record.length} columns, expected ${headers.length}',
      );
    }
    final map = <String, String>{};
    for (var c = 0; c < headers.length; c++) {
      map[headers[c]] = record[c];
    }
    rows.add(map);
  }
  return MailMergeDataSource(headers: headers, rows: rows);
}

List<List<String>> _parseCsvRecords(String csv) {
  final records = <List<String>>[];
  var current = <String>[];
  final field = StringBuffer();
  var inQuotes = false;
  for (var i = 0; i < csv.length; i++) {
    final c = csv[i];
    if (c == '"') {
      if (inQuotes && i + 1 < csv.length && csv[i + 1] == '"') {
        field.write('"');
        i++;
      } else {
        inQuotes = !inQuotes;
      }
      continue;
    }
    if (!inQuotes && c == ',') {
      current.add(field.toString());
      field.clear();
      continue;
    }
    if (!inQuotes && (c == '\n' || c == '\r')) {
      if (c == '\r' && i + 1 < csv.length && csv[i + 1] == '\n') {
        i++;
      }
      current.add(field.toString());
      field.clear();
      if (current.any((f) => f.isNotEmpty)) {
        records.add(current);
      }
      current = <String>[];
      continue;
    }
    field.write(c);
  }
  if (inQuotes || field.isNotEmpty || current.isNotEmpty) {
    current.add(field.toString());
    if (current.any((f) => f.isNotEmpty)) {
      records.add(current);
    }
  }
  return records;
}

/// Word-style unbound merge-field placeholder.
String mergeFieldPlaceholder(String name) => '«$name»';

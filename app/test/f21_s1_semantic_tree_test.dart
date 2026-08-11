import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('I-F21-S1-semantic-tree-from-engine', () {
    final engine = MockDocumentEngine(initialText: 'Chapter');
    engine.setSemanticTreeForTest([
      {
        'id': 'h1',
        'role': 'heading',
        'level': 0,
        'text': 'Chapter',
        'children': [
          {
            'id': 'h2',
            'role': 'heading',
            'level': 1,
            'text': 'Section',
            'children': [
              {
                'id': 'p1',
                'role': 'paragraph',
                'text': 'Body',
                'children': <Map<String, dynamic>>[],
              },
            ],
          },
        ],
      },
    ]);
    final controller = createTestEditorController(engine: engine);
    addTearDown(controller.dispose);

    final tree = controller.semanticDocumentTree;
    expect(tree, hasLength(1));
    expect(tree.first.role, 'heading');
    expect(tree.first.level, 0);
    expect(tree.first.text, 'Chapter');
    expect(tree.first.children, hasLength(1));
    expect(tree.first.children.first.level, 1);
    expect(tree.first.children.first.children.first.role, 'paragraph');
  });
}

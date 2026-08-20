import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/title_bar.dart';

import '../editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('S-P0-qat-home switches ribbon tab repeatedly', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);
    final ribbonKey = GlobalKey<WordRibbonState>();

    await tester.binding.setSurfaceSize(const Size(1200, 800));
    addTearDown(() => tester.binding.setSurfaceSize(null));

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Column(
            children: [
              WordTitleBar(
                controller: controller,
                onHomePressed: () =>
                    ribbonKey.currentState?.selectTab(RibbonTab.home),
              ),
              WordRibbon(key: ribbonKey, controller: controller),
            ],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Insert'));
    await tester.pump();
    expect(find.byType(HomeTab), findsNothing);

    for (var i = 0; i < 20; i++) {
      await tester.tap(find.byIcon(Icons.home_outlined));
      await tester.pump();
      expect(find.byType(HomeTab), findsOneWidget);
      await tester.tap(find.text('Insert'));
      await tester.pump();
    }
  });
}

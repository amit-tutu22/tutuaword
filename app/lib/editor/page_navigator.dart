import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Page thumbnail strip for multi-page navigation.
class PageNavigator extends StatelessWidget {
  const PageNavigator({
    super.key,
    required this.controller,
    required this.currentPage,
    required this.onPageSelected,
  });

  final EditorController controller;
  final int currentPage;
  final ValueChanged<int> onPageSelected;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: ListView.builder(
        padding: const EdgeInsets.all(8),
        itemCount: controller.pageCount,
        itemBuilder: (context, index) {
          final selected = index == currentPage;
          return Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: InkWell(
              onTap: () => onPageSelected(index),
              child: Container(
                height: 100,
                decoration: BoxDecoration(
                  color: Colors.white,
                  border: Border.all(
                    color: selected
                        ? Theme.of(context).colorScheme.primary
                        : Colors.grey.shade400,
                    width: selected ? 2 : 1,
                  ),
                  boxShadow: selected
                      ? [
                          BoxShadow(
                            color: Colors.black.withValues(alpha: 0.1),
                            blurRadius: 4,
                          ),
                        ]
                      : null,
                ),
                child: Center(
                  child: Text(
                    '${index + 1}',
                    style: TextStyle(
                      fontWeight: selected ? FontWeight.bold : FontWeight.normal,
                    ),
                  ),
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

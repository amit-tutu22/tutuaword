/// Built-in symbol categories for the Insert Symbol dialog (F15.S1).
class SymbolEntry {
  const SymbolEntry({
    required this.id,
    required this.character,
    this.description,
  });

  final String id;
  final String character;
  final String? description;
}

class SymbolCategory {
  const SymbolCategory({
    required this.id,
    required this.name,
    required this.symbols,
  });

  final String id;
  final String name;
  final List<SymbolEntry> symbols;
}

abstract final class SymbolCatalog {
  SymbolCatalog._();

  static const special = SymbolCategory(
    id: 'special',
    name: 'Special Characters',
    symbols: [
      SymbolEntry(id: 'copyright', character: '©', description: 'Copyright'),
      SymbolEntry(id: 'registered', character: '®', description: 'Registered'),
      SymbolEntry(id: 'trademark', character: '™', description: 'Trademark'),
      SymbolEntry(id: 'section', character: '§', description: 'Section'),
      SymbolEntry(id: 'paragraph', character: '¶', description: 'Pilcrow'),
      SymbolEntry(id: 'dagger', character: '†', description: 'Dagger'),
      SymbolEntry(id: 'double_dagger', character: '‡', description: 'Double dagger'),
      SymbolEntry(id: 'bullet', character: '•', description: 'Bullet'),
      SymbolEntry(id: 'ellipsis', character: '…', description: 'Ellipsis'),
      SymbolEntry(id: 'em_dash', character: '—', description: 'Em dash'),
      SymbolEntry(id: 'en_dash', character: '–', description: 'En dash'),
    ],
  );

  static const currency = SymbolCategory(
    id: 'currency',
    name: 'Currency',
    symbols: [
      SymbolEntry(id: 'dollar', character: '\$', description: 'Dollar'),
      SymbolEntry(id: 'euro', character: '€', description: 'Euro'),
      SymbolEntry(id: 'pound', character: '£', description: 'Pound'),
      SymbolEntry(id: 'yen', character: '¥', description: 'Yen'),
      SymbolEntry(id: 'cent', character: '¢', description: 'Cent'),
      SymbolEntry(id: 'won', character: '₩', description: 'Won'),
      SymbolEntry(id: 'rupee', character: '₹', description: 'Rupee'),
    ],
  );

  static const math = SymbolCategory(
    id: 'math',
    name: 'Mathematical',
    symbols: [
      SymbolEntry(id: 'plus_minus', character: '±', description: 'Plus-minus'),
      SymbolEntry(id: 'multiply', character: '×', description: 'Multiply'),
      SymbolEntry(id: 'divide', character: '÷', description: 'Divide'),
      SymbolEntry(id: 'not_equal', character: '≠', description: 'Not equal'),
      SymbolEntry(id: 'less_equal', character: '≤', description: 'Less or equal'),
      SymbolEntry(id: 'greater_equal', character: '≥', description: 'Greater or equal'),
      SymbolEntry(id: 'approx', character: '≈', description: 'Approximately'),
      SymbolEntry(id: 'infinity', character: '∞', description: 'Infinity'),
      SymbolEntry(id: 'sqrt', character: '√', description: 'Square root'),
      SymbolEntry(id: 'sum', character: '∑', description: 'Summation'),
      SymbolEntry(id: 'integral', character: '∫', description: 'Integral'),
      SymbolEntry(id: 'partial', character: '∂', description: 'Partial derivative'),
      SymbolEntry(id: 'pi', character: 'π', description: 'Pi'),
      SymbolEntry(id: 'degree', character: '°', description: 'Degree'),
    ],
  );

  /// Extended math / Greek subset (F15.S2).
  static const mathGreek = SymbolCategory(
    id: 'math_greek',
    name: 'Greek & Advanced',
    symbols: [
      SymbolEntry(id: 'alpha', character: 'α', description: 'Alpha'),
      SymbolEntry(id: 'beta', character: 'β', description: 'Beta'),
      SymbolEntry(id: 'gamma', character: 'γ', description: 'Gamma'),
      SymbolEntry(id: 'delta', character: 'δ', description: 'Delta'),
      SymbolEntry(id: 'epsilon', character: 'ε', description: 'Epsilon'),
      SymbolEntry(id: 'theta', character: 'θ', description: 'Theta'),
      SymbolEntry(id: 'lambda', character: 'λ', description: 'Lambda'),
      SymbolEntry(id: 'mu', character: 'μ', description: 'Mu'),
      SymbolEntry(id: 'sigma', character: 'σ', description: 'Sigma'),
      SymbolEntry(id: 'omega', character: 'ω', description: 'Omega'),
      SymbolEntry(id: 'capital_delta', character: 'Δ', description: 'Capital Delta'),
      SymbolEntry(id: 'capital_sigma', character: 'Σ', description: 'Capital Sigma'),
      SymbolEntry(id: 'capital_omega', character: 'Ω', description: 'Capital Omega'),
      SymbolEntry(id: 'forall', character: '∀', description: 'For all'),
      SymbolEntry(id: 'exists', character: '∃', description: 'There exists'),
      SymbolEntry(id: 'element_of', character: '∈', description: 'Element of'),
      SymbolEntry(id: 'not_element', character: '∉', description: 'Not element of'),
      SymbolEntry(id: 'subset', character: '⊂', description: 'Subset'),
      SymbolEntry(id: 'union', character: '∪', description: 'Union'),
      SymbolEntry(id: 'intersection', character: '∩', description: 'Intersection'),
      SymbolEntry(id: 'empty_set', character: '∅', description: 'Empty set'),
      SymbolEntry(id: 'nabla', character: '∇', description: 'Nabla'),
    ],
  );

  /// Common emoji subset (F15.S2).
  static const emoji = SymbolCategory(
    id: 'emoji',
    name: 'Emoji',
    symbols: [
      SymbolEntry(id: 'grinning', character: '😀', description: 'Grinning face'),
      SymbolEntry(id: 'smile', character: '🙂', description: 'Smiling face'),
      SymbolEntry(id: 'joy', character: '😂', description: 'Face with tears of joy'),
      SymbolEntry(id: 'wink', character: '😉', description: 'Winking face'),
      SymbolEntry(id: 'thinking', character: '🤔', description: 'Thinking face'),
      SymbolEntry(id: 'thumbs_up', character: '👍', description: 'Thumbs up'),
      SymbolEntry(id: 'thumbs_down', character: '👎', description: 'Thumbs down'),
      SymbolEntry(id: 'clap', character: '👏', description: 'Clapping hands'),
      SymbolEntry(id: 'heart', character: '❤', description: 'Red heart'),
      SymbolEntry(id: 'star', character: '⭐', description: 'Star'),
      SymbolEntry(id: 'check', character: '✅', description: 'Check mark'),
      SymbolEntry(id: 'cross_mark', character: '❌', description: 'Cross mark'),
      SymbolEntry(id: 'warning', character: '⚠', description: 'Warning'),
      SymbolEntry(id: 'fire', character: '🔥', description: 'Fire'),
      SymbolEntry(id: 'party', character: '🎉', description: 'Party popper'),
      SymbolEntry(id: 'light_bulb', character: '💡', description: 'Light bulb'),
      SymbolEntry(id: 'rocket', character: '🚀', description: 'Rocket'),
      SymbolEntry(id: 'sparkles', character: '✨', description: 'Sparkles'),
    ],
  );

  static const arrows = SymbolCategory(
    id: 'arrows',
    name: 'Arrows',
    symbols: [
      SymbolEntry(id: 'arrow_left', character: '←', description: 'Left arrow'),
      SymbolEntry(id: 'arrow_right', character: '→', description: 'Right arrow'),
      SymbolEntry(id: 'arrow_up', character: '↑', description: 'Up arrow'),
      SymbolEntry(id: 'arrow_down', character: '↓', description: 'Down arrow'),
      SymbolEntry(id: 'arrow_lr', character: '↔', description: 'Left-right arrow'),
      SymbolEntry(id: 'arrow_double_right', character: '⇒', description: 'Double right arrow'),
    ],
  );

  static const List<SymbolCategory> categories = [
    special,
    currency,
    math,
    mathGreek,
    emoji,
    arrows,
  ];

  static bool isEmojiCategory(String categoryId) => categoryId == emoji.id;

  static List<SymbolEntry> emojiSymbols() => emoji.symbols;

  static List<SymbolEntry> mathExtendedSymbols() => mathGreek.symbols;

  static SymbolEntry? findById(String id) {
    for (final category in categories) {
      for (final entry in category.symbols) {
        if (entry.id == id) return entry;
      }
    }
    return null;
  }

  static SymbolEntry? findByCharacter(String character) {
    for (final entry in allSymbols()) {
      if (entry.character == character) return entry;
    }
    return null;
  }

  static List<SymbolEntry> allSymbols() => [
        for (final category in categories) ...category.symbols,
      ];
}

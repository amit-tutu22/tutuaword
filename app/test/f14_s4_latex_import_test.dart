import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/equation_omml.dart';
import 'package:tutuaword/editor/latex_omml.dart';

void main() {
  group('F14.S4 LaTeX import', () {
    test('U-F14-S4-latex-frac', () {
      final omml = LatexOmml.convert(r'\frac{a}{b}');
      expect(omml, isNotNull);
      expect(omml, contains('<m:f>'));
      expect(omml, contains('<m:num>'));
    });

    test('U-F14-S4-latex-superscript', () {
      final omml = LatexOmml.convert('x^2');
      expect(omml, contains('<m:sSup>'));
    });

    test('U-F14-S4-latex-greek', () {
      final omml = LatexOmml.convert(r'\alpha+\beta');
      expect(omml, contains('α'));
      expect(omml, contains('β'));
    });

    test('U-F14-S4-latex-unknown-command', () {
      expect(LatexOmml.convert(r'\foo'), isNull);
      expect(LatexOmml.lastError, contains('Unknown'));
    });

    test('U-F14-S4-equation-model-latex-to-omml', () {
      final model = EquationModel.latex(source: r'\frac{1}{2}', display: true);
      final omml = model.toOmml();
      expect(omml, contains('<m:oMathPara'));
      expect(omml, contains('<m:f>'));
    });
  });
}

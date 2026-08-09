//! LaTeX → OMML subset converter (F14.S4).

use crate::math_preview::{build_display_omath_para, build_inline_omath, omml_run};

/// Errors from the LaTeX subset parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LatexError {
    EmptyInput,
    UnexpectedEnd,
    UnknownCommand(String),
    ExpectedArgument,
    TrailingInput,
}

/// Convert LaTeX math to inline `<m:oMath>` XML.
pub fn latex_to_inline_omath(latex: &str) -> Result<String, LatexError> {
    let content = latex_to_omml_content(latex)?;
    Ok(build_inline_omath(&content))
}

/// Convert LaTeX math to display `<m:oMathPara>` XML.
pub fn latex_to_display_omath_para(latex: &str) -> Result<String, LatexError> {
    let content = latex_to_omml_content(latex)?;
    Ok(build_display_omath_para(&build_inline_omath(&content)))
}

/// Convert LaTeX math to OMML inner content (runs/structures, no wrapper).
pub fn latex_to_omml_content(latex: &str) -> Result<String, LatexError> {
    let trimmed = latex.trim();
    if trimmed.is_empty() {
        return Err(LatexError::EmptyInput);
    }
    let mut parser = Parser::new(trimmed);
    let content = parser.parse_sequence()?;
    parser.skip_ws();
    if !parser.at_end() {
        return Err(LatexError::TrailingInput);
    }
    Ok(content)
}

struct Parser<'a> {
    input: &'a str,
    chars: std::str::Chars<'a>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars(),
            pos: 0,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.clone().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }

    fn at_group_end(&self) -> bool {
        self.chars.clone().next().is_none_or(|c| c == '}')
    }

    fn parse_sequence(&mut self) -> Result<String, LatexError> {
        let mut parts = Vec::new();
        self.skip_ws();
        while !self.at_end() && !self.at_group_end() {
            let atom = self.parse_atom()?;
            parts.push(self.apply_scripts(atom)?);
            self.skip_ws();
        }
        if parts.is_empty() {
            return Err(LatexError::ExpectedArgument);
        }
        Ok(parts.join(""))
    }

    fn parse_atom(&mut self) -> Result<String, LatexError> {
        self.skip_ws();
        let ch = self.peek().ok_or(LatexError::UnexpectedEnd)?;
        match ch {
            '{' => {
                self.bump();
                let inner = self.parse_sequence()?;
                if self.bump() != Some('}') {
                    return Err(LatexError::UnexpectedEnd);
                }
                Ok(inner)
            }
            '\\' => self.parse_command(),
            '^' | '_' | '}' => Err(LatexError::ExpectedArgument),
            _ => self.parse_text(),
        }
    }

    fn parse_text(&mut self) -> Result<String, LatexError> {
        let ch = self.peek().ok_or(LatexError::ExpectedArgument)?;
        // Scripts bind to the preceding atom; keep atoms small.
        let text = if ch.is_ascii_alphanumeric() {
            self.bump().unwrap().to_string()
        } else {
            self.bump().unwrap().to_string()
        };
        Ok(omml_run(&text))
    }

    fn parse_command(&mut self) -> Result<String, LatexError> {
        self.bump(); // backslash
        let name = self.parse_command_name()?;
        match name.as_str() {
            "frac" => self.parse_frac(),
            "sqrt" => self.parse_sqrt(),
            "text" => {
                let inner = self.parse_group_content()?;
                Ok(omml_run(&inner))
            }
            "left" | "right" | "cdot" => {
                // `\left`/`\right` delimiters and `\cdot` → usable symbols when alone.
                if name == "cdot" {
                    return Ok(omml_run("·"));
                }
                self.skip_ws();
                if matches!(self.peek(), Some('(' | ')' | '[' | ']' | '|' | '.')) {
                    let ch = self.bump().unwrap();
                    let sym = match ch {
                        '(' => "(",
                        ')' => ")",
                        '[' => "[",
                        ']' => "]",
                        '|' => "|",
                        '.' => "",
                        _ => unreachable!(),
                    };
                    return Ok(omml_run(sym));
                }
                Ok(String::new())
            }
            _ => command_symbol(&name)
                .map(|s| omml_run(s))
                .ok_or(LatexError::UnknownCommand(name)),
        }
    }

    fn parse_command_name(&mut self) -> Result<String, LatexError> {
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                name.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(LatexError::ExpectedArgument);
        }
        Ok(name)
    }

    fn parse_group_content(&mut self) -> Result<String, LatexError> {
        self.skip_ws();
        if self.bump() != Some('{') {
            return Err(LatexError::ExpectedArgument);
        }
        let xml = self.parse_sequence()?;
        if self.bump() != Some('}') {
            return Err(LatexError::UnexpectedEnd);
        }
        Ok(crate::math_preview::extract_omml_preview_text(&xml))
    }

    fn parse_frac(&mut self) -> Result<String, LatexError> {
        self.skip_ws();
        let num = self.parse_atom()?;
        self.skip_ws();
        let den = self.parse_atom()?;
        Ok(format!(
            "<m:f><m:num>{num}</m:num><m:den>{den}</m:den></m:f>"
        ))
    }

    fn parse_sqrt(&mut self) -> Result<String, LatexError> {
        self.skip_ws();
        let degree = if self.peek() == Some('[') {
            self.bump();
            let mut deg = String::new();
            while let Some(ch) = self.peek() {
                if ch == ']' {
                    self.bump();
                    break;
                }
                deg.push(self.bump().unwrap());
            }
            omml_run(&deg)
        } else {
            String::from("<m:deg/>")
        };
        self.skip_ws();
        let body = self.parse_atom()?;
        Ok(format!("<m:rad>{degree}<m:e>{body}</m:e></m:rad>"))
    }

    fn apply_scripts(&mut self, base: String) -> Result<String, LatexError> {
        let mut sup: Option<String> = None;
        let mut sub: Option<String> = None;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('^') => {
                    self.bump();
                    sup = Some(self.parse_atom()?);
                }
                Some('_') => {
                    self.bump();
                    sub = Some(self.parse_atom()?);
                }
                _ => break,
            }
        }
        match (sup, sub) {
            (None, None) => Ok(base),
            (Some(s), None) => Ok(format!("<m:sSup><m:e>{base}</m:e><m:sup>{s}</m:sup></m:sSup>")),
            (None, Some(s)) => Ok(format!("<m:sSub><m:e>{base}</m:e><m:sub>{s}</m:sub></m:sSub>")),
            (Some(sup), Some(sub)) => Ok(format!(
                "<m:sSubSup><m:e>{base}</m:e><m:sub>{sub}</m:sub><m:sup>{sup}</m:sup></m:sSubSup>"
            )),
        }
    }
}

fn command_symbol(name: &str) -> Option<&'static str> {
    match name {
        "alpha" => Some("α"),
        "beta" => Some("β"),
        "gamma" => Some("γ"),
        "pi" => Some("π"),
        "Sigma" | "sum" => Some("Σ"),
        "infty" => Some("∞"),
        "pm" => Some("±"),
        "times" => Some("×"),
        "div" => Some("÷"),
        "leq" | "le" => Some("≤"),
        "geq" | "ge" => Some("≥"),
        "neq" | "ne" => Some("≠"),
        "int" => Some("∫"),
        "partial" => Some("∂"),
        "theta" => Some("θ"),
        "Delta" => Some("Δ"),
        "lambda" => Some("λ"),
        "mu" => Some("μ"),
        "cdot" => Some("·"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math_preview::extract_omml_preview_text;

    #[test]
    fn latex_plain_text() {
        let xml = latex_to_inline_omath("E=mc^2").unwrap();
        assert!(xml.contains("<m:oMath"));
        assert!(extract_omml_preview_text(&xml).contains('E'));
    }

    #[test]
    fn latex_frac() {
        let xml = latex_to_inline_omath(r"\frac{a}{b}").unwrap();
        assert!(xml.contains("<m:f>"));
        assert!(xml.contains("<m:num>"));
        assert_eq!(extract_omml_preview_text(&xml), "ab");
    }

    #[test]
    fn latex_superscript() {
        let xml = latex_to_inline_omath("x^2").unwrap();
        assert!(xml.contains("<m:sSup>"));
        assert_eq!(extract_omml_preview_text(&xml), "x2");
    }

    #[test]
    fn latex_subsup() {
        let xml = latex_to_inline_omath("x^2_i").unwrap();
        assert!(xml.contains("<m:sSubSup>"));
        assert_eq!(extract_omml_preview_text(&xml), "xi2");
    }

    #[test]
    fn latex_sqrt() {
        let xml = latex_to_inline_omath(r"\sqrt{x}").unwrap();
        assert!(xml.contains("<m:rad>"));
        assert_eq!(extract_omml_preview_text(&xml), "x");
    }

    #[test]
    fn latex_greek() {
        let xml = latex_to_inline_omath(r"\alpha+\beta").unwrap();
        let text = extract_omml_preview_text(&xml);
        assert!(text.contains('α'));
        assert!(text.contains('β'));
    }

    #[test]
    fn latex_unknown_command() {
        assert_eq!(
            latex_to_inline_omath(r"\unknown{x}"),
            Err(LatexError::UnknownCommand("unknown".into()))
        );
    }

    #[test]
    fn latex_display_wrapper() {
        let xml = latex_to_display_omath_para(r"\frac{1}{2}").unwrap();
        assert!(xml.contains("<m:oMathPara"));
    }
}

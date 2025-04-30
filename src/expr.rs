use std::rc::Rc;

use crate::{Parser, alt, lit, many, opt, rule_ref, satisfy_dyn, seq};

/// An expression representation for parser combinators.
/// Enables building parser ASTs that can be compiled into actual parsers.
#[derive(Clone)]
pub enum ParserExpr {
    Literal(String),
    Satisfy(Rc<dyn Fn(char) -> bool>),
    Seq(Vec<ParserExpr>),
    Alt(Vec<ParserExpr>),
    Opt(Box<ParserExpr>),
    Many(Box<ParserExpr>),
    Ref(String),
}

impl ParserExpr {
    /// Compile the expression into a parser.
    pub fn compile(self) -> Parser {
        match self {
            ParserExpr::Literal(s) => lit(&s),
            ParserExpr::Satisfy(f) => satisfy_dyn(f),
            ParserExpr::Seq(exprs) => {
                let parsers: Vec<Parser> = exprs.into_iter().map(|e| e.compile()).collect();
                seq(parsers)
            }
            ParserExpr::Alt(exprs) => {
                let mut it = exprs.into_iter().map(|e| e.compile());
                let first = it
                    .next()
                    .expect("Alt requires at least one alternative expected");
                it.fold(first, |acc, p| alt(acc, p))
            }
            ParserExpr::Opt(expr) => opt(expr.compile()),
            ParserExpr::Many(expr) => many(expr.compile()),
            ParserExpr::Ref(name) => rule_ref(name),
        }
    }
}

/// Creates a ParserExpr that references a named rule.
///
/// This is a shorthand for writing `ParserExpr::Ref(name.to_string())`
/// and is useful for constructing grammars with macros or concise expressions.
///
/// # Example
/// ```ignore
/// let expr = seq![ref_(\"String\"), lit(\":\"), ref_(\"Value\")];
/// ```
pub fn ref_(name: &str) -> ParserExpr {
    ParserExpr::Ref(name.to_string())
}

/// Creates a ParserExpr that matches a literal string.
pub fn lit_(s: &str) -> ParserExpr {
    ParserExpr::Literal(s.to_string())
}

/// Creates a ParserExpr that matches a character satisfying a predicate.
pub fn satisfy_(f: impl Fn(char) -> bool + 'static) -> ParserExpr {
    ParserExpr::Satisfy(Rc::new(f))
}

#[cfg(test)]
mod expr_tests {
    use super::*;
    use crate::{Grammar, ParseResult, Value, satisfy};

    #[test]
    fn test_satisfy_dyn_matches_digit() {
        let pred = Rc::new(|c: char| c.is_ascii_digit());
        let parser = satisfy_dyn(pred);
        let mut g = Grammar::new();

        let result = parser(&mut g, "3abc");
        assert!(result.is_ok());

        let ParseResult { value, rest } = result.unwrap();
        assert_eq!(value, Value::Char('3'));
        assert_eq!(rest, "abc");
    }

    #[test]
    fn test_ref_expr_compile() {
        let mut g = Grammar::new();
        g.define("digit", satisfy(|c| c.is_ascii_digit()));

        let expr = ref_("digit").compile();
        let result = expr(&mut g, "7x");
        assert!(result.is_ok());

        let ParseResult { value, rest } = result.unwrap();
        assert_eq!(value, Value::Char('7'));
        assert_eq!(rest, "x");
    }

    #[test]
    fn test_lit_expr_compile() {
        let mut g = Grammar::new();
        let expr = ParserExpr::Literal("hi".into()).compile();

        let result = expr(&mut g, "hi!");
        assert!(result.is_ok());

        let ParseResult { value, rest } = result.unwrap();
        assert_eq!(value, Value::Str("hi".into()));
        assert_eq!(rest, "!");
    }

    #[test]
    fn test_seq_macro_compile() {
        let mut g = Grammar::new();
        let expr = seq![
            ParserExpr::Literal("a".into()),
            ParserExpr::Literal("b".into())
        ];
        let parser = expr.compile();
        let result = parser(&mut g, "abc");
        assert!(result.is_ok());

        let ParseResult { value, rest } = result.unwrap();
        assert_eq!(rest, "c");
        assert_eq!(
            value,
            Value::List(vec![Value::Str("a".into()), Value::Str("b".into())])
        );
    }

    #[test]
    fn test_alt_macro_compile() {
        let mut g = Grammar::new();
        let expr = alt![
            ParserExpr::Literal("a".into()),
            ParserExpr::Literal("b".into())
        ];
        let parser = expr.compile();

        let result = parser(&mut g, "bcd");
        assert!(result.is_ok());

        let ParseResult { value, rest } = result.unwrap();
        assert_eq!(rest, "cd");
        assert_eq!(value, Value::Str("b".into()));
    }
}

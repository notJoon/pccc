use std::{fmt, rc::Rc};

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
        match self.optimize() {
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

    /// Optimize the parser expression tree by:
    /// 1. Flattening nested [ParserExpr::Seq] and [ParserExpr::Alt] nodes
    /// 2. Removing trivial layers
    ///
    /// ## Example
    /// ```ignore
    /// // Before optimization:
    /// seq![
    ///     lit_("a"),
    ///     seq![
    ///         lit_("b"),
    ///         seq![lit_("c"), lit_("d")]
    ///     ]
    /// ]
    ///
    /// // After optimization:
    /// seq![lit_("a"), lit_("b"), lit_("c"), lit_("d")]
    /// ```
    pub fn optimize(self) -> ParserExpr {
        match self {
            ParserExpr::Seq(exprs) => {
                let mut optimized = Vec::new();
                for expr in exprs {
                    match expr.optimize() {
                        ParserExpr::Seq(mut nested) => optimized.append(&mut nested),
                        other => optimized.push(other),
                    }
                }
                match optimized.len() {
                    0 => panic!("Empty sequence is not allowed"),
                    1 => optimized.pop().unwrap(),
                    _ => ParserExpr::Seq(optimized),
                }
            }
            ParserExpr::Alt(exprs) => {
                let mut optimized = Vec::new();
                for expr in exprs {
                    match expr.optimize() {
                        ParserExpr::Alt(mut nested) => optimized.append(&mut nested),
                        other => optimized.push(other),
                    }
                }
                match optimized.len() {
                    0 => panic!("Empty alternative is not allowed"),
                    1 => optimized.pop().unwrap(),
                    _ => ParserExpr::Alt(optimized),
                }
            }
            ParserExpr::Opt(expr) => ParserExpr::Opt(Box::new(expr.optimize())),
            ParserExpr::Many(expr) => ParserExpr::Many(Box::new(expr.optimize())),
            // don't need optimization
            ParserExpr::Literal(_) | ParserExpr::Satisfy(_) | ParserExpr::Ref(_) => self,
        }
    }
}

impl fmt::Debug for ParserExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserExpr::Literal(s) => write!(f, "Literal({})", s),
            ParserExpr::Satisfy(_) => write!(f, "Satisfy"),
            ParserExpr::Seq(exprs) => write!(f, "Seq({:?})", exprs),
            ParserExpr::Alt(exprs) => write!(f, "Alt({:?})", exprs),
            ParserExpr::Opt(expr) => write!(f, "Opt({:?})", expr),
            ParserExpr::Many(expr) => write!(f, "Many({:?})", expr),
            ParserExpr::Ref(name) => write!(f, "Ref({})", name),
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
#[inline]
pub fn ref_(name: &str) -> ParserExpr {
    ParserExpr::Ref(name.to_string())
}

/// Creates a ParserExpr that matches a literal string.
#[inline]
pub fn lit_(s: &str) -> ParserExpr {
    ParserExpr::Literal(s.to_string())
}

/// Creates a ParserExpr that matches a character satisfying a predicate.
#[inline]
pub fn satisfy_(f: impl Fn(char) -> bool + 'static) -> ParserExpr {
    ParserExpr::Satisfy(Rc::new(f))
}

#[cfg(test)]
mod expr_tests {
    use super::*;
    use crate::{Grammar, ParseResult, satisfy};

    #[test]
    fn test_satisfy_dyn_matches_digit() {
        let pred = Rc::new(|c: char| c.is_ascii_digit());
        let parser = satisfy_dyn(pred);
        let mut g = Grammar::new();

        let result = parser(&mut g, "3abc");
        assert!(result.is_ok());

        let ParseResult { node, rest } = result.unwrap();
        assert_eq!(node.value, "3");
        assert_eq!(rest, "abc");
    }

    #[test]
    fn test_ref_expr_compile() {
        let mut g = Grammar::new();
        g.define("digit", satisfy(|c| c.is_ascii_digit()));

        let expr = ref_("digit").compile();
        let result = expr(&mut g, "7x");
        assert!(result.is_ok());

        let ParseResult { node, rest } = result.unwrap();
        assert_eq!(node.value, "7");
        assert_eq!(rest, "x");
    }

    #[test]
    fn test_lit_expr_compile() {
        let mut g = Grammar::new();
        let expr = ParserExpr::Literal("hi".into()).compile();

        let result = expr(&mut g, "hi!");
        assert!(result.is_ok());

        let ParseResult { node, rest } = result.unwrap();
        assert_eq!(node.value, "hi");
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

        let ParseResult { node, rest } = result.unwrap();
        assert_eq!(rest, "c");
        assert_eq!(node.value, "ab");
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

        let ParseResult { node, rest } = result.unwrap();
        assert_eq!(rest, "cd");
        assert_eq!(node.value, "b");
    }

    #[test]
    fn test_optimize_flattens_nested_seq() {
        let nested = seq![lit_("a"), seq![lit_("b"), lit_("c")], lit_("d")];
        let optimized = nested.optimize();

        if let ParserExpr::Seq(exprs) = optimized {
            assert_eq!(exprs.len(), 4); // Should be flattened to a,b,c,d
        } else {
            panic!("Expected Seq");
        }
    }

    #[test]
    fn test_optimize_flattens_nested_alt() {
        let nested = alt![lit_("a"), alt![lit_("b"), lit_("c")], lit_("d")];
        let optimized = nested.optimize();

        if let ParserExpr::Alt(exprs) = optimized {
            assert_eq!(exprs.len(), 4); // Should be flattened to a|b|c|d
        } else {
            panic!("Expected Alt");
        }
    }

    #[test]
    fn test_optimize_removes_trivial_layers() {
        let trivial = seq![lit_("a")];
        let optimized = trivial.optimize();

        match optimized {
            ParserExpr::Literal(_) => (), // Success
            _ => panic!("Expected trivial Seq to be removed"),
        }
    }

    #[test]
    #[should_panic(expected = "Empty sequence is not allowed")]
    fn optimize_empty_seq_panics() {
        // Seq with no children should panic
        let expr = ParserExpr::Seq(vec![]);
        let _ = expr.optimize();
    }

    #[test]
    #[should_panic(expected = "Empty alternative is not allowed")]
    fn optimize_empty_alt_panics() {
        // Alt with no children should panic
        let expr = ParserExpr::Alt(vec![]);
        let _ = expr.optimize();
    }

    #[test]
    fn optimize_flattens_inside_opt_and_many() {
        // Even inside Opt/Many we recurse
        let expr = ParserExpr::Opt(Box::new(seq![lit_("x"), seq![lit_("y"), lit_("z")]]));
        let optimized = expr.optimize();
        if let ParserExpr::Opt(inner) = optimized {
            // inner should have been flattened to Seq([x,y,z])
            if let ParserExpr::Seq(v) = *inner {
                assert_eq!(v.len(), 3);
            } else {
                panic!("Expected inner Seq");
            }
        } else {
            panic!("Expected Opt");
        }

        let expr = ParserExpr::Many(Box::new(alt![lit_("a"), alt![lit_("b"), lit_("c")]]));
        let optimized = expr.optimize();
        if let ParserExpr::Many(inner) = optimized {
            if let ParserExpr::Alt(v) = *inner {
                assert_eq!(v.len(), 3);
            } else {
                panic!("Expected inner Alt");
            }
        } else {
            panic!("Expected Many");
        }
    }
}

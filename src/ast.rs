#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// The kind of a parse node
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    /// A named non-terminal rule.
    NonTerminal(String),
    /// A terminal token
    /// either a literal string or a single character.
    Terminal(String),
    /// A sequence of nodes
    Sequence(Vec<Node>),
    /// An alternative node (one of many)
    Alternative(Vec<Node>),
    /// Zero or more repetitions
    Many(Vec<Node>),
    /// Optional node
    Optional(Box<Node>),
    /// A character that satisfies a predicate
    Satisfy(String), // description of the predicate
}

/// A node in the abstract syntax tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub span: Span,
    pub value: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(kind: NodeKind, span: Span, value: String) -> Self {
        Self {
            kind,
            span,
            value,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.kind, NodeKind::Terminal(_))
    }

    pub fn is_non_terminal(&self) -> bool {
        matches!(self.kind, NodeKind::NonTerminal(_))
    }
}

#[cfg(test)]
mod ast_tests {
    use super::*;
    use crate::ParseResult;
    use crate::{Grammar, digit, many};
    use crate::{lit, opt, satisfy, seq};

    #[test]
    fn test_literal_ast() {
        let mut g = Grammar::new();
        // define a literal rule "foo"
        g.define("L", lit("foo"));

        let ParseResult { node, rest } = g.parse("L", "foobar").unwrap();

        assert_eq!(node.kind, NodeKind::Terminal("foo".to_string()));
        assert_eq!(node.value, "foo");
        assert_eq!(node.span, Span { start: 0, end: 3 });
        assert_eq!(rest, "bar");
    }

    #[test]
    fn test_satisfy_ast() {
        let mut g = Grammar::new();
        // one-digit parser via satisfy
        g.define("D", satisfy(|c| c.is_ascii_digit()));

        let ParseResult { node, rest } = g.parse("D", "5xyz").unwrap();
        assert_eq!(
            node.kind,
            NodeKind::Satisfy("character satisfying predicate".to_string())
        );

        assert_eq!(node.value, "5");
        assert_eq!(node.span, Span { start: 0, end: 1 });
        assert_eq!(rest, "xyz");
    }

    #[test]
    fn test_sequence_ast() {
        let mut g = Grammar::new();
        // seq of "a" then "b"
        let parser = seq(vec![lit("a"), lit("b")]);
        g.define("AB", parser);

        let ParseResult { node, rest } = g.parse("AB", "abcd").unwrap();
        // top‐level kind is Sequence
        if let NodeKind::Sequence(children) = &node.kind {
            // two children: "a" and "b"
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].value, "a");
            assert_eq!(children[1].value, "b");
            // span from first start to last end: 0..2
            assert_eq!(node.span, Span { start: 0, end: 2 });
        } else {
            panic!("expected Sequence node");
        }
        assert_eq!(rest, "cd");
    }

    #[test]
    fn test_many_ast() {
        let mut g = Grammar::new();
        // many digits
        g.define("NUMS", many(digit()));

        let ParseResult { node, rest } = g.parse("NUMS", "123xyz").unwrap();
        // NodeKind::Many with three children
        if let NodeKind::Many(children) = &node.kind {
            assert_eq!(children.len(), 3);
            // each child is a Satisfy node matching one digit
            for (i, child) in children.iter().enumerate() {
                let expected = (b'1' + i as u8) as char;
                assert_eq!(child.value, expected.to_string());
                assert_eq!(
                    child.kind,
                    NodeKind::Satisfy("character satisfying predicate".to_string())
                );
            }
            // span covers [0..3]
            assert_eq!(node.span, Span { start: 0, end: 3 });
        } else {
            panic!("expected Many node");
        }
        assert_eq!(rest, "xyz");
    }

    #[test]
    fn test_optional_ast() {
        let mut g = Grammar::new();
        // optional literal "hi"
        g.define("OPT", opt(lit("hi")));

        // when present
        let ParseResult { node: n1, rest: r1 } = g.parse("OPT", "hi!").unwrap();
        if let NodeKind::Optional(inner) = &n1.kind {
            // inner should be a Terminal("hi")
            assert_eq!(inner.value, "hi");
            assert_eq!(inner.kind, NodeKind::Terminal("hi".to_string()));
            // optional node span = inner span
            assert_eq!(n1.span, inner.span);
        } else {
            panic!("expected Optional node");
        }
        assert_eq!(r1, "!");

        // when absent
        let ParseResult { node: n2, rest: r2 } = g.parse("OPT", "bye").unwrap();
        if let NodeKind::Optional(inner) = &n2.kind {
            // absent case: inner should still have no span (start==end)
            assert_eq!(inner.value, "");
            assert_eq!(inner.span, Span { start: 0, end: 0 });
        } else {
            panic!("expected Optional node");
        }
        assert_eq!(r2, "bye");
    }
}

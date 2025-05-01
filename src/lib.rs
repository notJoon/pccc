use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use ast::{Node, NodeKind, Span};
use expr::ParserExpr;

use crate::errors::ParseError;

pub mod ast;
pub mod errors;
pub mod expr;

/// A dynamically-typed value for parsed results.
///
/// `Value` represents the various result types that can be produced by parsers.
///
/// # Examples
///
/// ```
/// use pccc::{Value, ParseResult};
///
/// // String value
/// let string_val = Value::Str("hello".to_string());
///
/// // Character value
/// let char_val = Value::Char('a');
///
/// // List value (composite)
/// let list_val = Value::List(vec![
///     Value::Char('a'),
///     Value::Str("bc".to_string())
/// ]);
///
/// // None value (for optional patterns that didn't match)
/// let none_val = Value::None;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Char(char),
    List(Vec<Value>),
    None,
}

impl Value {
    /// get inner list if it exists.
    pub fn unwrap_list(&self) -> Vec<Value> {
        if let Value::List(lst) = self {
            lst.to_owned()
        } else {
            vec![]
        }
    }

    /// Get the kind of a `Value`.
    pub fn kind(&self) -> &'static str {
        match self {
            Value::Str(_) => "Str",
            Value::Char(_) => "Char",
            Value::List(_) => "List",
            Value::None => "None",
        }
    }

    fn format_list(items: &[Value]) -> String {
        if items.is_empty() {
            return "".to_string();
        }
        if items.len() == 1 {
            return items[0].to_string();
        }

        let mut result = String::new();
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                result.push_str(&item.to_string());
            } else {
                result = item.to_string();
            }
        }
        result
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{}", s),
            Value::Char(c) => write!(f, "{}", c),
            Value::List(items) => write!(f, "{}", Self::format_list(items)),
            Value::None => write!(f, "None"),
        }
    }
}

/// The result of a parse: a Node and the remaining input.
///
/// A `ParseResult` contains both the parsed value and any remaining unparsed input.
/// This enables parsers to be chained together, with each consuming part of the input.
///
/// # Examples
///
/// ```rust
/// use pccc::{Value, ParseResult};
/// use pccc::ast::{Node, NodeKind, Span};
///
/// // A successful parse of a character with remaining input
/// let result = ParseResult {
///     node: Node::new(
///         NodeKind::Terminal("a".to_string()),
///         Span { start: 0, end: 1 },
///         "a".to_string(),
///     ),
///     rest: "bc".to_string(),
/// };
///
/// // The parse consumed 'a' and left 'bc' unparsed
/// assert_eq!(result.rest, "bc");
/// ```
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub node: Node,
    pub rest: String,
}

/// A Parser is a wrapped function that transforms input text according to grammar rules.
///
/// The parser takes a mutable reference to a `Grammar` (for rule lookups and memoization)
/// and an input string, returning either a successful `ParseResult` or a `ParseError`.
///
/// # Example
///
/// ```rust
/// use pccc::{Parser, Grammar, lit, ParseResult, Value};
///
/// // simple "hello" parser
/// let hello_parser: Parser = lit("hello");
/// let mut g = Grammar::new();
///
/// let result = hello_parser(&mut g, "hello world");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, " world"); // Remaining unparsed input
/// ```
pub type Parser = Rc<dyn Fn(&mut Grammar, &str) -> Result<ParseResult, ParseError>>;

#[derive(PartialEq, Eq, Hash)]
struct MemoKey {
    name: String,
    pos: usize,
}

/// Grammar holds named parsing rules and a packrat memo cache.
///
/// A Grammar is the central container for parsing rules and memoization state.
/// It allows:
/// - Defining named rules that can refer to each other (even recursively)
/// - Parsing input using those rules
/// - Automatically memoizing results (packrat parsing) for efficiency
///
/// # Examples
///
/// ```rust
/// use pccc::{Grammar, lit, seq, digit, many};
///
/// let mut g = Grammar::new();
///
/// // rule: Number -> Digit+
/// g.define("Number", seq([digit(), many(digit())]));
///
/// // parse input using the defined rule
/// let result = g.parse("Number", "12345xyz");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "xyz"); // remaining unparsed input
/// ```
pub struct Grammar {
    input: String,
    rules: HashMap<String, Parser>,
    memo: HashMap<MemoKey, Result<ParseResult, ParseError>>,
}

impl Grammar {
    /// Create a new empty Grammar.
    #[allow(
        clippy::new_without_default,
        reason = "stack overflow when using default"
    )]
    pub fn new() -> Self {
        Grammar {
            input: String::new(),
            rules: HashMap::new(),
            memo: HashMap::new(),
        }
    }

    /// Define a rule by name, taking a `Parser`.
    #[inline(always)]
    pub fn define(&mut self, name: &str, p: Parser) {
        self.rules.insert(name.to_string(), p);
    }

    /// Define a grammar rule directly from a [`ParserExpr`].
    pub fn define_expr(&mut self, name: &str, expr: ParserExpr) {
        let parser = expr.compile();
        self.define(name, parser);
    }

    /// Parse the named rule over the given text, resetting memo.
    pub fn parse(&mut self, name: &str, text: &str) -> Result<ParseResult, ParseError> {
        self.input = text.to_string();
        self.memo.clear();
        self.parse_rule(name, text)
    }

    /// parse a rule with packrat memoization.
    fn parse_rule(&mut self, name: &str, input: &str) -> Result<ParseResult, ParseError> {
        // prevent overflow.
        let pos = self.input.len().saturating_sub(input.len());
        let key = MemoKey {
            name: name.to_string(),
            pos,
        };

        // Return cached result if present.
        if let Some(cached) = self.memo.get(&key) {
            return cached.clone();
        }

        // Ensure the rule is defined.
        if !self.rules.contains_key(name) {
            return Err(ParseError::new(format!("rule '{}' not defined", name), pos));
        }

        let parser = self.rules.get(name).unwrap().clone();

        let result = parser(self, input);
        if result.is_ok() {
            self.memo.insert(key, result.clone());
        }

        result
    }

    /// Check that there is no input left after parsing
    pub fn parse_all(&mut self, rule: &str, input: &str) -> Result<Node, ParseError> {
        let result = self.parse(rule, input)?;
        if result.rest.is_empty() {
            Ok(result.node)
        } else {
            Err(ParseError::new(
                "input not fully consumed".to_string(),
                self.input.len() - result.rest.len(),
            ))
        }
    }

    /// Get the size of the memoizationed cache.
    #[inline]
    pub fn memo_size(&self) -> usize {
        self.memo.len()
    }
}

/// Refer to a named rule, enabling recursion.
///
/// When using `rule_ref` in recursive grammars, be mindful of the order of alternatives:
///
/// 1. For **greedy matching** (consuming as much input as possible), put the recursive pattern first.
/// 2. For **non-greedy matching** (consuming as little input as possible), put the recursive pattern last.
///
/// Incorrect ordering can lead to infinite recursion or incomplete parsing.
pub fn rule_ref(name: String) -> Parser {
    Rc::new(move |g: &mut Grammar, input: &str| {
        let pos = g.input.len().saturating_sub(input.len());
        let result = g.parse_rule(&name, input)?;
        let node = Node::new(
            NodeKind::NonTerminal(name.clone()),
            Span {
                start: pos,
                end: pos + (input.len() - result.rest.len()),
            },
            result.node.value.clone(),
        )
        .with_children(vec![result.node]);

        Ok(ParseResult {
            node,
            rest: result.rest,
        })
    })
}

/// Sequence takes multiple parsers and applies them in sequence.
///
/// The `seq` combinator applies two parsers in sequence. It only succeeds
/// if both parsers succeed. The result combines both parsed values into a string.
///
/// # Features
///
/// - Consumes input sequentially through both parsers
/// - Returns a concatenated string of all parsed values
/// - Fails if either parser fails
///
/// # Examples
///
/// ```rust
/// use pccc::{Grammar, seq, lit};
///
/// let parser = seq([lit("hello"), lit(" world")]);
/// let mut g = Grammar::new();
///
/// let result = parser(&mut g, "hello world!");
/// assert!(result.is_ok());
///
/// // Check remaining input
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "!");
///
/// // Check the parsed value (it should be a concatenated string)
/// assert_eq!(parsed.node.value, "hello world");
/// ```
pub fn seq<I>(parsers: I) -> Parser
where
    I: IntoIterator<Item = Parser> + Clone + 'static,
{
    Rc::new(move |g, input| {
        let mut rest = input.to_string();
        let mut children = Vec::new();
        let start_pos = g.input.len().saturating_sub(input.len());

        for parser in parsers.clone().into_iter() {
            let result = parser(g, &rest)?;
            children.push(result.node);
            rest = result.rest;
        }

        let end_pos = g.input.len().saturating_sub(rest.len());
        let node = Node::new(
            NodeKind::Sequence(children.clone()),
            Span {
                start: start_pos,
                end: end_pos,
            },
            children.iter().map(|n| n.value.clone()).collect(),
        )
        .with_children(children);

        Ok(ParseResult { node, rest })
    })
}

/// Alternative: try p1, else p2.
///
/// # Warning
///
/// The order of alternatives is significant, especially in recursive grammars.
/// The first alternative that matches will be chosen, even if a later alternative
/// would consume more input.
///
/// ## Example:
///
/// In a recursive grammar like:
/// ```ignore
/// // A -> "a" | "a" A  (Matches a single "a")
/// g.define("A", alt(
///     lit("a"),
///     seq(lit("a"), rule_ref("A".to_string()))
/// ));
/// ```
///
/// The recursive pattern will never be reached because the first alternative
/// always succeeds on any input starting with "a".
///
/// For greedy matching (i.e., to match as many "a"s as possible), put the
/// recursive pattern first:
/// ```ignore
/// // A -> "a" A | "a"  (Matches multiple "a"s)
/// g.define("A", alt(
///     seq(lit("a"), rule_ref("A".to_string())),
///     lit("a")
/// ));
/// ```
pub fn alt(p1: Parser, p2: Parser) -> Parser {
    Rc::new(move |g, input| {
        let _pos = g.input.len().saturating_sub(input.len());
        match p1(g, input) {
            ok @ Ok(_) => ok,
            Err(err1) => match p2(g, input) {
                ok @ Ok(_) => ok,
                Err(err2) => {
                    let mut combined_err = err1.clone();
                    for expected in err2.expected {
                        combined_err.add_expected(expected);
                    }
                    Err(combined_err)
                }
            },
        }
    })
}

/// Zero or more repetitions of a parser.
///
/// The `many` combinator applies a parser repeatedly until it fails,
/// collecting all the successful results into a list. It always succeeds,
/// even if the parser matches zero times (producing an empty list).
///
/// # Features
///
/// - Matches the parser zero or more times, greedily
/// - Always succeeds, even if zero matches (returns empty list)
/// - Returns all matches as a concatenated string
///
/// # Examples
///
/// ```rust
/// use pccc::{Grammar, many, digit};
///
/// let parser = many(digit());
/// let mut g = Grammar::new();
///
/// // Test with several digits
/// let result = parser(&mut g, "12345abc");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.node.value, "12345");
/// assert_eq!(parsed.rest, "abc");
///
/// // The value should be a string of 5 digits
/// assert_eq!(parsed.node.value.len(), 5);
/// assert_eq!(parsed.node.value.chars().next().unwrap(), '1');
///
/// // no digits. should succeed with empty string
/// let result = parser(&mut g, "abc");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "abc");
/// assert_eq!(parsed.node.value, ""); // Empty string
/// ```
pub fn many(p: Parser) -> Parser {
    Rc::new(move |g, input| {
        let mut children = Vec::new();
        let mut rest = input.to_string();
        let pos = g.input.len().saturating_sub(input.len());

        while let Ok(r) = p(g, &rest) {
            children.push(r.node);
            rest = r.rest;
        }

        let node = Node::new(
            NodeKind::Many(children.clone()),
            Span {
                start: pos,
                end: pos + input.len().saturating_sub(rest.len()),
            },
            children.iter().map(|n| n.value.clone()).collect(),
        )
        .with_children(children);

        Ok(ParseResult { node, rest })
    })
}

/// Optional: try once, else return None.
///
/// The `opt` combinator makes a parser optional. It tries to apply the parser once,
/// and if successful, returns the parser's result. If the parser fails, `opt` still
/// succeeds but returns an empty node without consuming any input.
///
/// # Features
///
/// - Makes a pattern optional
/// - Always succeeds - either with the original parser's result or an empty node
/// - Consumes input only if the inner parser succeeds
///
/// # Examples
///
/// ```
/// use pccc::{Grammar, opt, lit};
///
/// // Create a parser for an optional "hello"
/// let parser = opt(lit("hello"));
///
/// let mut g = Grammar::new();
///
/// // pattern is present
/// let result = parser(&mut g, "hello world");
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, " world");
/// assert_eq!(parsed.node.value, "hello");
///
/// // no pattern
/// let result = parser(&mut g, "world");
/// assert!(result.is_ok()); // still succeeds
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "world"); // no input consumed
/// assert_eq!(parsed.node.value, "");
/// ```
pub fn opt(p: Parser) -> Parser {
    Rc::new(move |g, input| {
        let pos = g.input.len().saturating_sub(input.len());
        match p(g, input) {
            Ok(r) => {
                // If the parser succeeds, create an Optional node containing the result
                // We need to clone the inner node since we'll use it in two places
                let inner_node = r.node.clone();
                let node = Node::new(
                    // Wrap the successful result in an Optional node
                    NodeKind::Optional(Box::new(inner_node)),
                    // Span covers the entire matched portion
                    Span {
                        start: pos,
                        end: pos + (input.len() - r.rest.len()),
                    },
                    // Use the value from the successful parse
                    r.node.value.clone(),
                );
                Ok(ParseResult { node, rest: r.rest })
            }
            Err(_) => {
                // If the parser fails, create an Optional node with an empty inner node
                let node = Node::new(
                    // The inner node is an empty Terminal
                    NodeKind::Optional(Box::new(Node::new(
                        NodeKind::Terminal("".to_string()),
                        Span {
                            start: pos,
                            end: pos,
                        },
                        "".to_string(),
                    ))),
                    // The Optional node itself has the same empty span
                    Span {
                        start: pos,
                        end: pos,
                    },
                    "".to_string(),
                );
                Ok(ParseResult {
                    node,
                    rest: input.to_string(),
                })
            }
        }
    })
}

/// Match a literal string.
///
/// The `lit` combinator creates a parser that matches an exact string literal.
/// It succeeds if the input starts with the specified string, consuming that portion
/// of the input. Otherwise, it fails with an error message.
///
/// # Features
///
/// - Matches exact string literals
/// - Returns the matched string as `Value::Str`
/// - Consumes the matched portion of the input
///
/// # Examples
///
/// ```
/// use pccc::{Grammar, lit, Value};
///
/// // Create a parser for the literal "hello"
/// let parser = lit("hello");
/// let mut g = Grammar::new();
///
/// let result = parser(&mut g, "hello world");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, " world");
///
/// assert_eq!(parsed.node.value, "hello");
///
/// // non-matching input
/// let result = parser(&mut g, "goodbye");
/// assert!(result.is_err());
/// if let Err(err) = result {
///     assert_eq!(err.message, "expected 'hello'");
///     assert_eq!(err.position, 0);
///     assert_eq!(err.expected, vec!["hello"]);
/// }
/// ```
pub fn lit(s: &str) -> Parser {
    let s = s.to_string();
    Rc::new(move |g, input| {
        let pos = g.input.len().saturating_sub(input.len());
        if input.starts_with(&s) {
            let node = Node::new(
                NodeKind::Terminal(s.clone()),
                Span {
                    start: pos,
                    end: pos + s.len(),
                },
                s.clone(),
            );
            Ok(ParseResult {
                node,
                rest: input[s.len()..].to_string(),
            })
        } else {
            let mut err = ParseError::new(format!("expected '{}'", s), pos);
            err.add_expected(s.clone());
            Err(err)
        }
    })
}

/// Match a single character satisfying a predicate function.
///
/// The `satisfy` combinator creates a parser that matches a single character
/// that satisfies the given predicate function. It succeeds if the first character
/// of the input passes the predicate, consuming that character. Otherwise, it fails.
///
/// # Features
///
/// - Matches a single character based on a custom predicate
/// - Returns the matched character as `Value::Char`
/// - Consumes just the matched character
/// - Properly handles multi-byte UTF-8 characters
///
/// # Examples
///
/// ```
/// use pccc::{Grammar, satisfy, Value};
///
/// // Create a parser for uppercase letters
/// let parser = satisfy(|c| c.is_uppercase());
///
/// let mut g = Grammar::new();
///
/// // Test with matching input
/// let result = parser(&mut g, "Hello");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "ello");
///
/// // The value should be the matched character
/// assert_eq!(parsed.node.value, "H");
///
/// // non-matching input
/// let result = parser(&mut g, "hello");
/// assert!(result.is_err());
///
/// // multi-byte UTF-8 character
/// let result = satisfy(|c| true)(&mut g, "🦀 crab");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, " crab");
///
/// assert_eq!(parsed.node.value, "🦀");
///
/// // non-ascii character
/// let result = satisfy(|c| true)(&mut g, "테스트 문자열");
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "스트 문자열");
///
/// assert_eq!(parsed.node.value, "테");
/// ```
///
/// This version is monomorphic and intended for usage with inline closures. It allows
/// the compiler to monomorphize and potentially inline the predicate for performance.
pub fn satisfy<F>(pred: F) -> Parser
where
    F: Fn(char) -> bool + 'static,
{
    Rc::new(move |g, input| satisfy_internal(&pred, input, g))
}

/// Constructs a parser from a dynamically-dispatched character predicate.
///
/// This version is useful when the predicate needs to be passed around as a trait object,
/// such as when building combinator expressions dynamically (e.g. with [`ParserExpr::Satisfy`]).
///
/// # Example
/// ```ignore
/// let is_upper = Rc::new(|c: char| c.is_uppercase());
/// let parser = satisfy_dyn(is_upper);
/// ```
pub fn satisfy_dyn(pred: Rc<dyn Fn(char) -> bool>) -> Parser {
    Rc::new(move |g, input| satisfy_internal(&*pred, input, g))
}

fn satisfy_internal<'a, F>(
    pred: F,
    input: &'a str,
    g: &mut Grammar,
) -> Result<ParseResult, ParseError>
where
    F: Fn(char) -> bool,
{
    let pos = g.input.len().saturating_sub(input.len());
    let mut chars = input.chars();
    if let Some(c) = chars.next() {
        if pred(c) {
            let size = c.len_utf8();
            let node = Node::new(
                NodeKind::Satisfy(format!("character satisfying predicate")),
                Span {
                    start: pos,
                    end: pos + size,
                },
                c.to_string(),
            );
            return Ok(ParseResult {
                node,
                rest: input[size..].to_string(),
            });
        }
        let mut err = ParseError::new(format!("unexpected char '{}'", c), pos);
        err.add_expected("character satisfying predicate".to_string());
        Err(err)
    } else {
        Err(ParseError::new("unexpected end of input".to_string(), pos))
    }
}

/* Common predicates. */

/// Match a single digit character (0-9).
///
/// The `digit` function is a convenience parser that matches any ASCII digit character.
/// It is equivalent to `satisfy(|c| c.is_ascii_digit())`.
pub fn digit() -> Parser {
    satisfy(|c| c.is_ascii_digit())
}

/// Match a single letter (alphabetic character).
///
/// The `letter` function is a convenience parser that matches any Unicode alphabetic character.
/// It is equivalent to `satisfy(|c| c.is_alphabetic())`.
pub fn letter() -> Parser {
    satisfy(|c| c.is_alphabetic())
}

/// Match a single whitespace character.
///
/// The `space` function is a convenience parser that matches any Unicode whitespace character.
/// It is equivalent to `satisfy(|c| c.is_whitespace())`.
pub fn space() -> Parser {
    satisfy(|c| c.is_whitespace())
}

/// building sequence expressions.
#[macro_export]
macro_rules! seq {
    ($($x:expr),+ $(,)?) => {
        ParserExpr::Seq(vec![$($x),+])
    };
}

/// building alternative expressions.
#[macro_export]
macro_rules! alt {
    ($($x:expr),+ $(,)?) => {
        ParserExpr::Alt(vec![$($x),+])
    };
}

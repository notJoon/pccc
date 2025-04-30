use std::collections::HashMap;
use std::rc::Rc;

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
#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Char(char),
    List(Vec<Value>),
    None,
}

impl Value {
    /// Convert a `Value` to a string.
    ///
    /// ```
    /// use pccc::Value;
    ///
    /// let value = Value::Str("hello".to_string());
    /// assert_eq!(value.to_string(), "hello");
    ///
    /// let value = Value::Char('a');
    /// assert_eq!(value.to_string(), "a");
    ///
    /// let value = Value::List(vec![Value::Str("hello".to_string()), Value::Char('a')]);
    /// assert_eq!(value.to_string(), "[hello,a]");
    ///
    /// let value = Value::None;
    /// assert_eq!(value.to_string(), "None");
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::List(items) => format!("[{}]", Self::format_list(items)),
            Value::None => "None".to_string(),
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
        items
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}

/// The result of a parse: a Value and the remaining input.
///
/// A `ParseResult` contains both the parsed value and any remaining unparsed input.
/// This enables parsers to be chained together, with each consuming part of the input.
///
/// # Examples
///
/// ```rust
/// use pccc::{Value, ParseResult};
///
/// // A successful parse of a character with remaining input
/// let result = ParseResult {
///     value: Value::Char('a'),
///     rest: "bc".to_string(),
/// };
///
/// // The parse consumed 'a' and left 'bc' unparsed
/// assert_eq!(result.rest, "bc");
/// ```
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub value: Value,
    pub rest: String,
}

/// A Parser is a wrapped function that transforms input text according to grammar rules.
///
/// The parser takes a mutable reference to a `Grammar` (for rule lookups and memoization)
/// and an input string, returning either a successful `ParseResult` or an error message.
///
/// /// # Example
///
/// ```rust
/// use pccc::{Parser, Grammar, lit, ParseResult, Value};
/// use std::rc::Rc;
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
pub type Parser = Rc<dyn Fn(&mut Grammar, &str) -> Result<ParseResult, String>>;

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
/// g.define("Number", seq(digit(), many(digit())));
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
    memo: HashMap<MemoKey, Result<ParseResult, String>>,
}

impl Grammar {
    /// Create a new empty Grammar.
    pub fn new() -> Self {
        Grammar {
            input: String::new(),
            rules: HashMap::new(),
            memo: HashMap::new(),
        }
    }

    /// Define a rule by name, taking a `Parser`.
    pub fn define(&mut self, name: &str, p: Parser) {
        self.rules.insert(name.to_string(), p);
    }

    /// Parse the named rule over the given text, resetting memo.
    pub fn parse(&mut self, name: &str, text: &str) -> Result<ParseResult, String> {
        self.input = text.to_string();
        self.memo.clear();
        self.parse_rule(name, text)
    }

    /// parse a rule with packrat memoization.
    fn parse_rule(&mut self, name: &str, input: &str) -> Result<ParseResult, String> {
        if input.is_empty() {
            return Err("empty input".to_string());
        }

        let pos = self.input.len() - input.len();
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
            return Err(format!("rule '{}' not defined", name));
        }

        let parser = self.rules.get(name).unwrap().clone();

        let result = parser(self, input);
        if result.is_ok() {
            self.memo.insert(key, result.clone());
        }

        result
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
    Rc::new(move |g: &mut Grammar, input: &str| g.parse_rule(&name, input))
}

/// Sequence applies p1, then p2, returning their values as a list.
///
/// The `seq` combinator applies two parsers in sequence. It only succeeds
/// if both parsers succeed. The result combines both parsed values into a list.
///
/// # Features
///
/// - Consumes input sequentially through both parsers
/// - Returns a `Value::List` containing results from both parsers
/// - Fails if either parser fails
///
/// # Examples
///
/// ```rust
/// use pccc::{Grammar, seq, lit, Value};
///
/// let parser = seq(lit("hello"), lit(" world"));
/// let mut g = Grammar::new();
///
/// let result = parser(&mut g, "hello world!");
/// assert!(result.is_ok());
///
/// // Check remaining input
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "!");
///
/// // Check the parsed value (it should be a list with two items)
/// if let Value::List(items) = parsed.value {
///     assert_eq!(items.len(), 2);
///     // First item should be "hello"
///     if let Value::Str(s) = &items[0] {
///         assert_eq!(s, "hello");
///     }
///     // Second item should be " world"
///     if let Value::Str(s) = &items[1] {
///         assert_eq!(s, " world");
///     }
/// } else {
///     panic!("expected List value");
/// }
/// ```
pub fn seq(p1: Parser, p2: Parser) -> Parser {
    Rc::new(move |g, input| {
        let r1 = p1(g, input)?;
        let r2 = p2(g, &r1.rest)?;
        // each sequence combines exactly two parsers.
        // so we can pre-allocate the list with a capacity of 2.
        let mut list = Vec::with_capacity(2);

        list.push(r1.value);
        list.push(r2.value);

        Ok(ParseResult {
            value: Value::List(list),
            rest: r2.rest,
        })
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
    Rc::new(move |g, input| match p1(g, input) {
        ok @ Ok(_) => ok,
        Err(_) => p2(g, input),
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
/// - Returns all matches as a `Value::List`
///
/// # Examples
///
/// ```rust
/// use pccc::{Grammar, many, digit, Value};
///
/// let parser = many(digit());
/// let mut g = Grammar::new();
///
/// // Test with several digits
/// let result = parser(&mut g, "12345abc");
/// assert!(result.is_ok());
///
///  let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "abc");
///
/// // The value should be a list of 5 digits
/// if let Value::List(items) = parsed.value {
///     assert_eq!(items.len(), 5);
///     // Check first digit is '1'
///     if let Value::Char(c) = items[0] {
///         assert_eq!(c, '1');
///     }
/// }
///
/// // no digits. should succeed with empty list
/// let result = parser(&mut g, "abc");
/// assert!(result.is_ok());
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "abc");
///
/// if let Value::List(items) = parsed.value {
///     assert_eq!(items.len(), 0); // Empty list
/// }
/// ```
pub fn many(p: Parser) -> Parser {
    Rc::new(move |g, input| {
        let mut out = Vec::new();
        let mut rest = input.to_string();
        while let Ok(r) = p(g, &rest) {
            out.push(r.value);
            rest = r.rest;
        }
        Ok(ParseResult {
            value: Value::List(out),
            rest,
        })
    })
}

/// Optional: try once, else return None.
///
/// The `opt` combinator makes a parser optional. It tries to apply the parser once,
/// and if successful, returns the parser's result. If the parser fails, `opt` still
/// succeeds but returns `Value::None` without consuming any input.
///
/// # Features
///
/// - Makes a pattern optional
/// - Always succeeds - either with the original parser's result or `Value::None`
/// - Consumes input only if the inner parser succeeds
///
/// # Examples
///
/// ```
/// use pccc::{Grammar, opt, lit, Value};
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
///
/// // The value should be the parsed string
/// if let Value::Str(s) = parsed.value {
///     assert_eq!(s, "hello");
/// }
///
/// // no pattern
/// let result = parser(&mut g, "world");
/// assert!(result.is_ok()); // still succeeds
///
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "world"); // no input consumed
/// assert!(matches!(parsed.value, Value::None));
/// ```
pub fn opt(p: Parser) -> Parser {
    Rc::new(move |g, input| match p(g, input) {
        Ok(r) => Ok(r),
        Err(_) => Ok(ParseResult {
            value: Value::None,
            rest: input.to_string(),
        }),
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
/// if let Value::Str(s) = parsed.value {
///     assert_eq!(s, "hello");
/// }
///
/// // non-matching input
/// let result = parser(&mut g, "goodbye");
/// assert!(result.is_err());
/// assert_eq!(result.unwrap_err(), "expected 'hello'");
/// ```
pub fn lit(s: &str) -> Parser {
    let s = s.to_string();
    Rc::new(move |_g, input| {
        if input.starts_with(&s) {
            Ok(ParseResult {
                value: Value::Str(s.clone()),
                rest: input[s.len()..].to_string(),
            })
        } else {
            Err(format!("expected '{s}'"))
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
/// if let Value::Char(c) = parsed.value {
///     assert_eq!(c, 'H');
/// }
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
/// if let Value::Char(c) = parsed.value {
///     assert_eq!(c, '🦀');
/// }
///
/// // non-ascii character
/// let result = satisfy(|c| true)(&mut g, "테스트 문자열");
/// assert!(result.is_ok());
/// let parsed = result.unwrap();
/// assert_eq!(parsed.rest, "스트 문자열");
///
/// if let Value::Char(c) = parsed.value {
///     assert_eq!(c, '테');
/// }
/// ```
pub fn satisfy<F>(pred: F) -> Parser
where
    F: Fn(char) -> bool + 'static,
{
    Rc::new(move |_g, input| {
        let mut chars = input.chars();
        if let Some(c) = chars.next() {
            if pred(c) {
                let size = c.len_utf8();
                return Ok(ParseResult {
                    value: Value::Char(c),
                    rest: input[size..].to_string(),
                });
            }
            Err(format!("unexpected char '{c}'"))
        } else {
            Err("unexpected end of input".to_string())
        }
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_helpers() {
        let mut g = Grammar::new();

        assert!(digit()(&mut g, "123").is_ok());
        assert!(digit()(&mut g, "abc").is_err());

        assert!(letter()(&mut g, "abc").is_ok());
        assert!(letter()(&mut g, "123").is_err());

        assert!(space()(&mut g, " abc").is_ok());
        assert!(space()(&mut g, "\tabc").is_ok());
        assert!(space()(&mut g, "\nabc").is_ok());
        assert!(space()(&mut g, "abc").is_err());
    }

    #[test]
    fn test_lit() {
        let mut g = Grammar::new();
        let parser = lit("hello");
        let result = parser(&mut g, "hello world");
        assert!(result.is_ok());

        if let Ok(res) = result {
            assert_eq!(res.rest, " world");
            if let Value::Str(s) = res.value {
                assert_eq!(s, "hello");
            } else {
                panic!("expected Str value");
            }
        }
    }

    #[test]
    fn test_digit() {
        let mut g = Grammar::new();
        let parser = digit();

        let result = parser(&mut g, "5abc");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "abc");
            if let Value::Char(c) = res.value {
                assert_eq!(c, '5');
            } else {
                panic!("expected Char value");
            }
        }

        let result = parser(&mut g, "abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_letter() {
        let mut g = Grammar::new();
        let parser = letter();

        let result = parser(&mut g, "abc");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "bc");
            if let Value::Char(c) = res.value {
                assert_eq!(c, 'a');
            } else {
                panic!("expected Char value");
            }
        }

        let result = parser(&mut g, "123");
        assert!(result.is_err());
    }

    #[test]
    fn test_seq() {
        let mut g = Grammar::new();
        let parser = seq(lit("hello"), lit("world"));

        let result = parser(&mut g, "helloworld!");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "!");
            if let Value::List(items) = res.value {
                assert_eq!(items.len(), 2);
                if let Value::Str(s1) = &items[0] {
                    assert_eq!(s1, "hello");
                } else {
                    panic!("expected Str value for first item");
                }
                if let Value::Str(s2) = &items[1] {
                    assert_eq!(s2, "world");
                } else {
                    panic!("expected Str value for second item");
                }
            } else {
                panic!("expected List value");
            }
        }

        let result = parser(&mut g, "hello123");
        assert!(result.is_err());
    }

    #[test]
    fn test_alt() {
        let mut g = Grammar::new();
        let parser = alt(lit("hello"), lit("world"));

        let result = parser(&mut g, "hello123");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "123");
            if let Value::Str(s) = res.value {
                assert_eq!(s, "hello");
            } else {
                panic!("expected Str value");
            }
        }

        let result = parser(&mut g, "world123");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "123");
            if let Value::Str(s) = res.value {
                assert_eq!(s, "world");
            } else {
                panic!("expected Str value");
            }
        }

        let result = parser(&mut g, "123");
        assert!(result.is_err());
    }

    #[test]
    fn test_many() {
        let mut g = Grammar::new();
        let parser = many(digit());

        let result = parser(&mut g, "12345abc");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "abc");
            if let Value::List(items) = res.value {
                assert_eq!(items.len(), 5);
                for (i, item) in items.iter().enumerate() {
                    if let Value::Char(c) = item {
                        assert_eq!(*c, char::from_digit((i + 1) as u32, 10).unwrap());
                    } else {
                        panic!("expected Char value");
                    }
                }
            } else {
                panic!("expected List value");
            }
        }

        let result = parser(&mut g, "abc");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "abc");
            if let Value::List(items) = res.value {
                assert_eq!(items.len(), 0);
            } else {
                panic!("expected List value");
            }
        }
    }

    #[test]
    fn test_opt() {
        let mut g = Grammar::new();
        let parser = opt(lit("hello"));

        let result = parser(&mut g, "hello world");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, " world");
            if let Value::Str(s) = res.value {
                assert_eq!(s, "hello");
            } else {
                panic!("expected Str value");
            }
        }

        let result = parser(&mut g, "world");
        assert!(result.is_ok()); // 항상 성공함
        if let Ok(res) = result {
            assert_eq!(res.rest, "world"); // 입력이 그대로 남아야 함
            assert!(matches!(res.value, Value::None));
        }
    }

    #[test]
    fn test_rule_ref() {
        let mut g = Grammar::new();

        // A -> "a" | "a" A
        g.define("A", alt(seq(lit("a"), rule_ref("A".to_string())), lit("a")));

        // recursive rule test
        let result = g.parse("A", "aaa");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "");
        }

        let result = g.parse("A", "a");
        assert!(result.is_ok());
        if let Ok(res) = result {
            assert_eq!(res.rest, "", "rest should be empty");
        }

        let result = g.parse("A", "b");
        assert!(result.is_err());
    }

    #[test]
    fn test_grammar_memoization() {
        let mut g = Grammar::new();

        // grammar:
        //
        // Expr -> Term ("+" Term)*
        // Term -> digit

        g.define("Term", digit());
        g.define(
            "Expr",
            seq(
                rule_ref("Term".to_string()),
                many(seq(lit("+"), rule_ref("Term".to_string()))),
            ),
        );

        // test memoization effect with long expression
        let result = g.parse("Expr", "1+2+3+4+5");
        assert!(result.is_ok());
        assert!(g.memo.len() > 0, "memo cache should be non-empty");
    }

    #[test]
    fn test_empty_input() {
        let mut g = Grammar::new();

        g.define("Term", digit());
        g.define(
            "Expr",
            seq(
                rule_ref("Term".to_string()),
                many(seq(lit("+"), rule_ref("Term".to_string()))),
            ),
        );

        let result = g.parse("Expr", "");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "empty input");

        // whitespace input
        let result = g.parse("Expr", " ");
        assert!(result.is_err());

        // undefined rule
        let result = g.parse("Undefined", "");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "empty input");
    }
}

use std::rc::Rc;

use pccc::*;

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
        assert!(is_str(&res.value, "hello"));
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
        assert!(is_char(&res.value, '5'));
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
    let parser = seq(vec![lit("hello"), lit("world")]);

    let result = parser(&mut g, "helloworld!");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "!");
        assert!(is_list(&res.value, 2));
        assert!(is_list_str(&res.value, 0, "hello"));
        assert!(is_list_str(&res.value, 1, "world"));
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
        assert!(is_str(&res.value, "hello"));
    }

    let result = parser(&mut g, "world123");
    assert!(result.is_ok());

    if let Ok(res) = result {
        assert_eq!(res.rest, "123");
        assert!(is_str(&res.value, "world"));
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
                assert!(is_char(item, char::from_digit((i + 1) as u32, 10).unwrap()));
            }
        } else {
            panic!("expected List value");
        }
    }

    let result = parser(&mut g, "abc");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "abc");
        assert!(is_list(&res.value, 0));
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
        assert!(is_str(&res.value, "hello"));
    }

    let result = parser(&mut g, "world");
    assert!(result.is_ok()); // always succeeds
    if let Ok(res) = result {
        assert_eq!(res.rest, "world"); // input should be left unchanged
        assert!(matches!(res.value, Value::None));
    }
}

#[test]
fn test_recursive_rule() {
    let mut g = Grammar::new();

    // A -> "a" | "a" A
    g.define(
        "A",
        alt(seq([lit("a"), rule_ref("A".to_string())]), lit("a")),
    );

    // recursive rule test
    let result = g.parse("A", "aaa");
    assert!(result.is_ok());

    if let Ok(res) = result {
        // ParseResult { value: List([Str("a"), List([Str("a"), Str("a")])]), rest: "" }
        assert!(is_list(&res.value, 2));
        assert!(is_list_str(&res.value, 0, "a"));
        if let Value::List(items) = &res.value {
            if let Value::List(nested) = &items[1] {
                assert_eq!(nested.len(), 2);
                assert!(is_str(&nested[0], "a"));
                assert!(is_str(&nested[1], "a"));
            } else {
                panic!("expected nested List");
            }
        }
        assert_eq!(res.rest, "");
    }

    let result = g.parse("A", "aaab");
    assert!(result.is_ok());
    if let Ok(res) = result {
        // ParseResult { value: List([Str("a"), List([Str("a"), Str("a")])]), rest: "b" }
        assert!(is_list(&res.value, 2));
        assert!(is_list_str(&res.value, 0, "a"));
        if let Value::List(items) = &res.value {
            if let Value::List(nested) = &items[1] {
                assert_eq!(nested.len(), 2);
                assert!(is_str(&nested[0], "a"));
                assert!(is_str(&nested[1], "a"));
            } else {
                panic!("expected nested List");
            }
        }
        assert_eq!(res.rest, "b");
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
        seq([
            rule_ref("Term".to_string()),
            many(seq([lit("+"), rule_ref("Term".to_string())])),
        ]),
    );

    // test memoization effect with long expression
    let result = g.parse("Expr", "1+2+3+4+5");
    assert!(result.is_ok());
    assert!(g.memo_size() > 0, "memo cache should be non-empty");
    println!("memo size: {}", g.memo_size());
}

#[test]
fn test_empty_input() {
    let mut g = Grammar::new();

    let parser = lit("hello");
    let result = parser(&mut g, "");
    assert!(result.is_err());

    let parser = space();
    let result = parser(&mut g, "");
    assert!(result.is_err(), "failed to process empty input");

    let parser = opt(lit("hello"));
    let result = parser(&mut g, "");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "");
        assert!(matches!(res.value, Value::None));
    }
}

#[test]
fn test_invalid_input() {
    let mut g = Grammar::new();
    let parser = lit("hello");

    let result = parser(&mut g, "world");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.message, "expected 'hello'");
        assert_eq!(err.position, 0);
        assert_eq!(err.expected, vec!["hello"]);
    }

    let result = digit()(&mut g, "abc123");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.message, "unexpected char 'a'");
        assert_eq!(err.position, 0);
        assert_eq!(err.expected, vec!["character satisfying predicate"]);
    }
}

#[test]
fn test_boundary_cases() {
    let mut g = Grammar::new();

    // distinguish "a" and "b"
    let parser = alt(lit("a"), lit("b"));
    let result = parser(&mut g, "a");
    assert!(result.is_ok(), "failed to handle boundary");
    if let Ok(res) = result {
        assert_eq!(res.rest, "");
        assert!(is_str(&res.value, "a"));
    }

    let result = parser(&mut g, "b");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "");
        assert!(is_str(&res.value, "b"));
    }

    let result = parser(&mut g, "");
    assert!(result.is_err());
}

#[test]
fn test_character_sets() {
    let mut g = Grammar::new();

    let parser = alt(digit(), letter());

    // number
    let result = parser(&mut g, "1abc");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "abc");
        assert!(is_char(&res.value, '1'));
    }

    // letter
    let result = parser(&mut g, "a123");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "123");
        assert!(is_char(&res.value, 'a'));
    }

    // special character
    let parser = satisfy(|c| c == '@');
    let result = parser(&mut g, "@hello");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "hello");
        assert!(is_char(&res.value, '@'));
    }
}

#[test]
fn test_complex_sequence() {
    let mut g = Grammar::new();

    // (hello\s+)*(world\s+)*
    let parser = seq([
        many(seq([lit("hello"), space()])),
        many(seq([lit("world"), space()])),
    ]);

    let result = parser(&mut g, "hello world world 123");
    assert!(result.is_ok());
    if let Ok(res) = result {
        assert_eq!(res.rest, "123");
    }
}

#[test]
fn test_alt_error_handling() {
    let mut g = Grammar::new();
    let parser = alt(lit("a"), lit("b"));

    let result = parser(&mut g, "c");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.message, "expected 'a'");
        assert_eq!(err.position, 0);
        assert_eq!(err.expected, vec!["a", "b"]);
    }
}

/// Helper function to check if a `Value` is a String (`Str` variant)
fn is_str(value: &Value, expected: &str) -> bool {
    match value {
        Value::Str(s) => s == expected,
        _ => false,
    }
}

/// Helper function to check if a `Value` is a Character (`Char` variant)
fn is_char(value: &Value, expected: char) -> bool {
    match value {
        Value::Char(c) => *c == expected,
        _ => false,
    }
}

/// Helper function to check if a `Value` is a List (`List` variant) of specific length
fn is_list(value: &Value, expected_len: usize) -> bool {
    match value {
        Value::List(items) => items.len() == expected_len,
        _ => false,
    }
}

/// Helper function to check if a List has a specific string at a given index
fn is_list_str(value: &Value, index: usize, expected: &str) -> bool {
    if let Value::List(items) = value {
        if let Some(Value::Str(s)) = items.get(index) {
            return s == expected;
        }
    }
    false
}

/// Helper function to check if a List has a specific character at a given index
fn is_list_char(value: &Value, index: usize, expected: char) -> bool {
    if let Value::List(items) = value {
        if let Some(Value::Char(c)) = items.get(index) {
            return *c == expected;
        }
    }
    false
}

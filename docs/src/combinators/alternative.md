# Alternative Combinator

The alternative combinator (`alt`) is a fundamental building block in PCCC that allows you to express choice between two parsers. It tries the first parser, and if that fails, it tries the second parser.

## Basic Concept

The alternative combinator takes two parsers and returns a new parser that:

1. Applies the first parser to the input
2. If the first parser fails, applies the second parser to the original input
3. Returns the result of whichever parser succeeded first

```
                  +--------------+
                  | First Parser |----> Result (if successful)
                  +--------------+
                       |
                       | (on failure)
                       v
Input -------------> Switch
                       |
                       v
                  +--------------+
                  | Second Parser |----> Result (if successful)
                  +--------------+
                       |
                       | (on failure)
                       v
                  Error Message
```

## State Transitions

The state transitions for the alternative combinator can be visualized as:

```
Initial State: (Input: "abcdef", Parsers: [P1, P2])

Attempt 1: Apply P1 to "abcdef"
   +------+    P1     
   |"abcdef"|-------X  (P1 fails)
   +------+            

Attempt 2: Apply P2 to "abcdef" (original input)
   +------+    P2     +------+
   |"abcdef"|--------->|"def"  |  (P2 succeeds, consuming "abc", result: "abc")
   +------+            +------+

Final State: (Result: "abc", Remaining: "def")
```

## Warning: Order Matters!

⚠️ **Important**: The order of alternatives is significant, especially in recursive grammars.

The first alternative that matches will be chosen, even if a later alternative would consume more input. This can lead to unexpected behavior in recursive grammars:

```rust
// A -> "a" | "a" A  (Matches a single "a")
g.define("A", alt(
    lit("a"),
    seq(lit("a"), rule_ref("A".to_string()))
));
```

In the example above, the recursive pattern will never be reached because the first alternative always succeeds on any input starting with "a".

For greedy matching (i.e., to match as many "a"s as possible), put the recursive pattern first:

```rust
// A -> "a" A | "a"  (Matches multiple "a"s)
g.define("A", alt(
    seq(lit("a"), rule_ref("A".to_string())),
    lit("a")
));
```

## Example Usage

Here's how to use the alternative combinator for different parsing scenarios:

```rust
use pccc::{Grammar, alt, lit, digit, letter};

let mut g = Grammar::new();

// Create a parser for either a digit or a letter
let digit_or_letter = alt(digit(), letter());

// Parse a digit
let result = digit_or_letter(&mut g, "5abc");
assert!(result.is_ok());
let parsed = result.unwrap();
assert_eq!(parsed.rest, "abc");
if let Value::Char(c) = parsed.value {
    assert_eq!(c, '5');
}

// Parse a letter
let result = digit_or_letter(&mut g, "abc");
assert!(result.is_ok());
let parsed = result.unwrap();
assert_eq!(parsed.rest, "bc");
if let Value::Char(c) = parsed.value {
    assert_eq!(c, 'a');
}

// Parsing fails if neither alternative matches
let result = digit_or_letter(&mut g, "!abc");
assert!(result.is_err());
```

## Common Patterns with Alternative

### Multiple Keyword Choices

Alternatives are often used for selecting between keywords:

```rust
// Parser for boolean literals: "true" or "false"
let boolean = alt(lit("true"), lit("false"));
```

### Complex Patterns

For more than two alternatives, you can nest them:

```rust
// Parser for control keywords: "if", "else", "while", "for"
let control_keyword = alt(
    lit("if"),
    alt(
        lit("else"),
        alt(
            lit("while"),
            lit("for")
        )
    )
);
```

### Type Alternatives

In languages with multiple expression types:

```rust
// Different types of expressions
g.define("expr", alt(
    rule_ref("number".to_string()),
    alt(
        rule_ref("string_literal".to_string()),
        rule_ref("identifier".to_string())
    )
));
```

## Creating Multi-way Alternatives

PCCC provides only a binary `alt` combinator, but you can easily build multi-way alternatives by nesting:

```
                +------+
                | P1   |----> Result
                +------+
                   |
                   | (on failure)
                   v
Input---------> +------+
                | alt  |
                +------+
                   |
                   v
                +------+  +------+
                | alt  |->| P2   |----> Result
                +------+  +------+
                   |
                   v
                +------+  +------+
                | alt  |->| P3   |----> Result
                +------+  +------+
                   |
                   | (and so on...)
```

For convenience, you might want to define a helper function for multi-way alternatives:

```rust
fn choose(parsers: Vec<Parser>) -> Parser {
    if parsers.is_empty() {
        panic!("Cannot create alternative with no parsers");
    }
    
    let mut result = parsers[0].clone();
    for i in 1..parsers.len() {
        result = alt(result, parsers[i].clone());
    }
    result
}
```

## Best Practices

1. **Consider the order**: Put more specific patterns before more general ones
2. **Be careful with recursion**: In recursive rules, consider whether to use greedy or non-greedy matching
3. **Provide meaningful errors**: Remember that the error from the last alternative is what the user will see
4. **Test all branches**: Ensure each alternative works correctly in isolation and in combination

## When to Use Alternative

Use the alternative combinator when:

- You need to handle different possible inputs at the same position
- You're defining grammar rules with multiple production options
- You want to provide fallback parsing strategies

The alternative combinator is essential for expressing the branching nature of most grammars and is frequently used alongside other combinators like `seq` and `many`.

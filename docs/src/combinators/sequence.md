# Sequence Combinator

The sequence combinator (`seq`) is one of the most fundamental combinators in PCCC. It allows you to specify that two parsers should be applied in sequence, one after the other.

## Basic Concept

The sequence combinator takes two parsers and returns a new parser that:

1. Applies the first parser to the input
2. If the first parser succeeds, applies the second parser to the remaining input
3. If both parsers succeed, returns a list containing both results

```
            +--------------+        +--------------+
Input ----> | First Parser | -----> | Second Parser | ----> Result List
            +--------------+        +--------------+
                   |                       |
                   | (on failure)          | (on failure)
                   v                       v
              Error Message           Error Message
```

## State Transitions

The state transitions for the sequence combinator can be visualized as:

```
Initial State: (Input: "abcdef", Parsers: [P1, P2])

Step 1: Apply P1 to "abcdef"
   +--------+    P1    +-------+
   |"abcdef"|--------->|"cdef" |  (P1 consumed "ab", result: "ab")
   +--------+          +-------+

Step 2: Apply P2 to "cdef"
   +-------+    P2    +-------+
   |"cdef" |--------->|"ef"   |  (P2 consumed "cd", result: "cd")
   +-------+          +-------+

Final State: (Result: ["ab", "cd"], Remaining: "ef")
```

## Example Usage

Here's how to use the sequence combinator to parse simple patterns:

```rust
use pccc::{Grammar, seq, lit, digit, letter};

let mut g = Grammar::new();

// Create a parser for "hello" followed by a digit
let hello_digit = seq(lit("hello"), digit());

// Parse a matching input
let result = hello_digit(&mut g, "hello5world");
assert!(result.is_ok());

let parsed = result.unwrap();
assert_eq!(parsed.rest, "world");  // Remaining input

// The result is a list with two values: the string "hello" and the character '5'
if let Value::List(items) = parsed.value {
    assert_eq!(items.len(), 2);
    
    // First item is the string "hello"
    if let Value::Str(s) = &items[0] {
        assert_eq!(s, "hello");
    }
    
    // Second item is the character '5'
    if let Value::Char(c) = items[1] {
        assert_eq!(c, '5');
    }
}

// Parsing fails if the sequence doesn't match
let result = hello_digit(&mut g, "hello world");
assert!(result.is_err());  // Fails because 'w' is not a digit
```

## Common Patterns with Sequence

### Building Compound Structures

Sequences are often used to build compound structures:

```rust
// Parser for a simple assignment: "let x = 42;"
let assignment = seq(
    lit("let"),
    seq(
        satisfy(|c| c.is_alphabetic()),
        seq(
            lit("="),
            seq(
                many(digit()),
                lit(";")
            )
        )
    )
);
```

### Parser for Delimited Content

Sequences can define content between delimiters:

```rust
// Parser for content in parentheses: (content)
let parens_content = seq(
    lit("("),
    seq(
        many(satisfy(|c| c != ')')),  // Any characters except closing paren
        lit(")")
    )
);
```

## Nesting Sequences

When you need to sequence more than two parsers, you'll need to nest sequences:

```
             +-----+
             | P1  |
             +-----+
                |
                v
Input ----> +-----+  +-----+
            | seq |->| P2  |
            +-----+  +-----+
                        |
                        v
                     +-----+  +-----+
                     | seq |->| P3  |---> Result
                     +-----+  +-----+
```

For example, to parse "a", "b", and "c" in sequence:

```rust
let abc = seq(lit("a"), seq(lit("b"), lit("c")));
```

## Best Practices

1. **Keep sequences manageable**: Use helper functions to break down complex sequences
2. **Handle errors gracefully**: Remember that sequences fail if any step fails
3. **Consider extraction**: If you only need one part of a sequence, use specific extractors afterward
4. **Right-associative sequences**: When building long sequences, build them right-associatively like the examples above

## When to Use Sequence

Use the sequence combinator when:

- You need to match patterns in a specific order
- Multiple elements form a single logical unit
- You're building structured data with predictable patterns

The sequence combinator is particularly powerful when combined with other combinators like `alt` and `many` to create complex grammars.

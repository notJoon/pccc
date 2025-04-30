# Introduction to PCCC

Welcome to the documentation for PCCC (Parser Combinator in Rust), a lightweight and efficient parser combinator library written in Rust.

## What is PCCC?

PCCC is a library that allows you to build complex parsers by combining simple ones. It follows the parser combinator pattern, a functional approach to parsing where small, reusable parsers are composed to create more powerful ones.

Key features of PCCC include:

- **Composable** - Build complex parsers from simple building blocks
- **Readable** - Grammar definitions closely resemble their formal descriptions
- **Efficient** - Uses packrat parsing with memoization for linear time complexity
- **Zero dependencies** - Only relies on Rust's standard library

## Parser Combinators: A Brief Overview

Parser combinators are functions that take parsers as input and return new parsers as output. This approach allows for:

- **Modularity**: Define small parsers and compose them into larger ones
- **Reusability**: Combine parsers in different ways for different grammars
- **Maintainability**: Changes to grammar rules are localized and intuitive

## How PCCC Works

At a high level, PCCC operates by:

```plain
                   +---------------+
Input String ----> |               | ----> ParseResult
                   |  Parser       |        - Parsed Value
                   |               |        - Remaining Input
                   +---------------+
                          |
                          | (on failure)
                          v
                     Error Message
```

A parser in PCCC is a function that:

1. Takes an input string and a grammar context
2. Consumes some prefix of that input (or none)
3. Returns either:
   - A success result with a value and the remaining unparsed input
   - An error message describing why parsing failed

## When Should Use PCCC?

PCCC is ideal for:

- **Compiler writers** creating parsers for programming languages
- **Data processing** applications that need to parse custom formats
- **Text processing** tools requiring complex text manipulation

## Example: A Simple Calculator

Here's a taste of what parsing with PCCC looks like:

```rust
use pccc::{Grammar, digit, many, lit, seq, alt, rule_ref};

// Create a grammar for basic arithmetic
let mut g = Grammar::new();

// Define our rules
g.define("number", seq(digit(), many(digit())));

g.define("factor", alt(
    rule_ref("number".to_string()),
    seq(
        lit("("),
        seq(
            rule_ref("expr".to_string()),
            lit(")")
        )
    )
));

g.define("term", seq(
    rule_ref("factor".to_string()),
    many(seq(
        alt(lit("*"), lit("/")),
        rule_ref("factor".to_string())
    ))
));

g.define("expr", seq(
    rule_ref("term".to_string()),
    many(seq(
        alt(lit("+"), lit("-")),
        rule_ref("term".to_string())
    ))
));

// Parse an expression
let result = g.parse("expr", "2+3*4");
assert!(result.is_ok());
```

In the following chapters, we'll explore how to use PCCC to build parsers for a variety of applications, from simple calculators to complex domain-specific languages.

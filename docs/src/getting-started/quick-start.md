# Quick Start

This guide will help you get started with PCCC quickly. We'll build a simple parser for a calculator that can handle basic arithmetic expressions.

## Adding PCCC to Your Project

First, add PCCC to your Cargo.toml file:

```toml
[dependencies]
pccc = "0.1.0"
```

## A Simple Calculator Parser

Let's create a parser for simple arithmetic expressions like `2+3*4` or `(1+2)*3`.

```rust
use pccc::{Grammar, lit, seq, alt, many, digit, rule_ref};

fn main() {
    // Create a new grammar
    let mut g = Grammar::new();

    // Define rules for our arithmetic grammar

    // Number → Digit+
    g.define("Number", seq(digit(), many(digit())));

    // Factor → Number | "(" Expr ")"
    g.define("Factor", alt(
        rule_ref("Number".to_string()),
        seq(
            lit("("),
            seq(
                rule_ref("Expr".to_string()),
                lit(")")
            )
        )
    ));

    // Term → Factor (("*" | "/") Factor)*
    g.define("Term", seq(
        rule_ref("Factor".to_string()),
        many(seq(
            alt(lit("*"), lit("/")),
            rule_ref("Factor".to_string())
        ))
    ));

    // Expr → Term (("+" | "-") Term)*
    g.define("Expr", seq(
        rule_ref("Term".to_string()),
        many(seq(
            alt(lit("+"), lit("-")),
            rule_ref("Term".to_string())
        ))
    ));

    // Parse an expression
    let result = g.parse("Expr", "2+3*4");
    match result {
        Ok(parsed) => println!("Successfully parsed: {:?}", parsed.value),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

## Step-by-Step Breakdown

Let's break down how this parser works:

### 1. Creating a Grammar

```rust
let mut g = Grammar::new();
```

This creates a new empty grammar where we'll define our parsing rules.

### 2. Defining Rules

We define rules from the bottom up:

```
Number → Digit+
Factor → Number | "(" Expr ")"
Term → Factor (("*" | "/") Factor)*
Expr → Term (("+" | "-") Term)*
```

This grammar captures the standard precedence of arithmetic operations.

### 3. Parsing Input

```rust
let result = g.parse("Expr", "2+3*4");
```

We parse the input string "2+3*4" starting with the "Expr" rule.

## Visualizing the Parse

When parsing "2+3*4", the steps look like:

```
Input: "2+3*4"

1. Start with Expr rule
   |
   +---> Parse Term: "2"
   |      |
   |      +---> Parse Factor: "2"
   |      |      |
   |      |      +---> Parse Number: "2"
   |      |
   |      +---> No * or / operators, Term complete
   |
   +---> Parse + operator
   |
   +---> Parse Term: "3*4"
          |
          +---> Parse Factor: "3"
          |      |
          |      +---> Parse Number: "3"
          |
          +---> Parse * operator
          |
          +---> Parse Factor: "4"
                 |
                 +---> Parse Number: "4"

Final parse result represents: (2) + (3 * 4)
```

## Working with the Parsed Result

The parsed result is a nested structure of `Value` objects:

```rust
match result {
    Ok(parsed) => {
        // parsed.value contains the parse tree
        // parsed.rest contains any unparsed input (should be empty for complete parse)
        println!("Successfully parsed with remaining: {:?}", parsed.rest);
    },
    Err(e) => println!("Parse error: {}", e),
}
```

## Building More Complex Parsers

From this foundation, you can extend your parser to handle:

- Variables and assignments
- Function calls
- More operators
- Whitespace handling
- Error reporting

## Next Steps

Now that you have a basic understanding of PCCC, you might want to:

1. Learn about [Grammar and Rules](../core-concepts/grammar-and-rules.md) in depth
2. Explore more [Parser Combinators](../combinators/sequence.md)
3. Study [Recursive Grammars](../advanced-features/recursive-grammars.md) for complex structures
4. Check out the [API Reference](../api/grammar.md) for detailed documentation

You can find the complete source code for this example and more advanced examples in the PCCC repository.

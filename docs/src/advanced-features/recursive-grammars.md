# Recursive Grammars

Recursive grammars are powerful constructs that allow you to define patterns that refer to themselves. This capability is essential for parsing hierarchical structures like nested expressions, complex data formats, and programming languages.

## What is Grammar Recursion?

A recursive grammar contains rules that refer to themselves, either directly or indirectly. For example:

- **Direct recursion**: Rule A contains a reference to rule A
- **Indirect recursion**: Rule A refers to rule B, which refers back to rule A

Recursion allows us to express infinitely nested structures with finite grammar rules.

## Types of Recursion

### Right Recursion

Right recursion is when the recursive reference appears on the right side of a sequence:

```
A → α A | β
```

Where α and β are other patterns.

In PCCC, this can be implemented as:

```rust
g.define("A", alt(
    seq(alpha, rule_ref("A".to_string())),
    beta
));
```

This pattern is well-suited for parser combinators and is used for parsing sequences of elements.

State transitions in right recursion (parsing "aaaab" with A → "a" A | "b"):

```
Initial State: Input = "aaaab", Rule = A

Step 1: Try first alternative: "a" A
   +"aaaab"+    lit("a")   +"aaab"+   rule_ref("A")
   +-------+--------------->+-----+------------------>
   
Step 2: Recursive call with Input = "aaab", Rule = A
   +"aaab"+    lit("a")   +"aab"+   rule_ref("A")
   +------+--------------->+-----+------------------>
   
Step 3: Recursive call with Input = "aab", Rule = A
   +"aab"+    lit("a")   +"ab"+   rule_ref("A")
   +-----+--------------->+----+------------------>
   
Step 4: Recursive call with Input = "ab", Rule = A
   +"ab"+    lit("a")   +"b"+   rule_ref("A")
   +----+--------------->+---+------------------>
   
Step 5: Recursive call with Input = "b", Rule = A
   +"b"+    Try first alt  +fail+   Try second alt  +"+   lit("b")
   +---+------------------>+----+------------------>+---+----------> Success

Final State: Successful parse of "aaaab"
```

### Left Recursion

Left recursion is when the recursive reference appears on the left side of a sequence:

```
A → A α | β
```

**Important**: Direct left recursion is problematic for recursive descent parsers including PCCC, as it can lead to infinite loops. It requires special handling techniques.

### Mutual Recursion

Mutual recursion involves multiple rules that reference each other:

```
A → B | α
B → A | β
```

In PCCC, this is implemented as:

```rust
g.define("A", alt(rule_ref("B".to_string()), alpha));
g.define("B", alt(rule_ref("A".to_string()), beta));
```

Mutual recursion is useful for expressing complex language constructs.

## Implementation with PCCC

PCCC supports recursive grammars through the `rule_ref` function, which creates a reference to a named rule:

### Basic Example: Matching Parentheses

Here's how to match balanced parentheses with recursion:

```rust
let mut g = Grammar::new();

// Parens → "(" Parens ")" | ε
g.define("Parens", alt(
    seq(
        lit("("),
        seq(
            rule_ref("Parens".to_string()),
            lit(")")
        )
    ),
    // Empty alternative (epsilon)
    Rc::new(|_g, input| Ok(ParseResult {
        value: Value::None,
        rest: input.to_string(),
    }))
));

// Test parsing
assert!(g.parse("Parens", "").is_ok());  // Empty string
assert!(g.parse("Parens", "()").is_ok());  // Simple pair
assert!(g.parse("Parens", "(())").is_ok());  // Nested pairs
assert!(g.parse("Parens", "((()()))").is_ok());  // Complex nesting
assert!(g.parse("Parens", "(()").is_err());  // Unbalanced - error
```

### Expression Grammar Example

Here's a more complex example for parsing arithmetic expressions:

```rust
let mut g = Grammar::new();

// Define basic number rule
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

// Test parsing expressions
assert!(g.parse("Expr", "1+2*3").is_ok());
assert!(g.parse("Expr", "(1+2)*3").is_ok());
assert!(g.parse("Expr", "1+(2*3)").is_ok());
```

## Controlling Recursion: Greedy vs. Non-Greedy

The order of alternatives in recursive rules determines whether the parsing is greedy (matches as much as possible) or non-greedy (matches as little as possible).

### Greedy Matching (Recursive First)

```rust
// A -> "a" A | "a"
g.define("A", alt(
    seq(lit("a"), rule_ref("A".to_string())),
    lit("a")
));
```

This matches as many "a"s as possible in the input string.

### Non-Greedy Matching (Recursive Last)

```rust
// A -> "a" | "a" A
g.define("A", alt(
    lit("a"),
    seq(lit("a"), rule_ref("A".to_string()))
));
```

This matches only a single "a" in the input string because the first alternative always succeeds.

## Memoization and Recursion

PCCC uses packrat parsing with memoization to handle recursion efficiently:

```rust
fn parse_rule(&mut self, name: &str, input: &str) -> Result<ParseResult, String> {
    let pos = self.input.len() - input.len();
    let key = MemoKey {
        name: name.to_string(),
        pos,
    };

    // Return cached result if present
    if let Some(cached) = self.memo.get(&key) {
        return cached.clone();
    }

    // ... parsing logic ...

    // Cache the result
    if result.is_ok() {
        self.memo.insert(key, result.clone());
    }

    result
}
```

This memoization prevents exponential explosion in parsing time for complex recursive grammars.

## Common Pitfalls and Solutions

### Infinite Recursion

**Problem**: Rules that always recurse without consuming input.
**Solution**: Ensure recursive paths consume at least one input character.

```
// Problematic - can recurse infinitely
Bad → Bad | "a"

// Better - consumes input before recursion
Good → "a" Good | "a"
```

### Left Recursion

**Problem**: Direct left recursion causes infinite loops.
**Solution**: Rewrite as right recursion, or use a more sophisticated algorithm.

```
// Problematic - direct left recursion
Expr → Expr "+" Term | Term

// Rewritten without direct left recursion
Expr → Term ExprTail
ExprTail → "+" Term ExprTail | ε
```

### Inefficient Backtracking

**Problem**: Excessive backtracking in complex grammars.
**Solution**: Use the memoization provided by PCCC's packrat parsing.

## Best Practices

1. **Test thoroughly**: Recursive grammars can have subtle behaviors - test with various inputs
2. **Consider termination**: Ensure recursive rules have proper base cases
3. **Order alternatives carefully**: Place alternatives in order of specificity or greediness as needed
4. **Use memoization**: Let PCCC's packrat parsing handle the efficiency concerns
5. **Break down complex rules**: Split complex recursive structures into multiple simpler rules

Recursive grammars are powerful, but they require careful design. With PCCC's support for rule references and efficient memoization, you can create parsers for highly complex and nested structures.

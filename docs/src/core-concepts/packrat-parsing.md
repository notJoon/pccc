# Packrat Parsing

Packrat parsing is a technique that significantly improves the efficiency of parser combinators by memoizing (caching) intermediate parsing results. PCCC uses packrat parsing internally to achieve linear-time parsing performance even for complex grammars.

## The Problem: Inefficient Backtracking

Without memoization, recursive descent parsers (including traditional parser combinators) can have exponential time complexity due to repeated backtracking. Consider this grammar:

```
S → A | B
A → "a" A | "a"
B → "a" B | "a" "b"
```

When parsing the string "aaaaab", a naive parser might:

1. Try path A many times, backtracking at each step
2. Eventually try path B, which ultimately succeeds
3. Re-parse the same input positions many times

This leads to an exponential explosion in parsing time for certain inputs.

## The Solution: Memoization

Packrat parsing solves this by storing the result of parsing each rule at each input position:

```
                  +----------------+
Parse Request --->| Check if result|---> Return cached result
(rule, position)  | is in cache    |     (if found)
                  +----------------+
                          |
                          | (if not found)
                          v
                  +----------------+
                  | Parse normally |
                  +----------------+
                          |
                          v
                  +----------------+
                  | Store result   |
                  | in cache       |
                  +----------------+
                          |
                          v
                  +----------------+
                  | Return result  |
                  +----------------+
```

## How PCCC Implements Packrat Parsing

PCCC implements packrat parsing by:

1. **Maintaining a memo table**: Each Grammar instance has a memo table that maps (rule name, position) pairs to parsing results
2. **Checking the memo before parsing**: Before applying a rule, PCCC checks if the result is already cached
3. **Caching successful results**: After successfully parsing, PCCC caches the result for future use

## The Memoization Key

The key for the memo table consists of:

- **Rule name**: The name of the parsing rule being applied
- **Position**: The position in the input string where parsing started

This ensures that each unique (rule, position) pair is memoized only once.

## Visual Example of Memoization

Let's visualize how packrat parsing works with a simple grammar:

```
Expr → Term ("+" Term)*
Term → "a" | "b"
```

Parsing the input "a+b+a":

```
Initial state: Input = "a+b+a", Memo = {}

Step 1: Parse Expr at position 0
   - Parse Term at position 0
     - Try "a" at position 0: Success, consuming "a"
     - Add to memo: {(Term, 0) -> Success("a", rest: "+b+a")}
   - Parse ("+" Term)* at position 1 (input: "+b+a")
     - Match "+": Success
     - Parse Term at position 2 (input: "b+a")
       - Try "b" at position 2: Success, consuming "b"
       - Add to memo: {(Term, 2) -> Success("b", rest: "+a")}
     - Continue with ("+" Term)* at position 3 (input: "+a")
       - Match "+": Success
       - Parse Term at position 4 (input: "a")
         - Try "a" at position 4: Success, consuming "a"
         - Add to memo: {(Term, 4) -> Success("a", rest: "")}
     - Continue with ("+" Term)* at position 5 (input: "")
       - End of input, stop repeating
   - Add to memo: {(Expr, 0) -> Success(["a", "+", "b", "+", "a"], rest: "")}

Final memo table:
{
  (Term, 0) -> Success("a", rest: "+b+a"),
  (Term, 2) -> Success("b", rest: "+a"),
  (Term, 4) -> Success("a", rest: ""),
  (Expr, 0) -> Success(["a", "+", "b", "+", "a"], rest: "")
}
```

If we later need to parse Term at position 2 again (within a different rule), we can retrieve the result directly from the memo table instead of re-parsing.

## Benefits of Packrat Parsing

1. **Linear Time Complexity**: Guaranteed O(n) time complexity, where n is the length of the input
2. **Avoids Redundant Work**: Each rule is parsed at most once at each input position
3. **Enables Left Recursion Handling**: Advanced packrat implementations can handle left recursion
4. **Preserves Backtracking Capability**: Still allows backtracking without the exponential cost

## Performance Considerations

While packrat parsing provides excellent asymptotic performance, it has some practical considerations:

1. **Memory Usage**: The memo table can consume significant memory for long inputs
2. **Initialization Overhead**: There's some overhead in setting up the memo table
3. **Cloning Results**: PCCC clones results when retrieving from the memo table, which has a small cost

## When Packrat Parsing Matters Most

Packrat parsing provides the biggest benefits in these scenarios:

1. **Highly Ambiguous Grammars**: Where multiple interpretations are possible
2. **Complex Recursive Structures**: Deeply nested grammatical structures
3. **Rules Applied Multiple Times**: When the same rule is applied at the same position in different contexts
4. **Backtracking-Heavy Parsing**: When many alternatives need to be tried

## Implementation Details

PCCC resets the memo table for each new parse operation:

```rust
pub fn parse(&mut self, name: &str, text: &str) -> Result<ParseResult, String> {
    self.input = text.to_string();
    self.memo.clear();  // Clear memo table for each new parse
    self.parse_rule(name, text)
}
```

This ensures that the memo table doesn't grow indefinitely across multiple parse operations.

## Memo Table Growth Exaple

As parsing progresses through the input, the memo table fills with results:

```
Input:  a b c d e f
        ↓ ↓ ↓ ↓ ↓ ↓
Pos:    0 1 2 3 4 5

Memo table growth:
┌─────────────┬────────────────────────┐
│ (Rule, Pos) │ Result                 │
├─────────────┼────────────────────────┤
│ (A, 0)      │ Success(val1, rest: 1) │ 
├─────────────┼────────────────────────┤
│ (B, 0)      │ Error                  │
├─────────────┼────────────────────────┤
│ (C, 0)      │ Error                  │
├─────────────┼────────────────────────┤
│ (A, 1)      │ Success(val2, rest: 2) │
├─────────────┼────────────────────────┤
│ (B, 1)      │ Success(val3, rest: 3) │
├─────────────┼────────────────────────┤
│     ...     │ ...                    │
└─────────────┴────────────────────────┘
```

## Conclusion

Packrat parsing is a crucial optimization in PCCC that enables efficient parsing of complex grammars. By memoizing intermediate results, PCCC avoids redundant work and achieves linear-time performance, making it suitable for parsing large inputs with sophisticated grammatical structures.

The packrat mechanism in PCCC is largely transparent to users - you define your grammar rules normally, and PCCC handles the memoization internally. This gives you the best of both worlds: the clean, compositional style of parser combinators with the performance benefits of packrat parsing.
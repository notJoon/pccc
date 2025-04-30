use pccc::*;

#[test]
fn test_pseudo_json_grammar() {
    let mut g = Grammar::new();

    // JSON format
    // Value -> Object | Array | String | Number
    // Object -> "{" (String ":" Value)? ("," String ":" Value)* "}"
    // Array -> "[" Value? ("," Value)* "]"
    // String -> "\"" Char* "\""
    // Number -> Digit+

    g.define("Number", seq(digit(), many(digit())));

    // String: double quotes with any characters (simple version)
    g.define(
        "String",
        seq(lit("\""), seq(many(satisfy(|c| c != '"')), lit("\""))),
    );

    // Array: double quotes with any characters (simple version)
    g.define(
        "Array",
        seq(
            lit("["),
            seq(
                opt(rule_ref("Value".to_string())),
                seq(many(seq(lit(","), rule_ref("Value".to_string()))), lit("]")),
            ),
        ),
    );

    // Object: curly braces with key-value pairs
    g.define(
        "Object",
        seq(
            lit("{"),
            seq(
                opt(seq(
                    rule_ref("String".to_string()),
                    seq(lit(":"), rule_ref("Value".to_string())),
                )),
                seq(
                    many(seq(
                        lit(","),
                        seq(
                            rule_ref("String".to_string()),
                            seq(lit(":"), rule_ref("Value".to_string())),
                        ),
                    )),
                    lit("}"),
                ),
            ),
        ),
    );

    // Value: one of Object, Array, String, Number
    g.define(
        "Value",
        alt(
            rule_ref("Object".to_string()),
            alt(
                rule_ref("Array".to_string()),
                alt(
                    rule_ref("String".to_string()),
                    rule_ref("Number".to_string()),
                ),
            ),
        ),
    );

    let result = g.parse("Value", "{\"name\":\"value\"}");
    assert!(result.is_ok());

    let arr_result = g.parse("Value", "[1,2,3]");
    assert!(arr_result.is_ok());

    let nested_result = g.parse("Value", "{\"arr\":[1,{\"key\":\"val\"}]}");
    assert!(nested_result.is_ok());
}

#[test]
fn test_calculator_grammar() {
    let mut g = Grammar::new();

    // Simple calculator grammar
    //  Expr ::= Term (("+" | "-") Term)*
    //  Term ::= Factor (("*" | "/") Factor)*
    //  Factor ::= Number | "(" Expr ")"
    //  Number ::= Digit+

    // Define Number
    g.define("Number", seq(digit(), many(digit())));

    // Define Factor (parenthesized expression or number)
    g.define(
        "Factor",
        alt(
            rule_ref("Number".to_string()),
            seq(lit("("), seq(rule_ref("Expr".to_string()), lit(")"))),
        ),
    );

    // Define Term (multiplication/division of terms)
    g.define(
        "Term",
        seq(
            rule_ref("Factor".to_string()),
            many(seq(alt(lit("*"), lit("/")), rule_ref("Factor".to_string()))),
        ),
    );

    // Define Expr (addition/subtraction of terms)
    g.define(
        "Expr",
        seq(
            rule_ref("Term".to_string()),
            many(seq(alt(lit("+"), lit("-")), rule_ref("Term".to_string()))),
        ),
    );

    // Simple expression test
    let result = g.parse("Expr", "1+2*3");
    assert!(result.is_ok());

    // Expression with parentheses test
    let result = g.parse("Expr", "(1+2)*3");
    assert!(result.is_ok());

    // Complex expression test
    let result = g.parse("Expr", "1+(2*3-4)/5");
    assert!(result.is_ok());
}

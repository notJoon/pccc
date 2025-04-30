use pccc::*;

#[test]
#[ignore = "fix this"]
fn test_pseudo_json_grammar() {
    let mut g = Grammar::new();

    // JSON format
    // Value -> Object | Array | String | Number
    // Object -> "{" (String ":" Value)? ("," String ":" Value)* "}"
    // Array -> "[" Value? ("," Value)* "]"
    // String -> "\"" Char* "\""
    // Number -> Digit+

    // Number: a simple integer number
    g.define(
        "Number",
        seq([
            opt(lit("-")),                         // Optional negative sign
            many(satisfy(|c| c.is_ascii_digit())), // Digits
        ]),
    );

    // String: double quotes with any characters (simple version)
    g.define(
        "String",
        seq([lit("\""), many(satisfy(|c| c != '"')), lit("\"")]),
    );

    // Array: "[" followed by an optional Value, followed by zero or more ", Value" and then "]"
    g.define(
        "Array",
        seq([
            lit("["),
            opt(rule_ref("Value".to_string())), // optional first value
            many(seq([lit(","), rule_ref("Value".to_string())])), // many comma-separated values
            lit("]"),
        ]),
    );

    // Object: "{" followed by key-value pairs (key: value), separated by commas, and then "}"
    g.define(
        "Object",
        seq([
            lit("{"),
            opt(seq([
                rule_ref("String".to_string()),
                lit(":"),
                rule_ref("Value".to_string()),
            ])),
            many(seq([
                lit(","),
                rule_ref("String".to_string()),
                lit(":"),
                rule_ref("Value".to_string()),
            ])),
            lit("}"),
        ]),
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

    let result = g.parse("Value", r#"{"name":"John", "age":30, "active":true}"#);
    println!("{:?}", result);
    assert!(result.is_ok());

    // if let Ok(value) = result {
    //     assert!(matches!(value, Value::List(_)), "파싱된 값이 List가 아님");
    // }
}

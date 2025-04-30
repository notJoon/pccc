use pccc::*;

macro_rules! parse_to_string {
    ($g:expr, $rule:expr, $input:expr) => {
        $g.parse($rule, $input).unwrap().value.to_string()
    };
}

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
    println!(
        "result: {:?}",
        parse_to_string!(g, "Value", "{\"name\":\"value\"}")
    );
    assert!(result.is_ok());

    let arr_result = g.parse("Value", "[1,2,3]");
    println!("arr_result: {:?}", parse_to_string!(g, "Value", "[1,2,3]"));
    assert!(arr_result.is_ok());

    let nested_result = g.parse("Value", "{\"arr\":[1,{\"key\":\"val\"}]}");
    println!(
        "nested_result: {:?}",
        parse_to_string!(g, "Value", "{\"arr\":[1,{\"key\":\"val\"}]}")
    );
    assert!(nested_result.is_ok());
}

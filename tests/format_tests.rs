#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_Monolith_Function()
{
    let Input = "int Add(int A, int B) { return A + B; }";
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), "int Add(int A, int B) { return A + B; }");
}

#[test]
fn Test_Expanded_Function_With_KR_Hugging_Removed()
{
    let Input = r#"void ExecuteTask(int A, int B) {
    DoFirst();
    DoSecond();
}"#;
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 50;
    // Canvas breach forces expansion

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let Expected = "void ExecuteTask(int A, int B)\n{\n    DoFirst();\n    DoSecond();\n}";
    assert_eq!(Formatted.trim(), Expected);
}

#[test]
fn Test_Multiline_Call_Detaching_Paren()
{
    let Input = r#"ProcessUserData(
    UserAccountNameFromDatabase,
    GlobalSystemIdentificationNumber,
    ListOfSpecificSecurityPermissions
);"#;
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 60;
    // Force expansion due to length

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("ProcessUserData\n("), "Opening paren must detach on multiline call");
    assert!(Formatted.contains("\n);"), "Closing paren must align and have semicolon");
}

#[test]
fn Test_Compound_Delimiter_Sole_Child_Chain()
{
    let Input = r#"InitializeCoreEngine[{(
    " SecurityModule",
    " DatabaseDriver",
    " RoutingInterface"
)}];"#;
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 60;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("[{(\n"), "Opening compound delimiter [{{( must sit on its own line");
    assert!(Formatted.contains("\n)}];"), "Closing compound delimiter )}}] must sit on its own line with semicolon glue");
}

#[test]
fn Test_Control_Flow_If_Else()
{
    let Input = r#"if (Condition) {
    Statement1();
} else {
    Statement2();
}"#;
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 20;
    // Canvas breach forces expansion of both clauses

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("if (Condition)\n{"), "Opening brace must detach after if condition");
    assert!(Formatted.contains("else\n{"), "Opening brace must detach after else");
}

#[test]
fn Test_Monolith_If_Else_Statement()
{
    let Input = "if (Condition) { Statement1(); } else { Statement2(); }";
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), "if (Condition) { Statement1(); } else { Statement2(); }");
}

#[test]
fn Test_Comments_Preservation()
{
    let Input = r#"// Important service header
void Foo() {
    // Inner work
    DoWork(); // Trailing comment
}"#;
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 60;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("// Important service header"));
    assert!(Formatted.contains("// Inner work"));
    assert!(Formatted.contains("// Trailing comment"));
}

#[test]
fn Test_CSharp_Generics_And_Strings()
{
    let Input = r#"class Service {
    List<string> Names;
    string Path = @" C: \ Data \ Files";
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::CSharp;
    Config.Canvas_Width = 60;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("class Service\n{"));
    assert!(Formatted.contains("List<string> Names;"));
    assert!(Formatted.contains(r#"@" C: \ Data \ Files""#));
}

#[test]
fn Test_Idempotence()
{
    let Input = r#"#include <stdio.h>

void Process(int Value)
{
    if (Value > 10)
    {
        printf(" High \ n");
    }
    else
    {
        printf(" Low \ n");
    }
}
"#;
    let Config = Config_Settings:: default();
    let FirstPass = Source_Formatter::Format(Input, & Config).expect("First pass failed");
    let SecondPass = Source_Formatter::Format( & FirstPass, & Config).expect("Second pass failed");
    assert_eq!(FirstPass, SecondPass, "Formatter must be strictly idempotent!");
}

#[test]
fn Test_Cpp_Template_Class()
{
    let Input = r#"template <typename T, typename Alloc>
class Container {
    T* Buffer;
};"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Cpp;
    Config.Canvas_Width = 30;
    // Canvas breach forces class body expansion

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("class Container\n{"));
    assert!(Formatted.contains("Buffer;"));
}

#[test]
fn Test_Cpp_Nested_Generics_Closer_Splitting()
{
    let Input = "std::vector<std::vector<int>> Matrix;";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Cpp;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), "std::vector<std::vector<int>> Matrix;");
}

#[test]
fn Test_Rust_Chained_Try_Operator_Not_Merged_As_Optional_Chaining()
{
    let Input = "fn Run() -> Result<(), ()>\n{\n    value?\n        .execute()\n}\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("value?.execute()"));
}

#[test]
fn Test_Blank_Line_Between_Top_Level_Functions()
{
    let Input = "fn First()\n{\n    DoA();\n}\nfn Second()\n{\n    DoB();\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("}\n\nfn Second()"), "Top level functions must be separated by a blank line");
}

#[test]
fn Test_Blank_Line_Between_Struct_And_Impl()
{
    let Input = "struct Point\n{\n    X: f64,\n    Y: f64,\n}\nimpl Point\n{\n    fn New() -> Self\n    {\n        Point { X: 0.0, Y: 0.0 }\n    }\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("}\n\nimpl Point"), "Struct and impl must be separated by a blank line");
}

#[test]
fn Test_Blank_Line_Between_Methods_In_Impl()
{
    let Input = "impl Point\n{\n    fn GetX(&self) -> f64\n    {\n        self.X\n    }\n    fn GetY(&self) -> f64\n    {\n        self.Y\n    }\n}\n";
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("\n\n    fn GetY"), "Methods in an impl must be separated by a blank line");
}

#[test]
fn Test_Blank_Line_After_Import_Block()
{
    let Input = "use std::io;\nuse std::fs;\nfn Main()\n{\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("use std::fs;\n\nfn Main()"), "Import block must be followed by a blank line before code");
}

#[test]
fn Test_Blank_Lines_Clamped_To_Max_Blank_Lines()
{
    let Input = "fn Alpha()\n{\n    DoA();\n}\n\n\n\n\nfn Beta()\n{\n    DoB();\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!( ! Formatted.contains("\n\n\n"), "Multiple blank lines must be clamped to at most one empty line");
    assert!(Formatted.contains("}\n\nfn Beta()"));
}

#[test]
fn Test_CSharp_Null_Coalescing_Operator()
{
    let Input = "string Name = Person?.Name ?? \"Default\";\nPerson ??= new Person();\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::CSharp;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("Person?.Name ?? \"Default\""), "Got: {}", Formatted);
    assert!(Formatted.contains("Person ??= new Person()"), "Got: {}", Formatted);
}

#[test]
fn Test_Rust_Chained_Try_Operator_Double_Question_Mark()
{
    let Input = "fn UnwrapDouble(Val: Option<Option<i32>>) -> Option<i32>\n{\n    Val??;\n    None\n}\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("Val??;"), "Rust try operators must not be merged as ?? binary operator, got: {}", Formatted);
}

#[test]
fn Test_Monolith_Preserves_Preceding_Newlines_And_Blank_Lines()
{
    let Input = r#"[package]
name = "lafs"
version = "0.1.0"
edition = "2021"

[lib]
name = "lafs"
path = "src/lib.rs"

[[bin]]
name = "lafs"
path = "src/main.rs"

[dependencies]
chrono = "0.4.45"
"#;
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("edition = \"2021\"\n\n[lib]"), "Monolith [lib] must not eat preceding blank line/newline, got: {}", Formatted);
    assert!(Formatted.contains("path = \"src/lib.rs\"\n\n[[bin]]"), "Monolith [[bin]] must not eat preceding blank line/newline, got: {}", Formatted);
    assert!(Formatted.contains("path = \"src/main.rs\"\n\n[dependencies]"), "Monolith [dependencies] must not eat preceding blank line/newline, got: {}", Formatted);
}

#[test]
fn Test_Monolith_Does_Not_Glue_Across_Single_Newline_After_Assignment()
{
    let Input = "edition = \"2021\"\n[lib]\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted, "edition = \"2021\"\n[lib]\n");
}

#[test]
fn Test_Standalone_Container_Preserves_Preceding_Newlines()
{
    let Input = "let x = 1;\n\n[1, 2, 3];\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("let x = 1;\n\n[1, 2, 3];"), "Standalone array must preserve preceding blank line, got: {}", Formatted);
}

#[test]
fn Test_Rust_Closure_Return_Type_Arrow_Spacing()
{
    let Input = "let run_cross_test = || -> anyhow::Result<Output> {\n    Ok(Output)\n};\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("|| -> anyhow::Result<Output>"), "Closure return type arrow must preserve spaces around ->, got: {}", Formatted);
    assert!( ! Formatted.contains("||->"), "Closure return type arrow must never fuse into ||->, got: {}", Formatted);
}

#[test]
fn Test_Rust_Function_Return_Type_Arrow_Spacing()
{
    let Input = "fn disabled() -> bool {\n    true\n}\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("fn disabled() -> bool"), "Function return type arrow must preserve spaces around ->, got: {}", Formatted);
}

#[test]
fn Test_Rust_Raw_String_Delimiter_Alignment()
{
    let Input = r##"fn test() {
    let p = project()
        .file(
            "src/main.rs",
            r#"
                #[macro_use]
                extern crate noop;

                fn main() {}
            "#,
        )
        .build();
}
"##;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("        r#\"\n            #[macro_use]"), "Opening r#\" and content must be aligned, got:\n{}", Formatted);
    assert!(Formatted.contains("        \"#,\n"), "Closing \"# must align on exact same column as r#\", got:\n{}", Formatted);
}

#[test]
fn Test_C_Preprocessor_Conditional_Braces()
{
    let Input = r#"void test() {
#ifdef OPTION_A
    while (cond_a) {
#else
    while (cond_b) {
        do_b();
#endif
        common_work();
    }
}
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::C;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting C preprocessor conditional braces failed");
    assert!(Formatted.contains("common_work();"));
}

#[test]
fn Test_C_Preprocessor_Macro_Allman_Style()
{
    let Input = r#"#define GetPKeyEC(obj, pkey) do { \
    GetPKey((obj), (pkey)); \
    if (EVP_PKEY_base_id(pkey) != EVP_PKEY_EC) { \
        ossl_raise(rb_eRuntimeError, "THIS IS NOT A EC PKEY!"); \
    } \
} while (0)
#define GetEC(obj, key) do { \
    EVP_PKEY *_pkey; \
    GetPKeyEC(obj, _pkey); \
    (key) = EVP_PKEY_get0_EC_KEY(_pkey); \
    if ((key) == NULL) \
        ossl_raise(ePKeyError, "failed to get EC_KEY from EVP_PKEY"); \
} while (0)

#define GetECGroup(obj, group) do { \
    TypedData_Get_Struct(obj, EC_GROUP, &ossl_ec_group_type, group); \
    if ((group) == NULL) \
        ossl_raise(eEC_GROUP, "EC_GROUP is not initialized"); \
} while (0)

#define GetECPoint(obj, point) do { \
    TypedData_Get_Struct(obj, EC_POINT, &ossl_ec_point_type, point); \
    if ((point) == NULL) \
        ossl_raise(eEC_POINT, "EC_POINT is not initialized"); \
} while (0)
#define GetECPointGroup(obj, group) do { \
    VALUE _group = rb_attr_get(obj, id_i_group); \
    GetECGroup(_group, group); \
} while (0)
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::C;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed for C preprocessor macros");

    // Verify Allman style formatting:
    // 1. do is detached from {
    assert!(Formatted.contains("#define GetPKeyEC(obj, pkey) do \\\n{ \\"), "do and open brace must be detached into Allman style, got:\n{}", Formatted);
    // 2. if is detached from {
    assert!
    (
        Formatted.contains("    if (EVP_PKEY_base_id(pkey) != EVP_PKEY_EC) \\\n    { \\"),
        "if and open brace must be detached into Allman style, got:\n{}", Formatted
    );
    // 3. closing brace of do while
    assert!(Formatted.contains("} while (0)"), "Closing brace and while (0) must be present, got:\n{}", Formatted);

    // 4. Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Raw_String_With_Non_Ascii_Unicode_Characters()
{
    // Test that non-ASCII Unicode characters (such as \u{85} NEL, emojis, Cyrillic)
    // preceding C# and C++ raw string literals do not panic due to char/byte index mismatch.
    let Input = "string x = \"\u{85}\u{85}🦀🚀\";\nstring Raw = \"\"\"Hello \"\" World\"\"\";\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::CSharp;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed for C# raw string with non-ASCII chars");
    assert!(Formatted.contains("\"\"\"Hello \"\" World\"\"\""));

    let CppInput = "const char* s = \"\u{85}🌍\";\nconst char* r = R\"delim(content)delim\";\n";
    Config.Language = Source_Language::Cpp;
    let FormattedCpp = Source_Formatter::Format(CppInput, & Config).expect("Formatting failed for C++ raw string with non-ASCII chars");
    assert!(FormattedCpp.contains("R\"delim(content)delim\""));
}

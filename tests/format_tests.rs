#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_Monolith_Function()
{
    let Input = "int Add(int A, int B) { return A + B; }";
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(),"int Add(int A, int B) { return A + B; }");
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
    assert!(Formatted.contains("ProcessUserData\n("),"Opening paren must detach on multiline call");
    assert!(Formatted.contains("\n);"),"Closing paren must align and have semicolon");
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
    assert!(Formatted.contains("[{(\n"),"Opening compound delimiter [{{( must sit on its own line");
    assert!(Formatted.contains("\n)}];"),"Closing compound delimiter )}}] must sit on its own line with semicolon glue");
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
    assert!(Formatted.contains("if (Condition)\n{"),"Opening brace must detach after if condition");
    assert!(Formatted.contains("else\n{"),"Opening brace must detach after else");
}

#[test]
fn Test_Monolith_If_Else_Statement()
{
    let Input = "if (Condition) { Statement1(); } else { Statement2(); }";
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(),"if (Condition) { Statement1(); } else { Statement2(); }");
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
    assert_eq!(Formatted.trim(),"std::vector<std::vector<int>> Matrix;");
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
    assert!(Formatted.contains("}\n\nfn Second()"),"Top level functions must be separated by a blank line");
}

#[test]
fn Test_Blank_Line_Between_Struct_And_Impl()
{
    let Input = "struct Point\n{\n    X: f64,\n    Y: f64,\n}\nimpl Point\n{\n    fn New() -> Self\n    {\n        Point { X: 0.0, Y: 0.0 }\n    }\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("}\n\nimpl Point"),"Struct and impl must be separated by a blank line");
}

#[test]
fn Test_Blank_Line_Between_Methods_In_Impl()
{
    let Input = "impl Point\n{\n    fn GetX(&self) -> f64\n    {\n        self.X\n    }\n    fn GetY(&self) -> f64\n    {\n        self.Y\n    }\n}\n";
    let mut Config = Config_Settings:: default();
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("\n\n    fn GetY"),"Methods in an impl must be separated by a blank line");
}

#[test]
fn Test_Blank_Line_After_Import_Block()
{
    let Input = "use std::io;\nuse std::fs;\nfn Main()\n{\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("use std::fs;\n\nfn Main()"),"Import block must be followed by a blank line before code");
}

#[test]
fn Test_Blank_Lines_Clamped_To_Max_Blank_Lines()
{
    let Input = "fn Alpha()\n{\n    DoA();\n}\n\n\n\n\nfn Beta()\n{\n    DoB();\n}\n";
    let Config = Config_Settings:: default();

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!( ! Formatted.contains("\n\n\n"),"Multiple blank lines must be clamped to at most one empty line");
    assert!(Formatted.contains("}\n\nfn Beta()"));
}

#[test]
fn Test_CSharp_Null_Coalescing_Operator()
{
    let Input = "string Name = Person?.Name ?? \"Default\";\nPerson ??= new Person();\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::CSharp;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("Person?.Name ?? \"Default\""),"Got: {}", Formatted);
    assert!(Formatted.contains("Person ??= new Person()"),"Got: {}", Formatted);
}

#[test]
fn Test_Rust_Chained_Try_Operator_Double_Question_Mark()
{
    let Input = "fn UnwrapDouble(Val: Option<Option<i32>>) -> Option<i32>\n{\n    Val??;\n    None\n}\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("Val??;"),"Rust try operators must not be merged as ?? binary operator, got: {}", Formatted);
}

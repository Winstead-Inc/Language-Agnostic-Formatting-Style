#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use lafs:: { Config_Settings, Kotlin_Style, Source_Formatter, Source_Language };

#[test]
fn Test_Kotlin_Monolith_Function()
{
    let Input = "fun Add(A: Int, B: Int): Int { return A + B; }";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(),"fun Add(A: Int, B: Int): Int { return A + B; }");
}

#[test]
fn Test_Kotlin_V1_Expanded_Function_Detached_Brace()
{
    // Version 1: Pure Allman style for all containers
    let Input = r#"fun ProcessData(Item: String, Count: Int): Boolean {
    StepOne(Item);
    StepTwo(Count);
    return true;
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Allman;
    Config.Canvas_Width = 80;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let Expected = "fun ProcessData(Item: String, Count: Int): Boolean\n{\n    StepOne(Item);\n    StepTwo(Count);\n    return true;\n}";
    assert_eq!(Formatted.trim(), Expected);
}

#[test]
fn Test_Kotlin_V1_Function_Parameters_Multiline_Detached()
{
    let Input = r#"fun ExecuteLongTask(
    FirstParameterWithLongName: String,
    SecondParameterWithLongName: Int
): Boolean {
    RunTask();
    return true;
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Allman;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("fun ExecuteLongTask\n(\n"),"In Version 1, multiline parameters detach '(' onto its own line.\nGot:\n{}", Formatted);
    assert!(Formatted.contains("): Boolean\n{\n"),"Closing paren must be followed by ': Boolean' and detached body brace.\nGot:\n{}", Formatted);
}

#[test]
fn Test_Kotlin_V1_Trailing_Lambda_Detached()
{
    // Version 1: Allman for all containers detaches trailing lambdas
    let Input = r#"val Result = Items.map {
    val Doubled = it * 2;
    Doubled + 1;
};"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Allman;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!
    (
        Formatted.contains("Items.map\n{\n"),
        "In Version 1 (Allman for all containers), trailing lambda brace must detach onto its own line.\nGot:\n{}",
        Formatted
    );
}

#[test]
fn Test_Kotlin_V1_Call_Arguments_Detached()
{
    // Version 1: Allman for all containers detaches call arguments
    let Input = r#"val Result = CalculateTask(
    FirstArgumentValue,
    SecondArgumentValue
);"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Allman;
    Config.Canvas_Width = 30;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("CalculateTask\n(\n"),"In Version 1, call parentheses must detach onto their own line.\nGot:\n{}", Formatted);
}

#[test]
fn Test_Kotlin_V2_Trailing_Lambda_Hugs()
{
    // Version 2 (Official Kotlin / Brain KanceR):
    // Trailing lambda must hug the call/identifier on the same line to satisfy Kotlin compiler ASI grammar
    let Input = r#"val Result = Items.map {
    val Doubled = it * 2;
    Doubled + 1;
};"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::KotlinOfficial;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!
    (
        Formatted.contains("Items.map {\n"),
        "In Version 2 (Official Kotlin), trailing lambda must hug the call on the same line (Column-Anchored Hybrid).\nGot:\n{}",
        Formatted
    );
    assert!(Formatted.contains("\n}\n") || Formatted.contains("\n};"),"Closing brace must align correctly");
}

#[test]
fn Test_Kotlin_V2_Call_With_Trailing_Lambda()
{
    // Version 2: Call arguments followed by trailing lambda
    // `foo(arg) { ... }` where `(` hugs `foo`, `)` closes, and `{` hugs `)`
    let Input = r#"View.setOnClickListener(Listener) {
    PerformAction();
    UpdateState();
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Official;
    Config.Canvas_Width = 30;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!
    (
        Formatted.contains("View.setOnClickListener(Listener) {\n")
        || Formatted.contains(") {\n"),
        "Trailing lambda must hug closing paren in Version 2.\nGot:\n{}",
        Formatted
    );
}

#[test]
fn Test_Kotlin_V2_Call_Arguments_Hugs()
{
    // Version 2 (Official Kotlin): Call arguments '(' must hug the function name
    let Input = r#"val Result = CalculateTask(
    FirstArgumentValue,
    SecondArgumentValue
);"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Official;
    Config.Canvas_Width = 30;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("CalculateTask(\n"),"In Version 2, call arguments must hug the callee on the same line.\nGot:\n{}", Formatted);
}

#[test]
fn Test_Kotlin_V2_Function_And_Class_Detached_Braces()
{
    // In Version 2 (Official Kotlin), functions and classes STILL use detached Allman braces
    // because kotlinc natively supports Allman braces for declarations!
    let Input = r#"class UserService(val Repo: Repository) {
    fun FindUser(Id: Int): User? {
        CheckAuth();
        return Repo.Find(Id);
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::KotlinOfficial;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("class UserService(val Repo: Repository)\n{\n"),"Class body must have detached brace in Version 2.\nGot:\n{}", Formatted);
    assert!(Formatted.contains("fun FindUser(Id: Int): User?\n    {\n"),"Function body must have detached brace in Version 2.\nGot:\n{}", Formatted);
}

#[test]
fn Test_Kotlin_V2_Indexing_Hugs()
{
    // Version 2: Indexing suffix '[' must hug the expression
    let Input = r#"val Item = Matrix[
    RowIndex,
    ColumnIndex
];"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Official;
    Config.Canvas_Width = 25;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("Matrix[\n"),"In Version 2, indexing bracket must hug the expression.\nGot:\n{}", Formatted);
}

#[test]
fn Test_Kotlin_Control_Structures()
{
    // if-else, while, try-catch-finally in Kotlin have detached braces in both V1 and V2
    let Input = r#"if (Condition) {
    ActionA();
} else {
    ActionB();
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("if (Condition)\n{\n    ActionA();\n}\nelse\n{\n    ActionB();\n}"));
}

#[test]
fn Test_Kotlin_When_Expression()
{
    let Input = r#"when (Value) {
    1 -> DoOne();
    2 -> DoTwo();
    else -> {
        DoFallback();
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("when (Value)\n{\n"));
    assert!(Formatted.contains("else ->\n    {\n        DoFallback();\n    }"));
}

#[test]
fn Test_Kotlin_Compound_Delimiter_Sole_Child_Chain()
{
    let Input = r#"InitializeKotlinEngine([{(
    "SecurityModule",
    "DatabaseModule"
)}]);"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!
    (
        Formatted.contains("([{(\n") && Formatted.contains("\n)}]);"),
        "Sole-Child Chain must fuse into compound delimiters ([{{( ... )}}]).\nGot:\n{}",
        Formatted
    );
}

#[test]
fn Test_Kotlin_Raw_Multiline_String_Verbatim()
{
    // Multi-line raw strings `"""..."""` must be preserved with 100% byte-for-byte fidelity
    let Input = "val Query = \"\"\"\n    SELECT *\n    FROM users\n    WHERE status = 'ACTIVE'\n\"\"\".trimIndent();";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), Input);
}

#[test]
fn Test_Kotlin_Backtick_Identifiers()
{
    let Input = r#"fun `test calculation with invalid inputs`() {
    AssertThrows();
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Canvas_Width = 80;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("fun `test calculation with invalid inputs`()\n{\n    AssertThrows();\n}"));
}

#[test]
fn Test_Kotlin_Nested_Block_Comments()
{
    let Input = r#"/*
 * Outer comment
 * /* Nested block comment */
 * Back to outer comment
 */
fun Main() {
    println("OK");
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("/* Nested block comment */"));
    assert!(Formatted.contains("fun Main()"));
}

#[test]
fn Test_Kotlin_Operators_Formatting()
{
    let Input = r#"val Result = User?.Address?.City ?: "DefaultCity";
val Numbers = 1..10;
val Range = 0..<MaxCount;
val ClassRef = String::class;
val ForceNonNull = Value!!;"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("User?.Address?.City ?: \"DefaultCity\";"));
    assert!(Formatted.contains("1..10"));
    assert!(Formatted.contains("0..<MaxCount"));
    assert!(Formatted.contains("String::class"));
    assert!(Formatted.contains("Value!!;"));
}

#[test]
fn Test_Kotlin_Enum_Class()
{
    let Input = r#"enum class Direction {
    NORTH,
    SOUTH,
    WEST,
    EAST;

    fun IsVertical(): Boolean {
        return this == NORTH || this == SOUTH;
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("enum class Direction\n{\n"));
    assert!(Formatted.contains("EAST;\n"));
    assert!(Formatted.contains("fun IsVertical(): Boolean\n    {\n"));
}

#[test]
fn Test_Kotlin_V1_Idempotence()
{
    let Input = r#"class Account(val Id: String, var Balance: Double) {
    fun Deposit(Amount: Double) {
        if (Amount <= 0.0) {
            throw IllegalArgumentException("Amount must be positive");
        }
        Balance += Amount;
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Kotlin;
    Config.Kotlin_Style = Kotlin_Style::Allman;
    Config.Canvas_Width = 40;

    let FormattedFirst = Source_Formatter::Format(Input, & Config).expect("First formatting pass failed");
    let FormattedSecond = Source_Formatter::Format( & FormattedFirst, & Config).expect("Second formatting pass failed");
    assert_eq!(FormattedFirst, FormattedSecond, "Formatting must be 100% idempotent across multiple passes in Version 1");
}

#[test]
fn Test_Kotlin_V2_Idempotence()
{
    let Input = r#"class Account(val Id: String, var Balance: Double) {
    fun Deposit(Amount: Double) {
        val Log = Transactions.map {
            it.Format()
        };
        Balance += Amount;
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::KotlinOfficial;
    Config.Canvas_Width = 40;

    let FormattedFirst = Source_Formatter::Format(Input, & Config).expect("First formatting pass failed");
    let FormattedSecond = Source_Formatter::Format( & FormattedFirst, & Config).expect("Second formatting pass failed");
    assert_eq!(FormattedFirst, FormattedSecond, "Formatting must be 100% idempotent across multiple passes in Version 2");
}

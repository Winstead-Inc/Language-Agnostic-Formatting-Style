#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_JS_Monolith_Function()
{
    let Input = "function Add(A, B) { return A + B; }";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), "function Add(A, B) { return A + B; }");
}

#[test]
fn Test_JS_Expanded_Function_Detached_Brace()
{
    let Input = r#"function ExecuteTask(A, B) {
    DoFirst(A);
    DoSecond(B);
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let Expected = "function ExecuteTask(A, B)\n{\n    DoFirst(A);\n    DoSecond(B);\n}";
    assert_eq!(Formatted.trim(), Expected);
}

#[test]
fn Test_JS_Arrow_Function()
{
    let Input = r#"const Calculate = (A, B) => {
    StepOne(A);
    StepTwo(B);
};"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("const Calculate = (A, B) =>\n{\n"));
    assert!(Formatted.trim_end().ends_with("\n};"));
}

#[test]
fn Test_JS_ASI_Column_Anchored_Hybrid_On_Return()
{
    // OLAFS ASI Rule: after return, the opening brace MUST remain on the return line
    // to prevent JavaScript ASI from returning undefined
    let Input = r#"function GetData() {
    return {
        Status: 200,
        Payload: " OK"
    };
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("return {\n"), "Opening brace after return must remain on the return line (Column-Anchored Hybrid)");
    assert!(Formatted.contains("           };\n"), "Closing brace must align with column-anchored hybrid opener column (column 11)");
}

#[test]
fn Test_JS_CAH_Option_Disabled_Uses_Pure_Allman()
{
    let Input = r#"function GetData() {
    return {
        Status: 200,
        Payload: " OK"
    };
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;
    Config.Column_Anchored_Hybrid = false;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("return\n"), "When CAH is disabled, return must have a newline after it");
    assert!(Formatted.contains("    {\n"), "When CAH is disabled, opening brace must be detached Allman style");
}

#[test]
fn Test_JS_Template_Literal_Preservation()
{
    let Input = r#"const Message = `Hello, ${User.Name}!
Welcome to the system.
Your balance is: ${Account.Balance}.`;"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), Input.trim());
}

#[test]
fn Test_JS_Regex_Literal_Vs_Division()
{
    let Input = r#"const IsMatch = /^[a-z0-9_-]+$/i.test(Str);
const Half = Total / 2;"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("/^[a-z0-9_-]+$/i.test(Str);"));
    assert!(Formatted.contains("Total / 2;"));
}

#[test]
fn Test_JS_Compound_Delimiter_Sole_Child_Chain()
{
    let Input = r#"const Config = [{(
    Module: " Security",
    Active: true
)}];"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("[{(\n"));
    assert!(Formatted.contains("\n)}];"));
}

#[test]
fn Test_TS_Interface_And_Enum()
{
    let Input = r#"interface User_Profile {
    Id: number;
    Name: string;
    Active: boolean;
}

enum Status_Code {
    Success,
    Failure
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::TypeScript;
    Config.Canvas_Width = 30;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("interface User_Profile\n{\n"));
    assert!(Formatted.contains("enum Status_Code\n{\n"));
}

#[test]
fn Test_JS_Control_Flow_If_Else()
{
    let Input = r#"if (Condition) {
    DoAlpha();
}
else {
    DoBeta();
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 15;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let Expected = "if (Condition)\n{\n    DoAlpha();\n}\nelse\n{\n    DoBeta();\n}";
    assert_eq!(Formatted.trim(), Expected);
}

#[test]
fn Test_JS_Class_Declaration()
{
    let Input = r#"class Service_Manager {
    constructor(Config) {
        this.Config = Config;
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("class Service_Manager\n{\n"));
    assert!(Formatted.contains("constructor(Config)\n    {\n"));
}

#[test]
fn Test_JS_Idempotence()
{
    let Input = r#"function CalculateTotal(Items)
{
    let Total = 0;
    for (const Item of Items)
    {
        Total += Item.Price;
    }
    return Total;
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let FirstPass = Source_Formatter::Format(Input, & Config).expect("First pass failed");
    let SecondPass = Source_Formatter::Format( & FirstPass, & Config).expect("Second pass failed");
    assert_eq!(FirstPass, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_JS_Nullish_Coalescing_Operator()
{
    let Input = "const Value = A ?? B;\nconst Other = X ??= Y;\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("A ?? B"), "Formatter should keep ?? intact without whitespace, got: {}", Formatted);
    assert!(Formatted.contains("X ??= Y"), "Formatter should keep ??= intact without whitespace, got: {}", Formatted);
}

#[test]
fn Test_JS_Nullish_Coalescing_With_Optional_Chaining()
{
    let Input = "const Name = User?.Profile?.Name ?? \"Guest\";\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("User?.Profile?.Name ?? \"Guest\""), "Got: {}", Formatted);
}

#[test]
fn Test_JS_Nullish_Coalescing_Chained_And_Ternary()
{
    let Input = "const Result = Condition ? A ?? B ?? C : DefaultValue;\n";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("A ?? B ?? C"), "Got: {}", Formatted);
}

#[test]
fn Test_JS_One_Liner_If_Parenthesis_Monolith()
{
    let Input = r#"function matchPassContains(index, longIndex, queries, regexps)
{
    if
    (
        index.indexOf(queries[0]) == - 1
    )
    return false;
    for (var i = 1, l = regexps.length; i < l; i ++)
    {
        if
        (
            ! index.match(regexps[i]) && ! longIndex.match(regexps[i])
        )
        return false;
    };
    return true;
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("if (index.indexOf(queries[0]) == - 1)"), "Expected monolithic if paren, got:\n{}", Formatted);
    assert!(Formatted.contains("if (!index.match(regexps[i]) && !longIndex.match(regexps[i]))"), "Expected monolithic if paren, got:\n{}", Formatted);
}

#[test]
fn Test_JS_Torture_Test_Slop_And_Idempotence()
{
    let SlopPath = std::path::Path:: new ("PEAK K&R BRAIN ROT SLOP.JS");
    if ! SlopPath.exists() { return ; }
    let SlopSource = std::fs::read_to_string(SlopPath).expect("Failed to read torture slop test file");

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let FormattedPass1 = Source_Formatter::Format( & SlopSource, & Config).expect("Pass 1 of JS torture test formatting failed");
    let FormattedPass2 = Source_Formatter::Format( & FormattedPass1, & Config).expect("Pass 2 of JS torture test formatting failed");

    if FormattedPass1 != FormattedPass2
    {
        let Lines1: Vec < & str > = FormattedPass1 .lines().collect();
        let Lines2: Vec < & str > = FormattedPass2 .lines().collect();
        for (idx, (l1, l2)) in Lines1 .iter().zip(Lines2 .iter()).enumerate()
        { if l1 != l2 { eprintln ! ("First idempotence divergence at line {}:\nPass 1: {:?}\nPass 2: {:?}", idx + 1, l1, l2); break ; } }
        if Lines1 .len() != Lines2 .len()
        { eprintln ! ("Line count mismatch: Pass 1 has {} lines, Pass 2 has {} lines", Lines1 .len(), Lines2 .len()); }
    }

    assert_eq!(FormattedPass1, FormattedPass2, "JS formatting must be 100% idempotent");

    // Invariant verifications
    assert!(FormattedPass1 .contains("Nightmare_Type_Matrix"), "Nightmare type matrix present");
    assert!(FormattedPass1 .contains("GLOBAL_SPECTRUM_CONFIG"), "Config present");
    assert!(FormattedPass1 .contains("embeddedGlslShader"), "GLSL shader present");
    assert!(FormattedPass1 .contains("renderDiagnosticsPanel"), "JSX panel present");
}

#[test]
fn Test_JS_JSX_Diagnostics_Panel()
{
    let Input = r##"
function renderDiagnosticsPanel(props)
{
    return (
        <section className = "DiagnosticsContainer"
            data-engine-id = { this.instanceId }
            data-theme-primary = "#7100FF">
            <header className = "HeaderBar" title = "System Telemetry Header"
                data-count = { props.items.length }
                data-mask = { this.#internalSecretMask }>
                <h1>{ props.title }</h1>
            </header>
            <div className = "ContentGrid">
            {
                props.items.map
                (
                    (item, idx) =>
                    (
                        <article key = { item.id }
                            data-item-index = { idx }
                            data-item-hex = { item.hex }
                            data-item-label = { item.label }>
                            <span>{ item.label }</span>
                        </article>
                    )
                )
            }
            </div>
        </section>
    );
}
"##;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;

    let FormattedPass1 = Source_Formatter::Format(Input, & Config).expect("Pass 1 JSX formatting failed");
    let FormattedPass2 = Source_Formatter::Format( & FormattedPass1, & Config).expect("Pass 2 JSX formatting failed");

    assert_eq!(FormattedPass1, FormattedPass2, "JSX formatting must be 100% idempotent");
    assert!( ! FormattedPass1 .contains("data - engine - id"), "Should not space data-engine-id");
    assert!( ! FormattedPass1 .contains("</ header>"), "Should not space </ header>");
    assert!( ! FormattedPass1 .contains("</ section>"), "Should not space </ section>");
    assert!( ! FormattedPass1 .contains("</ div>"), "Should not space </ div>");
    assert!( ! FormattedPass1 .contains("this.# internalSecretMask"), "Should not space private field #internalSecretMask");
    assert!(FormattedPass1 .contains("data-engine-id"), "data-engine-id preserved intact");
    assert!(FormattedPass1 .contains("</header>"), "</header> preserved intact");
    assert!(FormattedPass1 .contains("this.#internalSecretMask"), "this.#internalSecretMask preserved intact");
}

#[test]
fn Test_JS_Nightmare_Type_Matrix()
{
    let Input = r#"export type Nightmare_Type_Matrix <
const T extends readonly Record <
string, { readonly id: number; readonly tone:`#${string}`; readonly nested ?: readonly[number, ...Array < { tag: string; active: boolean; }>]; }
>[],
U extends keyof T[number] = keyof T[number]
> = T extends readonly[infer Head extends Record <string, unknown>, ...infer Tail extends readonly Record <string, unknown>[]] ?
{
    readonly[K in keyof Head as K extends string ?`get_${Capitalize <K>}`: never]: Head[K] extends { readonly tone: infer ToneHex extends string; }
    ? (input: ToneHex, options ?: { readonly strict: boolean; readonly fallback: '#7100FF' | '#FC6A03' | '#FF1744'; }) => Promise <
    ReadonlyArray < { readonly resolved: ToneHex; readonly tailMatrix: Nightmare_Type_Matrix <Tail>; }
    >
    >: never;
}
: { readonly emptyMarker: '#00BFFF'; };
"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let Expected = r#"export type Nightmare_Type_Matrix
<
    const T extends readonly Record
    <
        string,
        {
            readonly id: number;
            readonly tone: `#${string}`;
            readonly nested?: readonly [number, ...Array<{ tag: string; active: boolean; }>];
        }
    >[],
    U extends keyof T[number] = keyof T[number]
> = T extends readonly
[
    infer Head extends Record<string, unknown>,
    ...infer Tail extends readonly Record<string, unknown>[]
]
?
{
    readonly [K in keyof Head as K extends string ? `get_${Capitalize<K>}` : never]:
        Head[K] extends { readonly tone: infer ToneHex extends string; }
        ?
        (
            input: ToneHex,
            options?: { readonly strict: boolean; readonly fallback: '#7100FF' | '#FC6A03' | '#FF1744'; }
        ) => Promise
        <
            ReadonlyArray
            <{
                readonly resolved: ToneHex;
                readonly tailMatrix: Nightmare_Type_Matrix<Tail>;
            }>
        >
        : never;
}
: { readonly emptyMarker: '#00BFFF'; };
"#;

    assert_eq!(Formatted, Expected);
}

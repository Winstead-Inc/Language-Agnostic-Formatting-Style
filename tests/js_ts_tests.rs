#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_JS_Comparison_Angle_Not_Generic()
{
    let Input = r#"
if (p < ku && (I = Bu(D, O, P, j, R, F, B, L)))
{
    var Y = D - I[0];
}
k > wu ? 1 : 2;
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::JavaScript;
    let Formatted = Source_Formatter::Format(Input, & Config).expect("Should format comparison operators without treating them as generics");
    assert!(Formatted.contains("p < ku"));
}

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
        let Lines1: Vec< & str> = FormattedPass1 .lines().collect();
        let Lines2: Vec< & str> = FormattedPass2 .lines().collect();
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
fn Test_TSX_Brainrot_Format()
{
    let Path = std::path::Path::new("J-TS K&R BRAINROT.TSX");
    if !Path.exists() { return; }
    let Source = std::fs::read_to_string(Path).expect("Failed to read");
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Tsx;
    Config.Canvas_Width = 160;

    let FormattedPass1 = Source_Formatter::Format(&Source, &Config).expect("Pass 1 of TSX torture test formatting failed");
    let FormattedPass2 = Source_Formatter::Format(&FormattedPass1, &Config).expect("Pass 2 of TSX torture test formatting failed");

    assert_eq!(FormattedPass1, FormattedPass2, "TSX formatting must be 100% idempotent");

    // Invariant verifications
    assert!(FormattedPass1.contains("data-node-name = { row.nodeName }"), "Hyphenated JSX attribute intact: {}", FormattedPass1);
    assert!(FormattedPass1.contains("data-latency-ms = { row.latencyMs }"), "Hyphenated JSX attribute intact");
    assert!(FormattedPass1.contains("data-tag-idx = { tagIdx }"), "Hyphenated JSX attribute intact");
    assert!(FormattedPass1.contains("data-parent-idx = { rowIdx }"), "Hyphenated JSX attribute intact");
    assert!(FormattedPass1.contains("data-badge-color = \"#C77DFF\""), "Hyphenated JSX attribute intact");
    assert!(FormattedPass1.contains("i < count"), "Relational < spaced intact");
    assert!(FormattedPass1.contains("count > 0"), "Relational > spaced intact");
    assert!(FormattedPass1.contains("i++"), "Postfix ++ attached without space");
    assert!(!FormattedPass1.contains("i ++"), "No spaced postfix i ++");
    assert!(!FormattedPass1.contains("\n=>"), "Arrow operator => must never be on its own line");
    assert!(FormattedPass1.contains(")()"), "IIFE call invocation must remain attached as )()");
    assert!(!FormattedPass1.contains(")\n()"), "No detached IIFE call invocation");
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
        ) => Promise<ReadonlyArray<{ readonly resolved: ToneHex; readonly tailMatrix: Nightmare_Type_Matrix<Tail>; }>>: never;
}
: { readonly emptyMarker: '#00BFFF'; };
"#;

    assert_eq!(Formatted, Expected);
}

#[test]
fn Test_TS_Generic_Type_Monolith_Only()
{
    let Input = "type Result = Promise<{ error?: Error; results: Record<string, unknown>; }>;\n";
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::TypeScript;
    Config.Canvas_Width = 160;

    let FormattedMonolith = Source_Formatter::Format(Input, &Config).expect("Monolith format failed");
    assert_eq!(FormattedMonolith.trim(), "type Result = Promise<{ error?: Error; results: Record<string, unknown>; }>;");

    // User requirement: Switch to Monolith only for (<{, <) generic type definitions to prevent syntax errors, even on narrow canvas
    Config.Canvas_Width = 40;
    let FormattedNarrow = Source_Formatter::Format(Input, &Config).expect("Narrow format failed");
    assert_eq!(FormattedNarrow.trim(), "type Result = Promise<{ error?: Error; results: Record<string, unknown>; }>;");

    // Test <( with function type / parameter list (Monolith only)
    let InputFn = "type Fn = Promise<(input: string) => void>;\n";
    Config.Canvas_Width = 40;
    let FormattedFnMonolith = Source_Formatter::Format(InputFn, &Config).expect("Fn monolith failed");
    assert_eq!(FormattedFnMonolith.trim(), "type Fn = Promise<(input: string) => void>;");

    // Idempotence verification
    let SecondPass = Source_Formatter::Format(&FormattedNarrow, &Config).expect("Idempotence failed");
    assert_eq!(FormattedNarrow, SecondPass);
}

#[test]
fn Test_TS_Const_Arrow_Function_Monolith_Only()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::TypeScript;
    Config.Canvas_Width = 160;

    // Test 1: Exact line 7 from user screenshot
    let Input1 = "const isFindQuery = (query: CursorConstructionOptionsWithChains,): query is CursorConstructionOptionsWithChains & { options: { method: 'find' }; } => { return query.options.method === 'find'; };\n";
    let Formatted1 = Source_Formatter::Format(Input1, &Config).expect("Format 1 failed");
    assert_eq!(Formatted1.trim(), "const isFindQuery = (query: CursorConstructionOptionsWithChains,): query is CursorConstructionOptionsWithChains & { options: { method: 'find' }; } => { return query.options.method === 'find'; };");
    assert!(!Formatted1.contains("\n=>"), "Arrow operator => must never be placed on its own line: {}", Formatted1);

    // Test 2: Exact line 9 from user screenshot (where options had method: 'aggregate')
    let Input2 = "const isAggregateQuery = (query: CursorConstructionOptionsWithChains,): query is CursorConstructionOptionsWithChains & { options: {\nmethod: 'aggregate' }; }\n=> { return query.options.method === 'aggregate'; };\n";
    let Formatted2 = Source_Formatter::Format(Input2, &Config).expect("Format 2 failed");
    assert_eq!(Formatted2.trim(), "const isAggregateQuery = (query: CursorConstructionOptionsWithChains,): query is CursorConstructionOptionsWithChains & { options: { method: 'aggregate' }; } => { return query.options.method === 'aggregate'; };");
    assert!(!Formatted2.contains("\n=>"), "Arrow operator => must never be placed on its own line: {}", Formatted2);

    // Test 3: Exact line 12 from user screenshot
    let Input3 = "const isFindParsedQuery = (query: ParsedQuery,): query is ParsedQuery & { find: FindQuery }\n=> { return 'find' in query; };\n";
    let Formatted3 = Source_Formatter::Format(Input3, &Config).expect("Format 3 failed");
    assert_eq!(Formatted3.trim(), "const isFindParsedQuery = (query: ParsedQuery,): query is ParsedQuery & { find: FindQuery } => { return 'find' in query; };");
    assert!(!Formatted3.contains("\n=>"), "Arrow operator => must never be placed on its own line: {}", Formatted3);

    // Test 4: Exact line 15 from user screenshot
    let Input4 = "const isAggregateParsedQuery = (query: ParsedQuery,): query is ParsedQuery & { aggregate: AggregateQuery }\n=> { return 'aggregate' in query; };\n";
    let Formatted4 = Source_Formatter::Format(Input4, &Config).expect("Format 4 failed");
    assert_eq!(Formatted4.trim(), "const isAggregateParsedQuery = (query: ParsedQuery,): query is ParsedQuery & { aggregate: AggregateQuery } => { return 'aggregate' in query; };");
    assert!(!Formatted4.contains("\n=>"), "Arrow operator => must never be placed on its own line: {}", Formatted4);

    // Test 5: Arrow function with multiple statements retains attached => and detached body
    let InputMulti = "const handler = (event: Event) => {\n    log(event);\n    return true;\n};\n";
    Config.Canvas_Width = 40;
    let FormattedMulti = Source_Formatter::Format(InputMulti, &Config).expect("Format multi failed");
    assert!(FormattedMulti.contains("const handler = (event: Event) =>\n{\n"), "Body must expand with detached brace while => remains attached: {}", FormattedMulti);
    assert!(!FormattedMulti.contains("\n=>"), "Arrow operator => must never be on its own line: {}", FormattedMulti);
}

#[test]
fn Test_JS_Preserve_Empty_Lines()
{
    let Input = r#"function DoWork()
{

    const x = 1;

    const obj =
    {
        a: 1,
        b: 2,
    };

}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::JavaScript;
    Config.Canvas_Width = 40;
    Config.Preserve_Empty_Lines = true;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains("{\n\n    const x = 1;"), "Empty line after opener preserved: {}", Formatted);
    assert!(Formatted.contains("};\n\n}"), "Empty line before closer preserved: {}", Formatted);

    Config.Preserve_Empty_Lines = false;
    let Formatted_Stripped = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(!Formatted_Stripped.contains("{\n\n"), "Empty line after opener stripped: {}", Formatted_Stripped);
    assert!(!Formatted_Stripped.contains("};\n\n}"), "Empty line before closer stripped: {}", Formatted_Stripped);
}

#[test]
fn Test_JSX_Self_Closing_And_Props()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Jsx;
    Config.Canvas_Width = 160;

    // Self-closing component with space before />
    let Input1 = "const elem = <Component />;\n";
    let Formatted1 = Source_Formatter::Format(Input1, &Config).expect("Format 1 failed");
    assert_eq!(Formatted1.trim(), "const elem = <Component />;");

    // Single-line component with prop
    let Input2 = "const elem = <Component prop={value} />;\n";
    let Formatted2 = Source_Formatter::Format(Input2, &Config).expect("Format 2 failed");
    assert_eq!(Formatted2.trim(), "const elem = <Component prop = { value } />;");

    // Single-line component with multiple props
    let Input3 = "const elem = <Component a={1} b={2} title=\"Hello\" />;\n";
    let Formatted3 = Source_Formatter::Format(Input3, &Config).expect("Format 3 failed");
    assert_eq!(Formatted3.trim(), "const elem = <Component a = { 1 } b = { 2 } title = \"Hello\" />;");

    // Button with onClick prop
    let Input4 = "const elem = <button onClick={handleClick}>Click Me</button>;\n";
    let Formatted4 = Source_Formatter::Format(Input4, &Config).expect("Format 4 failed");
    assert_eq!(Formatted4.trim(), "const elem = <button onClick = { handleClick }>Click Me</button>;");
}

#[test]
fn Test_JSX_Text_Mode_And_Apostrophe()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Jsx;
    Config.Canvas_Width = 160;

    // Text with apostrophe (L6c)
    let Input1 = "const elem = <p>Don't touch that</p>;\n";
    let Formatted1 = Source_Formatter::Format(Input1, &Config).expect("Format 1 failed");
    assert_eq!(Formatted1.trim(), "const elem = <p>Don't touch that</p>;");

    // Text with embedded expression and trailing apostrophe
    let Input2 = "const elem = <p>It's {name}'s birthday!</p>;\n";
    let Formatted2 = Source_Formatter::Format(Input2, &Config).expect("Format 2 failed");
    assert_eq!(Formatted2.trim(), "const elem = <p>It's { name }'s birthday!</p>;");

    // Nested inline elements
    let Input3 = "const elem = <div><span>Hello World</span></div>;\n";
    let Formatted3 = Source_Formatter::Format(Input3, &Config).expect("Format 3 failed");
    assert_eq!(Formatted3.trim(), "const elem = <div><span>Hello World</span></div>;");

    // Fragment
    let Input4 = "const elem = <><span>Fragment Content</span></>;\n";
    let Formatted4 = Source_Formatter::Format(Input4, &Config).expect("Format 4 failed");
    assert_eq!(Formatted4.trim(), "const elem = <><span>Fragment Content</span></>;");
}

#[test]
fn Test_TSX_Generic_Arrow_Function()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Tsx;
    Config.Canvas_Width = 160;

    // Generic arrow function with trailing comma in TSX
    let Input = "const identity = <T,>(x: T): T => x;\n";
    let Formatted = Source_Formatter::Format(Input, &Config).expect("Format failed");
    assert_eq!(Formatted.trim(), "const identity = <T,>(x: T): T => x;");

    // TSX component with generic prop
    let InputComp = "const elem = <List<string> items={items} />;\n";
    let FormattedComp = Source_Formatter::Format(InputComp, &Config).expect("Format comp failed");
    assert_eq!(FormattedComp.trim(), "const elem = <List<string> items = { items } />;");
}

#[test]
fn Test_JSX_Multiline_HTML_Tree_Indentation()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Tsx;
    Config.Canvas_Width = 160;

    let Input = r#"const App = () => (
    <div>
        <h1>Title</h1>
        <p>
            Paragraph
        </p>
    </div>
);
"#;
    let FormattedPass1 = Source_Formatter::Format(Input, &Config).expect("Pass 1 failed");
    assert!(FormattedPass1.contains("    <div>"), "div must be indented 4 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        <h1>Title</h1>"), "h1 monolith must be indented 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        <p>"), "p opener must be indented 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("            Paragraph"), "text inside p must be indented 12 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        </p>"), "p closer must align with <p> at 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("    </div>"), "div closer must align with <div> at 4 spaces: {}", FormattedPass1);

    // Idempotence
    let FormattedPass2 = Source_Formatter::Format(&FormattedPass1, &Config).expect("Pass 2 failed");
    assert_eq!(FormattedPass1, FormattedPass2, "Multiline JSX formatting must be 100% idempotent");
}

#[test]
fn Test_JSX_Multiline_Props()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Jsx;
    Config.Canvas_Width = 160;

    let Input = r#"const Card = () => (
    <div
        className="card-container"
        id="main-card"
    >
        <Component
            propA="val1"
            propB="val2"
        />
    </div>
);
"#;
    let FormattedPass1 = Source_Formatter::Format(Input, &Config).expect("Pass 1 failed");
    assert!(FormattedPass1.contains("    <div"), "div header must be at 4 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        className = \"card-container\""), "prop must be indented 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        id = \"main-card\""), "prop must be indented 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("    >"), "header > must align with <div at 4 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        <Component"), "Component must be at 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("            propA = \"val1\""), "Component prop must be at 12 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("            propB = \"val2\""), "Component prop must be at 12 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        />"), "Component closer /> must align at 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("    </div>"), "div closer must align at 4 spaces: {}", FormattedPass1);

    // Idempotence
    let FormattedPass2 = Source_Formatter::Format(&FormattedPass1, &Config).expect("Pass 2 failed");
    assert_eq!(FormattedPass1, FormattedPass2, "Multiline JSX with props must be 100% idempotent");
}

#[test]
fn Test_JSX_Embedded_Expression_Indentation()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Tsx;
    Config.Canvas_Width = 160;

    let Input = r#"const List = ({ items }) => (
    <ul>
        {items.map(item => (
            <li key={item.id}>
                {item.text}
            </li>
        ))}
    </ul>
);
"#;
    let FormattedPass1 = Source_Formatter::Format(Input, &Config).expect("Pass 1 failed");
    assert!(FormattedPass1.contains("    <ul>"), "ul must be at 4 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        {"), "expression opener must be at 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("        }"), "expression closer must be at 8 spaces: {}", FormattedPass1);
    assert!(FormattedPass1.contains("    </ul>"), "ul closer must be at 4 spaces: {}", FormattedPass1);

    // Idempotence
    let FormattedPass2 = Source_Formatter::Format(&FormattedPass1, &Config).expect("Pass 2 failed");
    assert_eq!(FormattedPass1, FormattedPass2, "JSX embedded expression must be 100% idempotent");
}

#[test]
fn Test_JSX_Text_Adjacent_Expressions()
{
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Tsx;
    Config.Canvas_Width = 160;

    let Input = r#"const Pagination = () => (
    <span>
        {startItem}-{endItem} of{' '}
    </span>
);
"#;
    let FormattedPass1 = Source_Formatter::Format(Input, &Config).expect("Pass 1 failed");
    assert!(FormattedPass1.contains("{ startItem }-{ endItem }"), "Adjacent hyphen expression must not have extra spaces around hyphen: {}", FormattedPass1);

    // Idempotence
    let FormattedPass2 = Source_Formatter::Format(&FormattedPass1, &Config).expect("Pass 2 failed");
    assert_eq!(FormattedPass1, FormattedPass2, "JSX text adjacent expressions must be 100% idempotent");
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_Dart_Monolith_Function()
{
    let Input = "int Add(int A, int B) { return A + B; }";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), "int Add(int A, int B) { return A + B; }");
}

#[test]
fn Test_Dart_Expanded_Function_Detached_Brace()
{
    let Input = r#"void ProcessData(String Item, int Count) {
    StepOne(Item);
    StepTwo(Count);
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;
    Config.Canvas_Width = 50;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    let Expected = "void ProcessData(String Item, int Count)\n{\n    StepOne(Item);\n    StepTwo(Count);\n}";
    assert_eq!(Formatted.trim(), Expected);
}

#[test]
fn Test_Dart_1_Tuple_Record_Trailing_Comma()
{
    let Input = "final (int,) SingleRecord = (42,);";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("(42,)"), "1-tuple record must retain trailing comma. Got:\n{}", Formatted);
    assert!(Formatted.contains("(int,)"), "1-tuple record type must retain trailing comma. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Empty_Container_Monolith()
{
    let Input = r#"class Point {
    Point(this.x, this.y) {}
    void EmptyMethod() {}
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("{}"), "Empty braces must stay compact '{{}}'. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Dot_Shorthands()
{
    let Input = r#"final Tone = .violetCore;
final Item = const .origin();
final Other = .new();"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains(".violetCore"), "Dot shorthand must remain atomic. Got:\n{}", Formatted);
    assert!(Formatted.contains(".origin()"), "Dot constructor must remain atomic. Got:\n{}", Formatted);
    assert!(Formatted.contains(".new()"), "Dot new must remain atomic. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Cascades()
{
    let Input = r#"final Map<String, dynamic>? Victim = <String, dynamic>{'init': Tag}
    ?..['step1'] = 123
    ..update('init', (x) => x)
    ..clear();"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("?..['step1']"), "Null-aware cascade must format cleanly. Got:\n{}", Formatted);
    assert!(Formatted.contains("..update"), "Cascade must format cleanly. Got:\n{}", Formatted);
    assert!(Formatted.contains("..clear()"), "Cascade must format cleanly. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Multiline_String_Preservation()
{
    let Input = r#"final Text = '''
Line 1
  Line 2 (indented)
Line 3
''';"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("Line 1\n  Line 2 (indented)\nLine 3"), "Multiline string interior whitespace must be preserved verbatim. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Control_Flow_Collections()
{
    let Input = r#"final List<int> Numbers = [
    1,
    for (var i = 0; i < 5; i++) i * 2,
    if (Flag) 99,
    ...?OptionalList,
    ?SingleItem
];"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("for (var i = 0; i < 5; i++) i * 2"), "Collection for must remain without braces. Got:\n{}", Formatted);
    assert!(Formatted.contains("if (Flag) 99"), "Collection if must remain without braces. Got:\n{}", Formatted);
    assert!(Formatted.contains("...?OptionalList"), "Spread must remain intact. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Switch_Expression()
{
    let Input = r#"String Label(int Value) => switch (Value) {
    <= 0 => 'LOW',
    1 || 2 => 'MED',
    var v when v.isOdd => 'ODD',
    _ => 'OTHER'
};"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("switch (Value)"), "Switch expression header. Got:\n{}", Formatted);
    assert!(Formatted.contains("<= 0 => 'LOW'"), "Case arm. Got:\n{}", Formatted);
    assert!(Formatted.contains("var v when v.isOdd => 'ODD'"), "When guard. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Primary_Constructor_And_This_Body()
{
    let Input = r#"sealed class AstPacket<T>(
    final String PacketId,
    final TelemetrySpectrum Spectrum
) {
    this : assert(PacketId.isNotEmpty, 'Packet ID cannot be empty') {
        if (PacketId == 'INVALID') { StateError('Bad packet'); }
    }
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("sealed class AstPacket <T>") || Formatted.contains("sealed class AstPacket<T>"), "Class header. Got:\n{}", Formatted);
    assert!(Formatted.contains("this : assert"), "This initializer. Got:\n{}", Formatted);
}

#[test]
fn Test_Dart_Torture_Test_Slop_And_Idempotence()
{
    let PossiblePaths = ["The Ultimat Failure Of K&R Exposed in Dart.md", "Dart.md",];
    let mut MdContent = None;
    for PathStr in & PossiblePaths
    {
        let P = std::path::Path:: new (env ! ("CARGO_MANIFEST_DIR")).join(PathStr);
        if let Ok(Content) = std::fs::read_to_string( & P)
        { MdContent = Some(Content); break ; }
        if let Ok(Content) = std::fs::read_to_string(PathStr)
        { MdContent = Some(Content); break ; }
    }
    let MdContent = MdContent.expect("Failed to locate Dart torture test reference markdown");
    let Part1Marker = "WARNING: INTENTIONALLY CORRUPTED";

    let MarkerIdx = MdContent.find(Part1Marker).expect("Failed to locate Part 1 marker");
    // Find the opening ```dart right before this marker
    let CodeBlockStart = MdContent[..MarkerIdx].rfind("```dart").expect("Failed to locate opening ```dart");
    let StartIdx = MdContent[CodeBlockStart..].find('\n').unwrap() + CodeBlockStart + 1;

    let EndIdx = MdContent[StartIdx..].find("```").expect("Failed to locate Part 1 closing ```") + StartIdx;

    let TortureSlop = & MdContent[StartIdx..EndIdx];

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;
    Config.Canvas_Width = 160;

    // Pass 1: Format slop
    let FormattedPass1 = Source_Formatter::Format(TortureSlop, & Config)
   .expect("Formatting Dart torture test slop failed on Pass 1");

    // Pass 2: Idempotence check
    let FormattedPass2 = Source_Formatter::Format( & FormattedPass1, & Config)
   .expect("Formatting Dart torture test slop failed on Pass 2");

    if FormattedPass1 != FormattedPass2
    {
        let Lines1: Vec < & str > = FormattedPass1 .lines().collect();
        let Lines2: Vec < & str > = FormattedPass2 .lines().collect();
        for (idx, (l1, l2)) in Lines1 .iter().zip(Lines2 .iter()).enumerate()
        { if l1 != l2 { eprintln ! ("First idempotence divergence at line {}:\nPass 1: {:?}\nPass 2: {:?}", idx + 1, l1, l2); break ; } }
        if Lines1 .len() != Lines2 .len()
        { eprintln ! ("Line count mismatch: Pass 1 has {} lines, Pass 2 has {} lines", Lines1 .len(), Lines2 .len()); }
    }

    assert_eq!(FormattedPass1, FormattedPass2, "Dart formatting must be 100% idempotent");

    // Invariant verifications
    assert!(FormattedPass1 .contains("(42,)"), "1-tuple record must retain comma");
    assert!(FormattedPass1 .contains("(scale,)"), "1-tuple scale record must retain comma");
    assert!(FormattedPass1 .contains(".violetCore"), "Dot shorthand must remain intact");
    assert!(FormattedPass1 .contains(".deepOrangeAlert"), "Dot shorthand must remain intact");
}

#[test]
fn Test_Dart_1_Tuple_Nested_List_Record_Grouping()
{
    let Input = r#"final (List<String>,) MultilineRecordTuple = ([
    'Primary_#7100FF_Configuration_Token',
    'Secondary_#FC6A03_Warning_Threshold_Token',
    'Tertiary_#FF1744_Critical_Failure_Fallback_Token'
],);"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;
    Config.Canvas_Width = 60;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("(["), "Alloyed opening node for nested 1-tuple list record");
    assert!(Formatted.contains("],);") || Formatted.contains("],)"), "Alloyed closing node must retain comma before paren");
}

#[test]
fn Test_Dart_Pure_Symmetry_On_Return_Throw_Yield()
{
    let Input = r#"Map<String, dynamic> GetData() {
    return {
        'status': 'OK',
        'code': 200
    };
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    // In Dart, return has no ASI hazard, so pure LAFS symmetry applies
    assert!(Formatted.contains("return\n    {\n") || Formatted.contains("return\n{\n"), "Pure symmetry on return in Dart without ASI hazard");
}

#[test]
fn Test_Dart_Adjacent_String_Literals()
{
    let Input = r#"final Message = 'Part A ' 'Part B ' "Part C";"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert_eq!(Formatted.trim(), Input);
}

#[test]
fn Test_Dart_Nested_Block_Comments()
{
    let Input = r#"/* Outer comment /* nested comment */ outer again */
void Main() {}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("/* Outer comment /* nested comment */ outer again */"));
}

#[test]
fn Test_Dart_Leading_Underscore_Privacy()
{
    let Input = r#"class _PrivateClass {
    final int _privateField = 42;
    void _privateMethod() {}
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting failed");
    assert!(Formatted.contains("_PrivateClass"));
    assert!(Formatted.contains("_privateField"));
    assert!(Formatted.contains("_privateMethod"));
}

#[test]
fn Test_Dart_Generic_Typedef_Allman_And_Monolith_Function()
{
    let Input = r#"
typedef NightmareCallback
<
    T extends Comparable <T>,
    U extends Map <String, List <(int, { String tag, bool? flag })>>
> = FutureOr <(T, (int,), { U payload, List <T> Function <V extends T>(V input, [String? prefix])? transformer })>
    Function <W extends Object?>
(
    T primary,
    (int,) singleTupleRecord,
    { required U? config, W Function(int, int, { String? label })? interceptor }
);
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Dart;
    Config.Canvas_Width = 160;

    let FormattedPass1 = Source_Formatter::Format(Input, & Config).expect("Formatting failed Pass 1");
    let FormattedPass2 = Source_Formatter::Format( & FormattedPass1, & Config).expect("Formatting failed Pass 2");

    assert_eq!(FormattedPass1, FormattedPass2, "Must be 100% idempotent");
    assert!(FormattedPass1 .contains("typedef NightmareCallback\n<\n"), "Allman-style < for generic typedef");
    assert!(FormattedPass1 .contains("> = FutureOr <("), "Closing > on same line as = FutureOr");
    assert!(FormattedPass1 .contains("    Function <W extends Object?>\n("), "Function <W extends Object?> monolith indented at 4 spaces followed by detached (");
}

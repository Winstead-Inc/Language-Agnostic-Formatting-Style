#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_Typst_Monolith_Function()
{
    let Input = "#let Add(A, B) = A + B";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst formatting failed");
    assert_eq!(Formatted.trim(), "#let Add(A, B) = A + B");
}

#[test]
fn Test_Typst_Empty_Dict_And_Array()
{
    let Input = "#let Data = (:\n)\n#let Arr = (\n)";
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 120;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst formatting failed");
    assert!(Formatted.contains("(:)"), "Empty dictionary must format as monolith (:)");
    assert!(Formatted.contains("()"), "Empty array must format as monolith ()");
}

#[test]
fn Test_Typst_Hyphenated_Identifiers()
{
    let Input = r#"#let Func(
  content-base: "/",
  extra-chapters: none,
) = {
  assert(content-base.starts-with("/"))
}"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 60;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst formatting failed");
    assert!(Formatted.contains("content-base"), "Hyphenated identifier must be preserved");
    assert!(Formatted.contains("extra-chapters"), "Hyphenated identifier must be preserved");
    assert!(Formatted.contains("starts-with"), "Hyphenated method name must be preserved");
}

#[test]
fn Test_Typst_Docs_Lib_File_Formatting()
{
    let Input = r#"#import "components/index.typ" as components

// Renders the documentation.
#let docs(
  // The base path for the documentation's content files in the bundle. Only
  // applies to the website output format.
  //
  // Should be an absolute path like `/docs/` and *must* start and end with a
  // forward slash.
  content-base: "/",

  // The base path for the documentation's asset files in the bundle. Only
  // applies to the website output format.
  //
  // Should be an absolute path like `/assets/` and *must* start and end with a
  // forward slash.
  asset-base: "/assets/",

  // Additional content that is appended to the end of the main docs content.
  extra-chapters: none,

  // Dictionary of external insertions that the chapters can use if present.
  insertions: (:),
) = {
  assert(content-base.starts-with("/") and content-base.ends-with("/"))
  assert(asset-base.starts-with("/") and asset-base.ends-with("/"))

  // The config element and the `docs` function can be merged once we have
  // custom types.
  set stdx.config(
    content-base: content-base,
    asset-base: asset-base,
    insertions: insertions,
  )

  show: components.styling

  context if target() == "bundle" {
    include "assets/index.typ"
  }

  include "content/index.typ"
  extra-chapters
}"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst lib.typ formatting failed");
    assert!( ! Formatted.is_empty());
    // Format a second time to ensure idempotency
    let ReFormatted = Source_Formatter::Format( & Formatted, & Config).expect("Typst re-formatting failed");
    assert_eq!(Formatted, ReFormatted, "Formatting must be idempotent");
}

#[test]
fn Test_Typst_Preserve_Empty_Lines()
{
    let Input = r#"#let A = 1

#let B = 2


#let C = 3"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;
    Config.Preserve_Empty_Lines = true;
    Config.Max_Blank_Lines = 1;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst empty line preservation failed");
    assert!(Formatted.contains("#let A = 1\n\n#let B = 2\n\n#let C = 3"));
}

#[test]
fn Test_Typst_Nested_Block_Comments()
{
    let Input = r#"/* outer /* inner */ comment */
#let Value = 42"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst nested block comments failed");
    assert!(Formatted.contains("/* outer /* inner */ comment */"));
    assert!(Formatted.contains("#let Value = 42"));
}

#[test]
fn Test_Typst_Labels()
{
    let Input = r#"#heading[Introduction] <sec:intro>
Reference to @sec:intro."#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Typst;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Typst labels failed");
    assert!(Formatted.contains("<sec:intro>"));
}

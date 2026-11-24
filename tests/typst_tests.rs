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

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst labels failed");
    assert!(Formatted.contains("<sec:intro>"));
}

#[test]
fn Test_Typst_Content_Block_Single_Parens()
{
    let Input = r#"#{
  let a = [(]
  let b = [)]
  let c = [{]
  let d = [}]
}"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst single parens in content blocks failed");
    assert!(Formatted.contains("[(]"), "Content block [(] must be preserved without mismatch error");
    assert!(Formatted.contains("[)]"), "Content block [)] must be preserved without mismatch error");
}

#[test]
fn Test_Typst_Prose_Parens_Not_Code_Delimiters()
{
    let Input = r#"= Introduction <intro>
This is standard prose text (with normal parentheses) that should not be parsed as code.
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst prose parens failed");
    assert!(Formatted.contains("(with normal parentheses)"));
}

#[test]
fn Test_Typst_Category_Pair_Stability()
{
    let Path = r"e:\Typst\docs\components\category.typ";
    if let Ok(Content) = std::fs::read_to_string(Path)
    {
        let mut Config = Config_Settings::default();
        Config.Language = Source_Language::Typst;
        Config.Canvas_Width = 160;

        let Formatted = Source_Formatter::Format(&Content, &Config).expect("Category.typ formatting failed");
        assert!(!Formatted.is_empty());
        let ReFormatted = Source_Formatter::Format(&Formatted, &Config).expect("Category.typ idempotency check failed");
        assert_eq!(Formatted, ReFormatted, "Formatting must be idempotent");
    }
}

#[test]
fn Test_Typst_Method_Chain_Indentation()
{
    let Input = r#"#{
  let skip = {
    groups
      .map(g => g.definitions.values())
      .filter(v => type(v) != function)
      .flatten()
    sub-categories.values()
  }
}"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 60;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst method chain failed");
    assert!(!Formatted.contains("\n.map"), "Chained method .map must not be at column 0");
    assert!(!Formatted.contains("\n.filter"), "Chained method .filter must not be at column 0");
    assert!(!Formatted.contains("\n.flatten"), "Chained method .flatten must not be at column 0");
    assert!(Formatted.contains("        groups\n            .map(g => g.definitions.values())\n            .filter(v => type(v) != function)\n            .flatten()\n        sub-categories.values()"));
}

#[test]
fn Test_Typst_Compound_Assignment_Operators()
{
    let Input = r#"
#let Update(items, heading, scope) = {
    items += query(heading.where(outlined: true).within(scope)).map(m => (dest: m.location(), level: m.level, body: m.body))
    count -= 1
    total *= 2
    factor /= 4
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst formatting failed");
    assert!(!Formatted.contains("+ ="), "Operator '+=' must never be detached into '+ ='");
    assert!(!Formatted.contains("- ="), "Operator '-=' must never be detached into '- ='");
    assert!(!Formatted.contains("* ="), "Operator '*=' must never be detached into '* ='");
    assert!(!Formatted.contains("/ ="), "Operator '/=' must never be detached into '/ ='");
    assert!(Formatted.contains("items += query("));
    assert!(Formatted.contains("count -= 1"));
    assert!(Formatted.contains("total *= 2"));
    assert!(Formatted.contains("factor /= 4"));
}

#[test]
fn Test_Typst_Markup_Prose_And_Punctuation_Spacing()
{
    let Input = r#"
#let section() = {
    [Requires the #raw(info.feature) feature]
    [ on #parent]
    with-tooltip[#heading-kinds #if nested { [ on #raw(parent)] }]
    [Functions and types can have associated #sentence-kinds. These are accessed by specifying the function or type, followed by a period, and then the #sentence-kind ' s name.]
    [That's the autocomplete panel. It can be very useful while you are writing your document: You can apply its suggestions by hitting the Return key or navigate to the desired completion with the arrow keys. The panel can be dismissed by hitting the Escape key and opened again by typing `#` or hitting #kbd("Ctrl") + #kbd("Space"). Use the autocomplete panel to discover the right arguments for functions. Most suggestions come with a small description of what they do.]
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 300;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst markup spacing failed");
    assert!(Formatted.contains("[Requires the #raw(info.feature) feature]"));
    assert!(Formatted.contains("[ on #parent]"));
    assert!(Formatted.contains("with-tooltip[#heading-kinds #if nested { [ on #raw(parent)] }]"));
    assert!(Formatted.contains("associated #sentence-kinds. These are"));
    assert!(Formatted.contains("and then the #sentence-kind's name."));
    assert!(Formatted.contains("That's the autocomplete panel. It can"));
    assert!(Formatted.contains("or hitting #kbd(\"Ctrl\") + #kbd(\"Space\"). Use"));
}

#[test]
fn Test_Typst_Contractions_And_Prose_Punctuation_Repair()
{
    let Input = r#"
Let ' s add a few more styles to our document.We want larger margins and a serif font.For the purposes of the example, we ' ll also set another page size.
Your teaching assistant does not yet know that you are using a new typesetting system, and you want your report to fit in with the other student ' s submissions.In this chapter, we will see how to format your report using Typst ' s styling system.
Wait, shouldn ' t all arguments of a function be specified within parentheses ?
As seen above, that works.With the @ text function, we can adjust the font.
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 300;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst contraction repair failed");
    assert!(Formatted.contains("Let's add a few more styles to our document. We want larger margins and a serif font. For the purposes of the example, we'll also set another page size."));
    assert!(Formatted.contains("student's submissions. In this chapter, we will see how to format your report using Typst's styling system."));
    assert!(Formatted.contains("Wait, shouldn't all arguments of a function be specified within parentheses?"));
    assert!(Formatted.contains("that works. With the @text function"));
}

#[test]
fn Test_Typst_Tutorial_Formatting_Prose_Repair()
{
    let Input = r#"
So far, you have written a report with some text, a few equations and images.However, it still looks very plain.Your teaching assistant does not yet know that you are using a new typesetting system, and you want your report to fit in with the other student's submissions.In this chapter, we will see how to format your report using Typst ' s styling system.

= Set rules <set-rules>
As we have seen in the previous chapter, Typst has functions that _insert_ content(e.g.the @ image function) and others that _manipulate_ content that they received as arguments(e.g.the @ align function).The first impulse you might have when you want, for example, to change the font, could be to look for a function that does that and wrap the complete document in it.

Wait, shouldn ' t all arguments of a function be specified within parentheses ? Why is there a second set of square brackets with content _after_ the parentheses ?

#docs-figure("2-formatting-autocomplete.png", alt: "Autocomplete panel", shadow: false ,)

As seen above, that works.With the @ text function, we can adjust the font for all text within it.However, wrapping the document in countless functions and applying styles selectively and in-situ can quickly become cumbersome.

Fortunately, Typst has a more elegant solution.With _set rules, _ you can apply style properties to all occurrences of some kind of content.You write a set rule by entering the `{set}` keyword, followed by the name of the function whose properties you want to set, and a list of arguments in parentheses.

Here, we set the font size to `{10pt}` and font family to `{"New Computer Modern"}`.The Typst app comes with many fonts that you can try for your document.

Typst internally calls the `heading` function every time you write `[= Conclusion]`.In fact, the function call `[#heading[Conclusion]]` is equivalent to the heading markup above.

Here, we called that argument `name`.We can now use the `name` variable in the function ' s body.

The panel can be dismissed by hitting the Escape key and opened again by typing `#` or hitting#kbd("Ctrl") + #kbd("Space").Use the autocomplete panel to discover the right arguments for functions.Most suggestions come with a small description of what they do.

Vertical and horizontal alignments can be combined with the `{+}` operator to yield a 2 D alignment.

We specified the string `{"1."}` as the numbering parameter.This tells Typst to number the headings with arabic numerals and to put a dot between the number of each level.We can also use @ numbering[letters, roman numerals, and symbols] for our headings:

Instead, you could maybe @ function: defining-functions[define a custom function] that always yields the logo with its image.However, there is an even easier way:

This can be observed in parameter lists, function definitions, and @ reference: scripting[code blocks].

You have handed in your report.Your supervisor was so happy with it that they want to adapt it into a conference paper !
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Typst;
    Config.Canvas_Width = 300;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Typst tutorial formatting prose repair failed");
    println!("--- FORMATTED OUTPUT ---\n{}\n--- END OUTPUT ---", Formatted);
    assert!(Formatted.contains("images. However, it still looks very plain. Your teaching assistant"));
    assert!(Formatted.contains("other student's submissions. In this chapter"));
    assert!(Formatted.contains("using Typst's styling system."));
    assert!(Formatted.contains("content (e.g. the @image function) and others that _manipulate_ content that they received as arguments (e.g. the @align function). The first impulse"));
    assert!(Formatted.contains("Wait, shouldn't all arguments of a function be specified within parentheses? Why is there"));
    assert!(Formatted.contains("content _after_ the parentheses?"));
    assert!(Formatted.contains("shadow: false,)"));
    assert!(Formatted.contains("`{\"New Computer Modern\"}`. The Typst app"));
    assert!(Formatted.contains("`[= Conclusion]`. In fact"));
    assert!(Formatted.contains("`name`. We can now use"));
    assert!(Formatted.contains("that works. With the @text function, we can adjust the font for all text within it. However, wrapping"));
    assert!(Formatted.contains("solution. With _set rules,_ you can apply style properties to all occurrences of some kind of content. You write a set rule"));
    assert!(Formatted.contains("or hitting #kbd(\"Ctrl\") + #kbd(\"Space\"). Use the autocomplete panel to discover the right arguments for functions. Most suggestions"));
    assert!(Formatted.contains("to yield a 2D alignment."));
    assert!(Formatted.contains("numbering parameter. This tells Typst"));
    assert!(Formatted.contains("use @numbering[letters, roman numerals, and symbols] for our headings:"));
    assert!(Formatted.contains("maybe @function:defining-functions[define a custom function] that always yields the logo with its image. However, there is an even easier way:"));
    assert!(Formatted.contains("and @reference:scripting[code blocks]."));
    assert!(Formatted.contains("report. Your supervisor was so happy with it that they want to adapt it into a conference paper!"));
}

#[test]
fn Test_Typst_Tutorial_Template_File()
{
    let Path = "E:\\Typst\\docs\\content\\tutorial\\4-template.typ";
    if let Ok(Content) = std::fs::read_to_string(Path)
    {
        let mut Config = Config_Settings::default();
        Config.Language = Source_Language::Typst;
        Config.Canvas_Width = 160;

        let Formatted = Source_Formatter::Format(&Content, &Config).expect("Formatting 4-template.typ failed");
        assert!(!Formatted.is_empty());
    }
}

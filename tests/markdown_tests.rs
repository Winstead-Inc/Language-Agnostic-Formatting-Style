#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style::formatter::Source_Formatter;
use Language_Agnostic_Formatting_Style::types::{Config_Settings, Source_Language};

#[test]
fn Test_Markdown_Rust_Snippet_Formatting()
{
    let Input = r#"# Rust Example

Here is a function in Rust with K&R slop:

```rust
fn calculate_total(items: &[i32]) -> i32 {
    let mut sum = 0;
    for &item in items {
        if item > 0 {
            sum += item;
        }
    }
    return sum;
}
```

That concludes the example.
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Markdown formatting failed");

    assert!(Formatted.contains("fn calculate_total(items: & [i32]) -> i32\n{\n"), "Rust function must have detached opening brace");
    assert!(Formatted.starts_with("# Rust Example\n\nHere is a function in Rust with K&R slop:"));
    assert!(Formatted.ends_with("```\n\nThat concludes the example.\n"));
}

#[test]
fn Test_Markdown_TypeScript_Snippet_Formatting()
{
    let Input = r#"# TypeScript Integration

```typescript
export function ProcessOrder(orderId: string): OrderResult {
    if (!orderId) {
        throw new Error("Invalid order");
    }
    return {
        id: orderId,
        status: "completed"
    };
}
```
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");

    assert!(Formatted.contains("export function ProcessOrder(orderId: string): OrderResult\n{"), "TS function formatted via JS_TS_Formatter");
}

#[test]
fn Test_Markdown_Blockquote_Indented_Snippet()
{
    let Input = r#"> [!NOTE]
> Here is a quick code sample:
> ```ts
> function Greet(name: string): string {
>     return `Hello, ${name}!`;
> }
> ```
> And a closing thought.
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");

    assert!(Formatted.contains("> function Greet(name: string): string\n> { return `Hello, ${name}!`; }\n"), "Blockquote prefix must be preserved across formatted lines");
    assert!(Formatted.contains("> ```\n> And a closing thought."));
}

#[test]
fn Test_Markdown_List_Indented_Snippet()
{
    let Input = r#"1. Configure your handler:
  ```js
  const HandleRequest = (req, res) => {
      if (req.method === 'POST') {
          res.end('OK');
      }
  };
  ```
2. Run your server.
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");

    assert!(Formatted.contains("  const HandleRequest = (req, res) => { if (req.method === 'POST') { res.end('OK'); } };\n"), "List indent must be preserved on formatted snippet");
    assert!(Formatted.contains("  ```\n2. Run your server."));
}

#[test]
fn Test_Markdown_Non_Code_Fences_Preserved()
{
    let Input = r#"# Diagram & Output

```text
  +---------+
  | Payload |
  +---------+
```

```mermaid
graph TD
    A[Start] --> B[Process]
```

```diff
- old line {
+ new line {
```
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert_eq!(Formatted, Input, "Non-code fences (text, mermaid, diff) must remain bit-for-bit identical");
}

#[test]
fn Test_Markdown_Prose_And_Headings_Untouched()
{
    let Input = r#"# Master Architecture Document

This is an unbroken semantic paragraph that must flow freely across the line without any artificial eighty-column hard line wrapping injected by archaic terminal limitations. It tests the global markdown preservation rule.

| Metric | Target | Status |
| :--- | :--- | :--- |
| Throughput | 100k req/s | Exceeded |
| Latency | < 5ms | Nominal |

- Bullet one with long explanatory text describing architectural invariants.
- Bullet two.
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert_eq!(Formatted, Input, "Markdown prose, tables, and lists must remain bit-for-bit identical");
}

#[test]
fn Test_Markdown_Idempotence()
{
    let Input = r#"# Idempotency Test

```cpp
void Execute() {
    if (Active) {
        Run();
    }
}
```
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Pass1 = Source_Formatter::Format(Input, &Config).expect("Pass 1 failed");
    let Pass2 = Source_Formatter::Format(&Pass1, &Config).expect("Pass 2 failed");

    assert_eq!(Pass1, Pass2, "Markdown formatting must be strictly idempotent");
}

#[test]
fn Test_Markdown_Incomplete_Snippet_Fallback()
{
    let Input = r#"# Snippet with Unclosed Delimiter

```rust
// Truly unclosed delimiter that fails matcher
fn broken() {
```
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Fallback must not panic or error");
    assert_eq!(Formatted, Input, "Incomplete snippets that fail safety verifier must be preserved verbatim");
}

#[test]
fn Test_Markdown_Four_Backtick_Fence()
{
    let Input = r#"# Nested Markdown Blocks

````markdown
```js
console.log("inner");
```
````
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Nested markdown fence failed");
    assert_eq!(Formatted, Input, "Inner 3-backtick fence inside 4-backtick fence must not prematurely close");
}

#[test]
fn Test_Markdown_Tilde_Fences()
{
    let Input = r#"# Tildes

~~~go
func main() {
    println("hello")
}
~~~
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Tilde formatting failed");
    assert!(Formatted.contains("func main()"), "Go function inside tilde fence must be processed");
}

#[test]
fn Test_Markdown_JSON_Formatting()
{
    let Input = r#"```json
{
  "name": "lafs",
  "config": {
    "width": 160
  }
}
```
"#;

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("JSON formatting failed");
    assert!(Formatted.contains(r#"{ "name": "lafs", "config": { "width": 160 } }"#) || Formatted.contains(r#""name": "lafs""#));
}

#[test]
fn Test_Markdown_Paragraph_Unwrapping()
{
    let Input = "This is a paragraph that was artificially\nsplit across multiple lines to fit an archaic\n80-column VT100 terminal constraint.\n\nAnd here is a second paragraph that was also\narbitrarily broken up into pieces.\n";

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;
    Config.Unwrap_Markdown_Paragraphs = true;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Markdown paragraph unwrapping failed");

    let Expected = "This is a paragraph that was artificially split across multiple lines to fit an archaic 80-column VT100 terminal constraint.\n\nAnd here is a second paragraph that was also arbitrarily broken up into pieces.\n";
    assert_eq!(Formatted, Expected);
}

#[test]
fn Test_Markdown_Intentional_Hard_Breaks_Preserved()
{
    // In Markdown, two spaces at the end of a line or a trailing backslash signal an intentional hard break.
    let Input = "Line one with two spaces  \nLine two after break.\n\nLine with backslash\\\nLine after backslash.\n";

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;
    Config.Unwrap_Markdown_Paragraphs = true;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Intentional hard breaks preservation failed");

    assert!(Formatted.contains("Line one with two spaces  \nLine two after break."));
    assert!(Formatted.contains("Line with backslash\\\nLine after backslash."));
}

#[test]
fn Test_Markdown_List_And_Blockquote_Unwrapping()
{
    let Input = "- First item that continues\n  on the next line.\n- Second item.\n\n> This is a blockquote quote\n> that wraps awkwardly across lines.\n";

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;
    Config.Unwrap_Markdown_Paragraphs = true;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("List and blockquote unwrapping failed");

    assert!(Formatted.contains("- First item that continues on the next line."));
    assert!(Formatted.contains("- Second item."));
    assert!(Formatted.contains("> This is a blockquote quote that wraps awkwardly across lines."));
}

#[test]
fn Test_Markdown_Code_Fences_Untouched_During_Unwrap()
{
    let Input = "Here is some text that wraps\nacross two lines.\n\n```rust\nfn Foo()\n{\n    let X = 1;\n    let Y = 2;\n    let Z = 3;\n    let W = 4;\n    let V = 5;\n}\n```\n\nClosing paragraph text that also\nwraps across lines.\n";

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Markdown;
    Config.Canvas_Width = 160;
    Config.Unwrap_Markdown_Paragraphs = true;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Unwrap with code fences failed");

    assert!(Formatted.contains("Here is some text that wraps across two lines."));
    assert!(Formatted.contains("Closing paragraph text that also wraps across lines."));
    assert!(Formatted.contains("```rust\nfn Foo()\n{\n    let X = 1;\n    let Y = 2;\n    let Z = 3;\n    let W = 4;\n    let V = 5;\n}\n```"));
}

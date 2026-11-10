#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style::formatter::Source_Formatter;
use Language_Agnostic_Formatting_Style::types:: { Config_Settings, Source_Language };

#[test]
fn Test_HTML_Monolith_And_Expanded_Tags()
{
    let Input = r#"<!DOCTYPE html>
<html>
<head>
<title>LAFS Web App</title>
<meta charset="UTF-8">
</head>
<body>
<div class="main-container">
<h1>Welcome to LAFS</h1>
<p>Clean structural architecture.</p>
</div>
</body>
</html>"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Html;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("HTML formatting failed");

    // Monolith verification
    assert!(Formatted.contains("<title>LAFS Web App</title>"), "Short tag must be monolith: {}", Formatted);
    assert!(Formatted.contains("<h1>Welcome to LAFS</h1>"), "Header must be monolith: {}", Formatted);
    assert!(Formatted.contains("<p>Clean structural architecture.</p>"), "Paragraph must be monolith: {}", Formatted);

    // Expanded symmetry verification
    assert!(Formatted.contains("    <div class=\"main-container\">\n"), "Div opening tag must be indented: {}", Formatted);
    assert!(Formatted.contains("    </div>\n"), "Div closing tag must align with opening tag: {}", Formatted);
}

#[test]
fn Test_HTML_Idempotence()
{
    let Input = r#"<!DOCTYPE html>
<html>
<head>
    <title>Idempotence Test</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <header class="app-header">
        <h1>Dashboard</h1>
    </header>
    <main>
        <section class="card">
            <h2>Overview</h2>
            <p>System status: Operational.</p>
        </section>
    </main>
</body>
</html>"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Html;

    let Pass1 = Source_Formatter::Format(Input, & Config).expect("Pass 1 failed");
    let Pass2 = Source_Formatter::Format( & Pass1, & Config).expect("Pass 2 failed");

    assert_eq!(Pass1, Pass2, "HTML formatting must be 100% idempotent");
}

#[test]
fn Test_CSS_Formatting_And_Colors()
{
    let Input = r#".dashboard-card {
    display: flex;
    flex-direction: column;
    background-color: #7100ff;
    border-color: #fc6a03;
    color: #ffffff;
}"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Css;
    Config.Canvas_Width = 40;
    // Force expanded symmetry

    let Formatted = Source_Formatter::Format(Input, & Config).expect("CSS formatting failed");

    // Uppercase hex color verification
    assert!(Formatted.contains("#7100FF"), "Hex colors must be uppercase, got:\n{}", Formatted);
    assert!(Formatted.contains("#FC6A03"), "Hex colors must be uppercase, got:\n{}", Formatted);
    assert!(Formatted.contains("#FFFFFF"), "Hex colors must be uppercase, got:\n{}", Formatted);

    // Detached delimiter symmetry verification
    assert!(Formatted.contains(".dashboard-card\n{\n"), "CSS rule block opening brace must detach, got:\n{}", Formatted);
    assert!(Formatted.contains("\n}\n"), "CSS rule block closing brace must detach on own line, got:\n{}", Formatted);
}

#[test]
fn Test_CSS_Media_Query_Nesting()
{
    let Input = r#"@media (max-width: 1024px) {
    .sidebar {
        display: none;
    }
    .content {
        width: 100%;
    }
}"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Css;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("CSS media query failed");

    assert!(Formatted.contains("@media (max-width: 1024px)\n{\n"), "At-rule block must detach brace, got:\n{}", Formatted);
    assert!(Formatted.contains("    .sidebar\n    {\n"), "Nested rule must be indented, got:\n{}", Formatted);
    assert!(Formatted.contains("        display: none;\n"), "Declaration must be double indented, got:\n{}", Formatted);
}

#[test]
fn Test_CSS_Idempotence()
{
    let Input = r#".container
{
    margin: 0;
    padding: 20px;
    background-color: #7100FF;
}

@media (max-width: 768px)
{
    .container
    {
        padding: 10px;
    }
}
"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Css;

    let Pass1 = Source_Formatter::Format(Input, & Config).expect("CSS pass 1 failed");
    let Pass2 = Source_Formatter::Format( & Pass1, & Config).expect("CSS pass 2 failed");

    assert_eq!(Pass1, Pass2, "CSS formatting must be 100% idempotent");
}

#[test]
fn Test_Polyglot_HTML_With_Embedded_Script_And_Style()
{
    let Input = r#"<!DOCTYPE html>
<html>
<head>
    <title>Polyglot Fullstack Test</title>
    <style>
.hero-box {
    margin: 0;
    background-color: #7100ff;
}
    </style>
</head>
<body>
    <div id="root">
        <h1>Telemetry Console</h1>
    </div>
    <script>
function startTelemetry() {
    const status = "ONLINE_#00E676";
    console.log(status);
}
    </script>
</body>
</html>"#;

    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Html;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Polyglot formatting failed");

    // CSS inside <style> check
    assert!(Formatted.contains("#7100FF"), "Embedded CSS must format colors uppercase: {}", Formatted);
    assert!(Formatted.contains("    <style>\n"), "Style tag must be indented: {}", Formatted);
    assert!(Formatted.contains("    </style>\n"), "Style close tag must align with open tag: {}", Formatted);

    // JS inside <script> check
    assert!(Formatted.contains("    <script>\n"), "Script tag must be indented: {}", Formatted);
    assert!(Formatted.contains("    </script>\n"), "Script close tag must align with open tag: {}", Formatted);
    assert!(Formatted.contains("        function startTelemetry()"), "Inner JS function must be indented inside script tag: {}", Formatted);

    // Idempotence check on polyglot
    let Pass2 = Source_Formatter::Format( & Formatted, & Config).expect("Polyglot pass 2 failed");
    assert_eq!(Formatted, Pass2, "Polyglot formatting must be 100% idempotent");
}

#[test]
fn Test_Polyglot_Automatic_Detection()
{
    // Pass without specifying language (Generic)
    let HtmlInput = "<!DOCTYPE html><html><head><title>Auto</title></head><body><p>Test</p></body></html>";
    let Config = Config_Settings:: default();

    let FormattedHtml = Source_Formatter::Format(HtmlInput, & Config).expect("Auto HTML failed");
    assert!(FormattedHtml.contains("<title>Auto</title>"), "Auto-detected HTML formatted properly");

    let CssInput = ".hero { margin: 0; padding: 10px; background-color: #7100ff; }";
    let FormattedCss = Source_Formatter::Format(CssInput, & Config).expect("Auto CSS failed");
    assert!(FormattedCss.contains("#7100FF"), "Auto-detected CSS formatted properly");
}

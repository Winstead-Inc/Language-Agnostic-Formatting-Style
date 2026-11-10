#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs:: { Config_Settings, Source_Formatter, Source_Language };

#[test]
fn Test_Toml_Monolith_Arrays_And_Inline_Tables()
{
    let Input = r#"[rust]
incremental = true
download-rustc = "if-unchanged"
crate-type = ["cdylib", "rlib"]

[build]
test-stage = 2
doc-stage = 2
compiler-docs = true
tidy-extra-checks = "auto:js,auto:py,auto:cpp,auto:spellcheck"
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting TOML failed");

    // Verify bare keys with hyphens are preserved without spaces
    assert!(Formatted.contains("download-rustc = \"if-unchanged\""),"download-rustc must not have spaces around hyphen, got:\n{}", Formatted);
    assert!(Formatted.contains("crate-type = [\"cdylib\", \"rlib\"]"),"crate-type monolith array must stay inline, got:\n{}", Formatted);
    assert!(Formatted.contains("test-stage = 2"),"test-stage must stay intact, got:\n{}", Formatted);
    assert!(Formatted.contains("tidy-extra-checks = \"auto:js,auto:py,auto:cpp,auto:spellcheck\""),"tidy-extra-checks must stay intact, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Expanded_Arrays_Allman_Style()
{
    let Input = r#"submodules = [
    "src/doc/book",
    "src/doc/nomicon",
    "src/doc/reference",
    "src/doc/rust-by-example",
]
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;
    Config.Canvas_Width = 80;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting TOML array failed");

    // Opening [ must detach below = on its own line
    assert!(Formatted.contains("submodules =\n[\n"),"Opening [ must sit on its own line below =, got:\n{}", Formatted);
    // Closing ] must sit on its own line
    assert!(Formatted.contains("\n]\n"),"Closing ] must sit on its own line, got:\n{}", Formatted);
    // Children must be indented
    assert!(Formatted.contains("    \"src/doc/book\","),"Children must be indented with 4 spaces, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Alloyed_Container_Grouping()
{
    let Input = r#"target-overrides = [{
    triple = "x86_64-unknown-linux-gnu",
    llvm-config = "/usr/bin/llvm-config",
    sanitizers = true,
    profiler = true,
}]
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;
    Config.Canvas_Width = 80;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting alloyed TOML failed");

    // Opening compound delimiter [{ must sit on its own line below =
    assert!(Formatted.contains("target-overrides =\n[{\n"),"Opening [{{ must sit on its own line below =, got:\n{}", Formatted);
    // Closing compound delimiter }] must sit on its own line
    assert!(Formatted.contains("\n}]\n"),"Closing }}] must sit on its own line, got:\n{}", Formatted);
    // Children must be indented
    assert!(Formatted.contains("    triple = \"x86_64-unknown-linux-gnu\","),"Fields must be indented, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Expanded_Inline_Tables()
{
    let Input = r#"server = {
    host = "127.0.0.1",
    port = 8080,
    tls = true,
}
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;
    Config.Canvas_Width = 40;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting expanded inline table failed");

    // Opening { must sit on its own line below =
    assert!(Formatted.contains("server =\n{\n"),"Opening {{ must sit on its own line below =, got:\n{}", Formatted);
    // Closing } must sit on its own line
    assert!(Formatted.contains("\n}\n"),"Closing }} must sit on its own line, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Literal_Strings_And_Comments()
{
    let Input = r#"# Top level comment
regex = '<\i\c*\s*>'
winpath = 'C:\Users\nodejs\templates'
multiline-literal = '''
First line
Second line
'''
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting literal strings failed");

    assert!(Formatted.contains("# Top level comment\n"),"Comment must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("regex = '<\\i\\c*\\s*>'"),"Single quoted regex string must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("winpath = 'C:\\Users\\nodejs\\templates'"),"Single quoted winpath string must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("multiline-literal = '''"),"Multiline literal string must be preserved, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Table_Arrays_And_Headers()
{
    let Input = r#"[package]
name = "my-crate"
version = "0.1.0"

[[bin]]
name = "my-bin"
path = "src/main.rs"

[[bin]]
name = "other-bin"
path = "src/other.rs"
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting table headers failed");

    assert!(Formatted.contains("[package]\n"),"Table header [package] must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("[[bin]]\n"),"Table array header [[bin]] must be preserved, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Dates_Times_And_Numbers()
{
    let Input = r#"[spec]
date1 = 1979-05-27
date2 = 1979-05-27T07:32:00Z
date3 = 1979-05-27T00:32:00-07:00
time1 = 07:32:00
int_pos = +99
int_neg = -17
float_neg = -0.01
float_exp = 5e+22
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting dates and numbers failed");

    assert!(Formatted.contains("date1 = 1979-05-27"),"date1 must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("date2 = 1979-05-27T07:32:00Z"),"date2 must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("date3 = 1979-05-27T00:32:00-07:00"),"date3 must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("time1 = 07:32:00"),"time1 must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("int_pos = +99"),"int_pos must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("int_neg = -17"),"int_neg must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("float_neg = -0.01"),"float_neg must be preserved, got:\n{}", Formatted);
    assert!(Formatted.contains("float_exp = 5e+22"),"float_exp must be preserved, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Comments_Inside_Array_Forces_Expansion()
{
    let Input = r#"items = [
    # First item
    "alpha",
    # Second item
    "beta",
]
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting array with comments failed");

    // Comments force expansion
    assert!(Formatted.contains("items =\n[\n"),"Array with comments must expand with detached [, got:\n{}", Formatted);
    assert!(Formatted.contains("# First item\n"),"Comment must be preserved, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Toml_Real_Cargo_Manifest()
{
    let Input = r#"[package]
name = "my-library"
version = "0.1.0"
edition = "2021"
rust-version = "1.80"

[dependencies]
serde = { version = "1.0", default-features = false }
anyhow = "1.0"

[features]
default = ["std"]
std = []
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::Toml;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting Cargo manifest failed");

    assert!(Formatted.contains("rust-version = \"1.80\""),"rust-version must stay intact, got:\n{}", Formatted);
    assert!(Formatted.contains("serde = { version = \"1.0\", default-features = false }"),"serde monolith inline table must stay intact, got:\n{}", Formatted);
    assert!(Formatted.contains("default = [\"std\"]"),"default monolith array must stay intact, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Cargo_Lock_Pure_Monolith()
{
    let Input = r#"version = 4

[[package]]
name = "demo"
version = "0.1.0"
dependencies = [
    "chrono",
]

[[package]]
name = "other"
version = "0.2.0"
dependencies = [
    "find-msvc-tools",
    "shlex",
]
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::CargoLock;
    Config.Canvas_Width = 160;

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting Cargo.lock failed");

    // Pure monoliths for short dependency arrays
    assert!(Formatted.contains("dependencies = [\"chrono\",]"),"Short dependency array must be a Pure Monolith, got:\n{}", Formatted);
    assert!(Formatted.contains("dependencies = [\"find-msvc-tools\", \"shlex\",]"),"Multiple dependency array must be a Pure Monolith, got:\n{}", Formatted);

    // Must NOT contain detached Allman
    assert!( ! Formatted.contains("dependencies =\n["),"Cargo.lock must NEVER have detached [ after =, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#[test]
fn Test_Cargo_Lock_Column_Anchored_Hybrid()
{
    let Input = r#"version = 4

[[package]]
name = "demo"
version = "0.1.0"
dependencies = [
    "android_system_properties",
    "core-foundation-sys",
    "iana-time-zone-haiku",
    "js-sys",
    "log",
    "wasm-bindgen",
    "windows-core",
]
"#;
    let mut Config = Config_Settings:: default();
    Config.Language = Source_Language::CargoLock;
    Config.Canvas_Width = 80;
    // Force expansion

    let Formatted = Source_Formatter::Format(Input, & Config).expect("Formatting Cargo.lock failed");

    // Must open inline on '=' line
    assert!(Formatted.contains("dependencies = [\n"),"Opening [ must sit inline after =, got:\n{}", Formatted);

    // Column-anchored hybrid: closing ] must align at column 15 (matching [)
    let ExpectedClose = format!("{:15}]\n", "");
    assert!(Formatted.contains( & ExpectedClose),"Closing ] must align at column 15 matching [, got:\n{}", Formatted);

    // Children must be indented +4 from column 15 (column 19)
    let ExpectedChild = format!("{:19}\"android_system_properties\",", "");
    assert!(Formatted.contains( & ExpectedChild),"Child must be indented at column 19, got:\n{}", Formatted);

    // Must NOT contain detached Allman
    assert!( ! Formatted.contains("dependencies =\n["),"Cargo.lock must NEVER have detached [ after =, got:\n{}", Formatted);

    // Verify idempotence
    let SecondPass = Source_Formatter::Format( & Formatted, & Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Formatting must be idempotent");
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style as lafs;
use lafs::{Config_Settings, Source_Formatter, Source_Language};

#[test]
fn Test_Rust_Cargo_Source_Tree_Battle()
{
    let CargoSrc = std::path::Path::new(r"b:\cargo\src");
    if !CargoSrc.exists()
    {
        eprintln!("Skipping Cargo source tree test (path not found)");
        return;
    }

    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let mut TotalTested = 0;
    let mut Errors = Vec::new();

    fn Walk(Dir: &std::path::Path, Config: &Config_Settings, Total: &mut usize, Errors: &mut Vec<String>)
    {
        if let Ok(Entries) = std::fs::read_dir(Dir)
        {
            for Entry in Entries.flatten()
            {
                let P = Entry.path();
                if P.is_dir()
                {
                    Walk(&P, Config, Total, Errors);
                }
                else if P.extension().and_then(|E| E.to_str()) == Some("rs")
                {
                    *Total += 1;
                    if let Ok(Content) = std::fs::read_to_string(&P)
                    {
                        // Pass 1: Format
                        match Source_Formatter::Format(&Content, Config)
                        {
                            Ok(Pass1) =>
                            {
                                // Pass 2: Idempotence
                                match Source_Formatter::Format(&Pass1, Config)
                                {
                                    Ok(Pass2) =>
                                    {
                                        if Pass1 != Pass2
                                        {
                                            Errors.push(format!("{}: Not idempotent! (Pass1 len: {}, Pass2 len: {})", P.display(), Pass1.len(), Pass2.len()));
                                        }
                                    }
                                    Err(E) =>
                                    {
                                        Errors.push(format!("{}: Pass 2 failed: {}", P.display(), E));
                                    }
                                }
                            }
                            Err(E) =>
                            {
                                Errors.push(format!("{}: Pass 1 failed: {}", P.display(), E));
                            }
                        }
                    }
                }
            }
        }
    }

    Walk(CargoSrc, &Config, &mut TotalTested, &mut Errors);

    eprintln!("Battle tested {} Rust files from Cargo repository.", TotalTested);
    if !Errors.is_empty()
    {
        for ErrMsg in &Errors
        {
            eprintln!("  FAILED: {}", ErrMsg);
        }
        panic!("Rust battle test encountered {} errors out of {} files!", Errors.len(), TotalTested);
    }
}

#[test]
fn Test_Rust_Complex_Lifetimes_And_HRTB()
{
    let Input = r#"pub struct Parser<'a, 'b: 'a, T>
where
    T: Clone + for<'c> Fn(&'c str, &'b [u8]) -> Result<&'a str, CustomError<'c>>,
{
    pub buffer: &'a [u8],
    pub context: &'b Context,
    pub handler: T,
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains("<'a, 'b: 'a, T>"), "Lifetimes must be preserved");
    assert!(Formatted.contains("for < 'c >"), "HRTB lifetime must be preserved");

    let SecondPass = Source_Formatter::Format(&Formatted, &Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Must be idempotent");
}

#[test]
fn Test_Rust_Turbofish_And_Generics()
{
    let Input = r#"fn parse_tree()
{
    let items = stream.collect::<Result<Vec<Option<HashMap<String, Vec<u8>>>>, MyError>>()?;
    let mask = flags >> 2;
    let nested = Vec::<Vec<Vec<i32>>>::new();
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains("collect::<Result<Vec<Option<HashMap<String, Vec<u8>>>>, MyError>>()"), "Turbofish must not be broken");
    assert!(Formatted.contains("flags >> 2"), "Right shift operator must be preserved");

    let SecondPass = Source_Formatter::Format(&Formatted, &Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Must be idempotent");
}

#[test]
fn Test_Rust_Raw_Strings_And_Byte_Literals()
{
    let Input = r###"const R1: &str = r#"Hello "world" \n not escaped"#;
const R2: &str = r##"Has "# inside hashes"##;
const BYTES: &[u8] = b"raw\x00\xFFbytes";
const BYTE_RAW: &[u8] = br#"byte raw "string""#;
"###;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains(r###"r#"Hello "world" \n not escaped"#"###));
    assert!(Formatted.contains(r###"r##"Has "# inside hashes"##"###));
    assert!(Formatted.contains(r###"b"raw\x00\xFFbytes""###));
    assert!(Formatted.contains(r###"br#"byte raw "string""#"###));

    let SecondPass = Source_Formatter::Format(&Formatted, &Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Must be idempotent");
}

#[test]
fn Test_Rust_Closures_And_Return_Type_Arrows()
{
    let Input = r#"fn test_closures()
{
    let f1 = |x: i32| -> i32 { x + 1 };
    let f2 = move |a: &'static str, b: usize| -> Result<String, Error>
    {
        Ok(format!("{}: {}", a, b))
    };
    let f3 = |x| x * 2;
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains("-> i32"));
    assert!(Formatted.contains("-> Result<String, Error>"));

    let SecondPass = Source_Formatter::Format(&Formatted, &Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Must be idempotent");
}

#[test]
fn Test_Rust_Match_Guards_And_Patterns()
{
    let Input = r#"fn evaluate(opt: Option<i32>) -> &'static str
{
    match opt
    {
        Some(x) if x > 10 && x < 100 => "medium",
        Some(0..=9) => "single_digit",
        Some(100 | 200 | 300) => "century",
        ref mut other => "other",
        None => "empty",
    }
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains("Some(x) if x > 10 && x < 100 =>"));
    assert!(Formatted.contains("0..=9"));
    assert!(Formatted.contains("100 | 200 | 300"));

    let SecondPass = Source_Formatter::Format(&Formatted, &Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Must be idempotent");
}

#[test]
fn Test_Rust_Macros_And_Attributes()
{
    let Input = r#"#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMap<K, V>
where
    K: Eq + Hash,
{
    #[serde(default)]
    pub entries: HashMap<K, V>,
    #[cfg(feature = "metadata")]
    pub metadata: Option<Metadata>,
}
"#;
    let mut Config = Config_Settings::default();
    Config.Language = Source_Language::Rust;

    let Formatted = Source_Formatter::Format(Input, &Config).expect("Formatting failed");
    assert!(Formatted.contains("#[derive("));
    assert!(Formatted.contains("#[serde("));
    assert!(Formatted.contains("#[cfg("));

    let SecondPass = Source_Formatter::Format(&Formatted, &Config).expect("Second pass failed");
    assert_eq!(Formatted, SecondPass, "Must be idempotent");
}

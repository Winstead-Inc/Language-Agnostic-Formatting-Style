#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use Language_Agnostic_Formatting_Style::changelog:: { Changelog_Generator, Changelog_Options };

#[test]
fn Test_Changelog_Commit_Categorization()
{
    let MockCommits = vec!
    [
        ("a1b2c3d".to_string(), "feat: add support for markdown snippet formatting".to_string()),
        ("e4f5g6h".to_string(), "fix(parser): resolve relational operator angle hazard".to_string()),
        ("i7j8k9l".to_string(), "docs: update README with full flag documentation".to_string()),
        ("m0n1o2p".to_string(), "sec: enforce OWASP trust boundary validation".to_string()),
        ("q3r4s5t".to_string(), "refactor: optimize sole-child chain grouping".to_string()),
    ];

    let (Added, Fixed, Changed, Security, Documentation) = Changelog_Generator::CategorizeCommits( & MockCommits);

    assert_eq!(Added.len(), 1);
    assert!(Added[0].contains("Add support for markdown snippet formatting"));

    assert_eq!(Fixed.len(), 1);
    assert!(Fixed[0].contains("Resolve relational operator angle hazard"));

    assert_eq!(Security.len(), 1);
    assert!(Security[0].contains("Enforce OWASP trust boundary validation"));

    assert_eq!(Documentation.len(), 1);
    assert!(Documentation[0].contains("Update README with full flag documentation"));

    assert_eq!(Changed.len(), 1);
    assert!(Changed[0].contains("Optimize sole-child chain grouping"));
}

#[test]
fn Test_Changelog_Markdown_Generation_With_Options()
{
    let Options = Changelog_Options
    { Output_Path: None, Release_Version: Some("2.0.0".to_string()), Release_Date: Some("2026-11-20".to_string()), Output_To_Stdout: true, };

    let Markdown = Changelog_Generator::GenerateChangelogMarkdown( & Options);

    assert!(Markdown.contains("# Changelog"));
    assert!(Markdown.contains("## [2.0.0] - 2026-11-20"));
    assert!(Markdown.contains("All notable changes to the Language-Agnostic Formatting Style"));
}

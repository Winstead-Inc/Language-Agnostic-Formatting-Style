#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::types:: { Source_Language, Token_Kind };

/// Configuration for line and block comments per Container Matching Law §5.1 & §11.
#[derive(Debug, Clone, Copy)]
pub struct Comment_Profile
{ pub Line_Prefixes: & 'static[ & 'static str], pub Block_Pairs: & 'static[( & 'static str, & 'static str)], pub Nestable_Blocks: bool , }

/// Configuration for string literals, verbatim blocks, and quotes.
#[derive(Debug, Clone, Copy)]
pub struct String_Profile
{
    pub Delimiters: & 'static[char],
    pub Escape_Char: Option<char>,
    pub Allows_Multiline: bool,
    pub Supports_Raw_Strings: bool,
    pub Supports_Triple_Quotes: bool,
    pub Supports_Backticks: bool,
    pub Raw_Prefixes: & 'static[ & 'static str],
}

/// Configuration for string interpolation.
#[derive(Debug, Clone, Copy)]
pub struct Interpolation_Profile
{ pub Enabled: bool , pub Openers: & 'static[ & 'static str], }

/// Character literals, lifetime annotations, and digit separators.
#[derive(Debug, Clone, Copy)]
pub struct Char_Literal_Profile
{ pub Enabled: bool , pub Lifetime_Or_Label_Prefix: bool , pub Digit_Separator_Apostrophe: bool , }

/// Regular expression literal rules.
#[derive(Debug, Clone, Copy)]
pub struct Regex_Profile
{ pub Enabled: bool , pub Suffix_Flags: & 'static[char], }

/// Directives and preprocessors.
#[derive(Debug, Clone, Copy)]
pub struct Directive_Profile
{ pub Enabled: bool , pub Prefix: Option<char>, pub Supports_Continuation: bool , }

/// Markup / JSX / Polyglot text modes.
#[derive(Debug, Clone, Copy)]
pub struct Markup_Profile
{ pub Supports_JSX: bool , pub Supports_HTML: bool , pub Raw_Text_Elements: & 'static[ & 'static str], }

/// Atomic multi-character tokens that must never be split.
#[derive(Debug, Clone, Copy)]
pub struct Atomic_Tokens_Profile
{ pub Multi_Char_Operators: & 'static[ & 'static str], }

/// Tentative containers (overloaded glyphs like < > and | | per §7).
#[derive(Debug, Clone, Copy)]
pub struct Tentative_Profile
{
    pub Angle_Generics: bool,
    pub Closure_Pipes: bool,
    pub Turbofish: bool,
    pub Angle_Admission_Predecessors: & 'static[ & 'static str],
    pub Angle_Follow_Tokens: & 'static[ & 'static str],
    pub Angle_Invalid_Inside: & 'static[ & 'static str],
}

/// Keyword containers (e.g. begin/end, case/esac, do/end).
#[derive(Debug, Clone, Copy)]
pub struct Keyword_Container_Profile
{ pub Pairs: & 'static[( & 'static str, & 'static str)], }

/// The complete declarative Language Profile per Container Matching Law §5.1 & §11.
#[derive(Debug, Clone, Copy)]
pub struct Language_Profile
{
    pub Language: Source_Language,
    pub Name: & 'static str,
    pub Comments: Comment_Profile,
    pub Strings: String_Profile,
    pub Interpolation: Interpolation_Profile,
    pub Char_Literals: Char_Literal_Profile,
    pub Regex: Regex_Profile,
    pub Directives: Directive_Profile,
    pub Markup: Markup_Profile,
    pub Atomic_Tokens: Atomic_Tokens_Profile,
    pub Tentative: Tentative_Profile,
    pub Keyword_Containers: Keyword_Container_Profile,
}

impl Language_Profile
{
    pub fn For(Language: Source_Language) -> & 'static Language_Profile
    {
        match Language
        {
            Source_Language::Rust => & RUST_PROFILE,
            Source_Language::JavaScript => & JAVASCRIPT_PROFILE,
            Source_Language::TypeScript => & TYPESCRIPT_PROFILE,
            Source_Language::Jsx => & JAVASCRIPT_PROFILE,
            Source_Language::Tsx => & TYPESCRIPT_PROFILE,
            Source_Language::C => & C_PROFILE,
            Source_Language::Cpp => & CPP_PROFILE,
            Source_Language::CSharp => & CSHARP_PROFILE,
            Source_Language::Dart => & DART_PROFILE,
            Source_Language::Kotlin => & KOTLIN_PROFILE,
            Source_Language::KotlinOfficial => & KOTLIN_OFFICIAL_PROFILE,
            Source_Language::Go => & GO_PROFILE,
            Source_Language::Shell => & SHELL_PROFILE,
            Source_Language::Html => & HTML_PROFILE,
            Source_Language::Css => & CSS_PROFILE,
            Source_Language::Php => & PHP_PROFILE,
            Source_Language::Batch => & BATCH_PROFILE,
            Source_Language::Toml => & TOML_PROFILE,
            Source_Language::CargoLock => & CARGO_LOCK_PROFILE,
            Source_Language::Generic => & GENERIC_PROFILE,
        }
    }

    pub fn IsTentativeAngleAdmitted( & self, PrevText: & str, PrevKind: & Token_Kind) -> bool
    {
        if ! self.Tentative.Angle_Generics
        { return false; }

        if self.Tentative.Angle_Admission_Predecessors.contains( & PrevText)
        { return true; }

        matches!(PrevKind, Token_Kind::Identifier(_))
    }

    pub fn IsTentativeAngleFollowValid( & self, NextText: & str, NextKind: & Token_Kind, PrecedingNewlines: usize) -> bool
    {
        if PrecedingNewlines > 0
        { return true; }

        if self.Tentative.Angle_Follow_Tokens.contains( & NextText)
        { return true; }

        matches!(NextKind, Token_Kind::Identifier(_) | Token_Kind::Keyword(_))
    }

    pub fn IsTentativeAngleTokenRejected( & self, TokenText: & str, ParenDepth: usize, BraceDepth: usize, BracketDepth: usize) -> bool
    {
        if ParenDepth == 0 && TokenText == ")"
        { return true; }

        if BracketDepth == 0 && TokenText == "]"
        { return true; }

        if BraceDepth == 0 && TokenText == "}"
        { return true; }

        if (ParenDepth > 0 || BracketDepth > 0) && TokenText == "="
        { return true; }

        if BraceDepth == 0 && TokenText == ";"
        { return true; }

        self.Tentative.Angle_Invalid_Inside.contains( & TokenText)
    }
}

// ----------------------------------------------------------------------------
// Shared Token Tables
// ----------------------------------------------------------------------------

static NO_TENTATIVE: Tentative_Profile = Tentative_Profile
{
    Angle_Generics: false,
    Closure_Pipes: false,
    Turbofish: false,
    Angle_Admission_Predecessors: & [],
    Angle_Follow_Tokens: & [],
    Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
};

static COMMON_ANGLE_INVALID_INSIDE: & [ & str] = &
[
    "&&", "||", "==", "!=", "===", "!==", "<=", ">=",
    "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "??=", "&&=", "||=",
    "++", "--",
    "if", "else", "while", "for", "do", "switch", "case", "default",
    "return", "throw", "yield", "break", "continue",
    "try", "catch", "finally", "var", "let", "function", "debugger", "with",
];

static TS_ANGLE_FOLLOW_TOKENS: & [ & str] = & ["=", ":", ";", ",", ".", "?", "(", ")", "[", "]", "{", "}", ">", "extends", "implements", "as", "is", "=>", "|", "&", "never",];

static TS_ANGLE_ADMISSION: & [ & str] = & ["class", "interface", "type", "enum", "const", "new", "export", "default",];

static DART_ANGLE_FOLLOW_TOKENS: & [ & str] = & ["(", ")", "]", "}", ":", ";", ",", ".", "?", "[", "{", ">", "extends", "implements", "with", "on", "when", "as", "is", "async", "=",];

static DART_ANGLE_ADMISSION: & [ & str] = &
[
    "=", "=>", ":", ",", "(", "[", "{", "??", "const", "new", "Function",
    "class", "enum", "mixin", "extension", "typedef",
    "int", "double", "num", "bool", "dynamic", "void", "never", "symbol",
    "Object", "Comparable", "Future", "FutureOr", "Stream",
];

static DART_ANGLE_INVALID_INSIDE: & [ & str] = &
[
    "&&", "||", "==", "!=", "<=", ">=",
    "+", "-", "*", "/", "%",
    "+=", "-=", "*=", "/=", "~/=", "%=",
    "++", "--",
    "if", "else", "while", "for", "do", "switch", "case", "default",
    "return", "throw", "yield", "break", "continue",
    "try", "catch", "finally", "var",
];

static RUST_ANGLE_FOLLOW_TOKENS: & [ & str] = & ["(", ")", "[", "]", "{", "}", ":", ";", ",", ".", "::", "=", ">", "->", "=>", "where", "as",];

static RUST_ANGLE_ADMISSION: & [ & str] = & ["::", "struct", "enum", "trait", "impl", "fn", "type", "where", "dyn",];

static CPP_ANGLE_ADMISSION: & [ & str] = & ["template", "typename", "class", "struct", "concept", "using", "::", "new", "where",];

static CPP_ANGLE_FOLLOW_TOKENS: & [ & str] = & ["(", ")", "[", "]", "{", "}", ":", ";", ",", ".", "::", "=", ">", "?", "==", "!=", "&", "*", "->",];

// ----------------------------------------------------------------------------
// Language Profiles
// ----------------------------------------------------------------------------

pub static RUST_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Rust,
    Name: "Rust",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: true, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: true,
        Supports_Triple_Quotes: false,
        Supports_Backticks: false,
        Raw_Prefixes: & ["r", "r#", "br", "cr", "c", "b"],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: true, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["->", "=>", "::", "..", "...", "..=", "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "&&", "||", "==", "!=", "<=", ">="],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: true,
        Turbofish: true,
        Angle_Admission_Predecessors: RUST_ANGLE_ADMISSION,
        Angle_Follow_Tokens: RUST_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static JAVASCRIPT_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::JavaScript,
    Name: "JavaScript",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\'', '`'],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["${"], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: true, Suffix_Flags: & ['g', 'i', 'm', 's', 'u', 'y', 'd', 'v'], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: true, Supports_HTML: false, Raw_Text_Elements: & ["script", "style"], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: &
        [
            "=>", "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??", "??=", "&&=", "||=", "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "...", "?."
        ],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: TS_ANGLE_ADMISSION,
        Angle_Follow_Tokens: TS_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static TYPESCRIPT_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::TypeScript,
    Name: "TypeScript",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\'', '`'],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["${"], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: true, Suffix_Flags: & ['g', 'i', 'm', 's', 'u', 'y', 'd', 'v'], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: true, Supports_HTML: false, Raw_Text_Elements: & ["script", "style"], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: &
        [
            "=>", "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??", "??=", "&&=", "||=", "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "...", "?."
        ],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: TS_ANGLE_ADMISSION,
        Angle_Follow_Tokens: TS_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static DART_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Dart,
    Name: "Dart",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: true, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: true,
        Supports_Triple_Quotes: true,
        Supports_Backticks: false,
        Raw_Prefixes: & ["r"],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["${", "$"], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["=>", "??", "??=", "&&", "||", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "~/=", "%=", "..", "?..", "?."],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: DART_ANGLE_ADMISSION,
        Angle_Follow_Tokens: DART_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: DART_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static KOTLIN_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Kotlin,
    Name: "Kotlin",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: true, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\'', '`'],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: true,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["${", "$"], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["->", "::", "?:", "!!", "==", "!=", "===", "!==", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "..", "..<"],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: & ["class", "interface", "fun", "val", "var", "typealias", "enum"],
        Angle_Follow_Tokens: TS_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static KOTLIN_OFFICIAL_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::KotlinOfficial,
    Name: "Kotlin (Official)",
    Comments: KOTLIN_PROFILE.Comments,
    Strings: KOTLIN_PROFILE.Strings,
    Interpolation: KOTLIN_PROFILE.Interpolation,
    Char_Literals: KOTLIN_PROFILE.Char_Literals,
    Regex: KOTLIN_PROFILE.Regex,
    Directives: KOTLIN_PROFILE.Directives,
    Markup: KOTLIN_PROFILE.Markup,
    Atomic_Tokens: KOTLIN_PROFILE.Atomic_Tokens,
    Tentative: KOTLIN_PROFILE.Tentative,
    Keyword_Containers: KOTLIN_PROFILE.Keyword_Containers,
};

pub static C_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::C,
    Name: "C",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: false,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: false,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: true, Prefix: Some('#'), Supports_Continuation: true, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["->", "++", "--", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>"],
    },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static CPP_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Cpp,
    Name: "C++",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: true,
        Supports_Triple_Quotes: false,
        Supports_Backticks: false,
        Raw_Prefixes: & ["R", "LR", "u8R", "uR", "UR"],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: true, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: true, Prefix: Some('#'), Supports_Continuation: true, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["->", "->*", "::", "++", "--", "==", "!=", "<=", ">=", "<=>", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>"],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: CPP_ANGLE_ADMISSION,
        Angle_Follow_Tokens: CPP_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static CSHARP_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::CSharp,
    Name: "C#",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: true,
        Supports_Triple_Quotes: true,
        Supports_Backticks: false,
        Raw_Prefixes: & ["@", "$@", "@$"],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["{"], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: true, Prefix: Some('#'), Supports_Continuation: true, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["=>", "::", "??", "??=", "&&", "||", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "?."],
    },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: & ["class", "interface", "struct", "record", "delegate", "enum", "new", "where", "using", "::"],
        Angle_Follow_Tokens: CPP_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static GO_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Go,
    Name: "Go",
    Comments: Comment_Profile { Line_Prefixes: & ["//"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\'', '`'],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["->", "<-", ":=", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "++", "--"],
    },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static SHELL_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Shell,
    Name: "Shell",
    Comments: Comment_Profile { Line_Prefixes: & ["#"], Block_Pairs: & [], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: true,
        Supports_Triple_Quotes: false,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["${", "$", "$("], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: true, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile { Multi_Char_Operators: & ["$(", "$((", ";;", "&&", "||", ">&", "<&", "|&", "<<", ">>"], },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [("case", "esac"),("if", "fi")], },
};

pub static HTML_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Html,
    Name: "HTML",
    Comments: Comment_Profile { Line_Prefixes: & [], Block_Pairs: & [("<!--", "-->")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: None,
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: false,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: true, Raw_Text_Elements: & ["script", "style", "textarea", "title"], },
    Atomic_Tokens: Atomic_Tokens_Profile { Multi_Char_Operators: & ["</", "/>", "<!--", "-->", "<!"], },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static CSS_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Css,
    Name: "CSS",
    Comments: Comment_Profile { Line_Prefixes: & [], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: false,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: false,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile { Multi_Char_Operators: & [], },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static PHP_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Php,
    Name: "PHP",
    Comments: Comment_Profile { Line_Prefixes: & ["//", "#"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\'', '`'],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["{$", "$"], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: true, Raw_Text_Elements: & ["script", "style"], },
    Atomic_Tokens: Atomic_Tokens_Profile
    {
        Multi_Char_Operators: & ["->", "=>", "::", "??", "??=", "==", "!=", "===", "!==", "<=", ">=", "<=>", "&&", "||", "+=", "-=", "*=", "/="],
    },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static BATCH_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Batch,
    Name: "Batch",
    Comments: Comment_Profile { Line_Prefixes: & ["REM", "::", "rem"], Block_Pairs: & [], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"'],
        Escape_Char: Some('^'),
        Allows_Multiline: false,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: false,
        Supports_Backticks: false,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["%", "!"], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile { Multi_Char_Operators: & ["==", "&&", "||"], },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static TOML_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Toml,
    Name: "TOML",
    Comments: Comment_Profile { Line_Prefixes: & ["#"], Block_Pairs: & [], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\''],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: true,
        Supports_Triple_Quotes: true,
        Supports_Backticks: false,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: false, Openers: & [], },
    Char_Literals: Char_Literal_Profile { Enabled: false, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: false, Prefix: None, Supports_Continuation: false, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile { Multi_Char_Operators: & [], },
    Tentative: NO_TENTATIVE,
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

pub static CARGO_LOCK_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::CargoLock,
    Name: "Cargo.lock",
    Comments: TOML_PROFILE.Comments,
    Strings: TOML_PROFILE.Strings,
    Interpolation: TOML_PROFILE.Interpolation,
    Char_Literals: TOML_PROFILE.Char_Literals,
    Regex: TOML_PROFILE.Regex,
    Directives: TOML_PROFILE.Directives,
    Markup: TOML_PROFILE.Markup,
    Atomic_Tokens: TOML_PROFILE.Atomic_Tokens,
    Tentative: NO_TENTATIVE,
    Keyword_Containers: TOML_PROFILE.Keyword_Containers,
};

pub static GENERIC_PROFILE: Language_Profile = Language_Profile
{
    Language: Source_Language::Generic,
    Name: "Generic",
    Comments: Comment_Profile { Line_Prefixes: & ["//", "#"], Block_Pairs: & [("/*", "*/")], Nestable_Blocks: false, },
    Strings: String_Profile
    {
        Delimiters: & ['"', '\'', '`'],
        Escape_Char: Some('\\'),
        Allows_Multiline: true,
        Supports_Raw_Strings: false,
        Supports_Triple_Quotes: true,
        Supports_Backticks: true,
        Raw_Prefixes: & [],
    },
    Interpolation: Interpolation_Profile { Enabled: true, Openers: & ["${", "$"], },
    Char_Literals: Char_Literal_Profile { Enabled: true, Lifetime_Or_Label_Prefix: false, Digit_Separator_Apostrophe: false, },
    Regex: Regex_Profile { Enabled: false, Suffix_Flags: & [], },
    Directives: Directive_Profile { Enabled: true, Prefix: Some('#'), Supports_Continuation: true, },
    Markup: Markup_Profile { Supports_JSX: false, Supports_HTML: false, Raw_Text_Elements: & [], },
    Atomic_Tokens: Atomic_Tokens_Profile { Multi_Char_Operators: & ["->", "=>", "::", "==", "!=", "<=", ">=", "&&", "||"], },
    Tentative: Tentative_Profile
    {
        Angle_Generics: true,
        Closure_Pipes: false,
        Turbofish: false,
        Angle_Admission_Predecessors: & ["class", "interface", "type", "enum", "struct"],
        Angle_Follow_Tokens: TS_ANGLE_FOLLOW_TOKENS,
        Angle_Invalid_Inside: COMMON_ANGLE_INVALID_INSIDE,
    },
    Keyword_Containers: Keyword_Container_Profile { Pairs: & [], },
};

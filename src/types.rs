#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source_Language { C, Cpp, CSharp, JavaScript, TypeScript, Shell, Go, Php, Html, Css, Rust, Kotlin, KotlinOfficial, Batch, Toml, CargoLock, Dart, Generic, }

impl Source_Language
{
    pub fn FromExtension(PathString: & str) -> Self
    {
        let Lower = PathString.to_lowercase();
        if Lower.ends_with(".cs") { return Source_Language::CSharp; }
        if Lower.ends_with(".cpp") || Lower.ends_with(".cxx") || Lower.ends_with(".cc") || Lower.ends_with(".hpp") || Lower.ends_with(".hxx") || Lower.ends_with(".hh")
        || Lower.ends_with(".inl") || Lower.ends_with(".cu") { return Source_Language::Cpp; }
        if Lower.ends_with(".c") || Lower.ends_with(".h") { return Source_Language::C; }
        if Lower.ends_with(".rs") { return Source_Language::Rust; }
        if Lower.ends_with(".ts") || Lower.ends_with(".tsx") || Lower.ends_with(".mts") || Lower.ends_with(".cts") { return Source_Language::TypeScript; }
        if Lower.ends_with(".js") || Lower.ends_with(".jsx") || Lower.ends_with(".mjs") || Lower.ends_with(".cjs") { return Source_Language::JavaScript; }
        if Lower.ends_with(".sh") || Lower.ends_with(".bash") || Lower.ends_with(".zsh") || Lower.ends_with(".ksh") { return Source_Language::Shell; }
        if Lower.ends_with(".go") { return Source_Language::Go; }
        if Lower.ends_with(".php") { return Source_Language::Php; }
        if Lower.ends_with(".html") || Lower.ends_with(".htm") { return Source_Language::Html; }
        if Lower.ends_with(".css") { return Source_Language::Css; }
        if Lower.ends_with(".kt") || Lower.ends_with(".kts") { return Source_Language::Kotlin; }
        if Lower.ends_with(".bat") || Lower.ends_with(".cmd") { return Source_Language::Batch; }
        if Lower.ends_with("cargo.lock") || Lower.ends_with(".lock") || Lower == "lock" { return Source_Language::CargoLock; }
        if Lower.ends_with(".toml") || Lower == "toml" { return Source_Language::Toml; }
        if Lower.ends_with(".dart") { return Source_Language::Dart; }
        Source_Language::Generic
    }

    pub fn SupportsAngleGenerics( & self) -> bool
    {
        matches!
        (
            self, Source_Language::Cpp | Source_Language::CSharp | Source_Language::TypeScript | Source_Language::Rust | Source_Language::Kotlin | Source_Language::KotlinOfficial | Source_Language::Dart
        )
    }

    pub fn SupportsPreprocessor( & self) -> bool
    { matches ! (self, Source_Language::C | Source_Language::Cpp | Source_Language::CSharp) }

    pub fn IsJavaScriptOrTypeScript( & self) -> bool { matches ! (self, Source_Language::JavaScript | Source_Language::TypeScript) }

    pub fn IsShell( & self) -> bool { matches ! (self, Source_Language::Shell) }

    pub fn IsKotlin( & self) -> bool { matches ! (self, Source_Language::Kotlin | Source_Language::KotlinOfficial) }

    pub fn IsBatch( & self) -> bool { matches ! (self, Source_Language::Batch) }

    pub fn IsToml( & self) -> bool { matches ! (self, Source_Language::Toml | Source_Language::CargoLock) }

    pub fn IsCargoLock( & self) -> bool { matches ! (self, Source_Language::CargoLock) }

    pub fn IsDart( & self) -> bool { matches ! (self, Source_Language::Dart) }

    pub fn DefaultExtensions( & self) -> & 'static[ & 'static str]
    {
        match self
        {
            Source_Language::C => & ["c", "h"], Source_Language::Cpp => & ["cpp", "hpp", "cc", "cxx", "hh", "hxx", "inl", "cu", "h"], Source_Language::CSharp => & ["cs"],
            Source_Language::JavaScript => & ["js", "mjs", "cjs", "jsx"], Source_Language::TypeScript => & ["ts", "tsx", "mts", "cts"], Source_Language::Shell => & ["sh", "bash", "zsh", "ksh"],
            Source_Language::Go => & ["go"], Source_Language::Php => & ["php"], Source_Language::Html => & ["html", "htm"], Source_Language::Css => & ["css"], Source_Language::Rust => & ["rs"],
            Source_Language::Kotlin | Source_Language::KotlinOfficial => & ["kt", "kts"],
            Source_Language::Batch => & ["bat", "cmd"],
            Source_Language::Toml => & ["toml"],
            Source_Language::CargoLock => & ["lock"],
            Source_Language::Dart => & ["dart"],
            Source_Language::Generic => & [],
        }
    }

    pub fn ShortName( & self) -> & 'static str
    {
        match self
        {
            Source_Language::C => "c", Source_Language::Cpp => "cpp", Source_Language::CSharp => "cs", Source_Language::JavaScript => "js", Source_Language::TypeScript => "ts", Source_Language::Shell => "sh", Source_Language::Go => "go", Source_Language::Php => "php", Source_Language::Html => "html", Source_Language::Css => "css", Source_Language::Rust => "rs", Source_Language::Kotlin => "kt", Source_Language::KotlinOfficial => "kt-official", Source_Language::Batch => "bat", Source_Language::Toml => "toml", Source_Language::CargoLock => "cargo-lock", Source_Language::Dart => "dart", Source_Language::Generic => "generic",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Container_Kind { Paren, Brace, Bracket, Angle, }

impl Container_Kind
{
    pub fn OpenGlyph( & self) -> & 'static str { match self { Container_Kind::Paren => "(", Container_Kind::Brace => "{", Container_Kind::Bracket => "[", Container_Kind::Angle => "<", } }

    pub fn CloseGlyph( & self) -> & 'static str { match self { Container_Kind::Paren => ")", Container_Kind::Brace => "}", Container_Kind::Bracket => "]", Container_Kind::Angle => ">", } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token_Kind
{
    Identifier(String), Keyword(String), Number(String), String_Lit(String), Char_Lit(String), Template_Lit(String), Regex_Lit(String), Comment_Line(String), Comment_Block(String),
    Preprocessor(String), Open_Delim(Container_Kind), Close_Delim(Container_Kind), Operator(String), Punctuation(char), Whitespace(String), Newline, Eof, Virtual_Semicolon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token_Info { pub Kind: Token_Kind, pub Text: String, pub Line: usize, pub Column: usize, pub Preceding_Newlines: usize, pub Leading_Whitespace: String, }

impl Token_Info
{
    pub fn New(Kind: Token_Kind, Text: String, Line: usize, Column: usize, Preceding_Newlines: usize,) -> Self { Token_Info { Kind, Text, Line, Column, Preceding_Newlines, Leading_Whitespace: String::new (), } }

    pub fn NewWithIndent(Kind: Token_Kind, Text: String, Line: usize, Column: usize, Preceding_Newlines: usize, Leading_Whitespace: String,) -> Self { Token_Info { Kind, Text, Line, Column, Preceding_Newlines, Leading_Whitespace, } }

    pub fn IsTrivia( & self) -> bool { matches ! (self.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline) }

    pub fn IsComment( & self) -> bool { matches ! (self.Kind, Token_Kind::Comment_Line(_) | Token_Kind::Comment_Block(_)) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout_State { Monolith, Grouped, Expanded, }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kotlin_Style
{
    Allman, // Version 1: Pure Allman style for all containers
    Official, // Version 2: Official Kotlin (Column-Anchored Hybrid where required by Kotlin compiler grammar / Brain K&R)
}

#[derive(Debug, Clone)]
pub struct Config_Settings
{
    pub Canvas_Width: usize, pub Indent_Width: usize, pub Use_Tabs: bool, pub Max_Statements_In_Monolith: usize, pub Max_Block_Depth: usize, pub Max_Containers_In_Monolith: usize, pub Max_Blank_Lines: usize, pub Language: Source_Language, pub Allow_Brace_Grouping: bool, pub Kotlin_Style: Kotlin_Style,
}

impl Default for Config_Settings
{
    fn default() -> Self
    {
        Config_Settings
        {
            Canvas_Width: 160, Indent_Width: 4, Use_Tabs: false, Max_Statements_In_Monolith: 4, Max_Block_Depth: 2, Max_Containers_In_Monolith: 12, Max_Blank_Lines: 1, Language: Source_Language::Generic, Allow_Brace_Grouping: true, Kotlin_Style: Kotlin_Style::Allman,
        }
    }
}

impl Config_Settings
{
    pub fn IndentString( & self, Level: usize) -> String { if self.Use_Tabs { "\t".repeat(Level) } else { " ".repeat(Level * self.Indent_Width) } }
}

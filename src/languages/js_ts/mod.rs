#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
use crate::types:: { Config_Settings, Container_Kind, Layout_State, Source_Language, Token_Info, Token_Kind, };

pub struct JS_TS_Formatter;

impl JS_TS_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let mut Lexer = JS_Lexer::New(Source, Config.Language);
        let Tokens = Lexer.TokenizeAll();

        let mut Builder = JS_Container_Builder::New(Tokens, Config.Column_Anchored_Hybrid);
        let Nodes = Builder.BuildTree();

        let Emitter = JS_Emitter::New(Config);
        let Formatted = Emitter.Emit( & Nodes);
        // Safety gate verification
        if let Err(VerifyErr) = VerifyJSTokenEquivalence(Source, & Formatted, Config.Language)
        {
            return Err(format ! ("{}. Output aborted to prevent code corruption.", VerifyErr));
        }

        Ok(Formatted)
    }

    pub fn IsFormatted(Source: & str, Config: & Config_Settings) -> bool { match Self::Format(Source, Config) { Ok(Formatted) => Formatted == Source, Err(_) => false, } }
}

#[derive(Debug, Clone)]
struct Template_Context
{ Brace_Depth: usize, }

pub struct JS_Lexer<'a>
{
    Source: & 'a str,
    Chars: Vec<char>,
    Index: usize,
    Line: usize,
    Column: usize,
    Language: Source_Language,
    Expects_Expression: bool,
    Template_Stack: Vec<Template_Context>,
    In_JSX_Tag_Header: usize,
    JSX_Brace_Depth: usize,
}

impl <'a> JS_Lexer<'a>
{
    pub fn New(Source: & 'a str, Language: Source_Language) -> Self
    {
        JS_Lexer
        {
            Source,
            Chars: Source.chars().collect(),
            Index: 0,
            Line: 1,
            Column: 0,
            Language,
            Expects_Expression: true,
            Template_Stack: Vec:: new (),
            In_JSX_Tag_Header: 0,
            JSX_Brace_Depth: 0,
        }
    }

    fn Peek( & self) -> Option<char> { if self.Index < self.Chars.len() { Some(self.Chars[self.Index]) } else { None } }

    fn PeekAhead( & self, Offset: usize) -> Option<char>
    {
        let Target = self.Index + Offset;
        if Target < self.Chars.len() { Some(self.Chars[Target]) } else { None }
    }

    fn Advance( & mut self) -> Option<char>
    {
        if self.Index >= self.Chars.len() { return None; }
        let Ch = self.Chars[self.Index];
        self.Index += 1;
        if Ch == '\n' { self.Line += 1; self.Column = 0; } else { self.Column += 1; }
        Some(Ch)
    }

    pub fn TokenizeAll( & mut self) -> Vec<Token_Info>
    {
        let mut RawTokens: Vec < Token_Info > = Vec:: new ();
        let mut PrecedingNewlines: usize = 0;

        while let Some(Ch) = self.Peek()
        {
            let StartLine = self.Line;
            let StartCol = self.Column;
            // 1. Newline
            if Ch == '\n' { self.Advance(); PrecedingNewlines += 1; continue ; }
            // 2. Horizontal Whitespace
            if Ch == ' ' || Ch == '\t' || Ch == '\r' { self.Advance(); continue ; }
            // 3. Comments
            if Ch == '/' && self.PeekAhead(1) == Some('/')
            {
                let CommentText = self.ScanLineComment();
                RawTokens.push(Token_Info::New(Token_Kind::Comment_Line(CommentText.clone()), CommentText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '/' && self.PeekAhead(1) == Some('*')
            {
                let CommentText = self.ScanBlockComment();
                RawTokens.push(Token_Info::New(Token_Kind::Comment_Block(CommentText.clone()), CommentText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            // 4. Regular Expression Literal vs Division
            if Ch == '/' && self.Expects_Expression
            {
                if let Some(RegexText) = self.ScanRegexLiteral()
                {
                    RawTokens.push(Token_Info::New(Token_Kind::Regex_Lit(RegexText.clone()), RegexText, StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = false;
                    continue;
                }
            }
            // 5. Template Literals (Backticks) with ${...} chunking
            if Ch == '`'
            {
                let (ChunkText, HasInterp) = self.ScanTemplateStart();
                if ! ChunkText.is_empty()
                {
                    RawTokens.push(Token_Info::New(Token_Kind::Template_Lit(ChunkText.clone()), ChunkText, StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                }
                if HasInterp
                {
                    self.Template_Stack.push(Template_Context { Brace_Depth: 0 });
                    RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "${".to_string(), self.Line, self.Column, 0,));
                    self.Expects_Expression = true;
                }
                else
                { self.Expects_Expression = false; }
                continue;
            }
            // 6. Strings (Single and Double Quotes)
            if Ch == '"' || Ch == '\''
            {
                let QuoteChar = Ch;
                let StringText = self.ScanQuotedString(QuoteChar);
                RawTokens.push(Token_Info::New(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = false;
                continue;
            }
            // 7. Delimiters
            if Ch == '('
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Paren), "(".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = true;
                continue;
            }

            if Ch == ')'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Paren), ")".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = false;
                continue;
            }

            if Ch == '{'
            {
                self.Advance();
                if let Some(Ctx) = self.Template_Stack.last_mut()
                { Ctx.Brace_Depth += 1; }
                if self.In_JSX_Tag_Header > 0
                { self.JSX_Brace_Depth += 1; }
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "{".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = true;
                continue;
            }

            if Ch == '}'
            {
                if let Some(Ctx) = self.Template_Stack.last_mut()
                {
                    if Ctx.Brace_Depth > 0
                    {
                        Ctx.Brace_Depth -= 1;
                        self.Advance();
                        if self.In_JSX_Tag_Header > 0 && self.JSX_Brace_Depth > 0
                        { self.JSX_Brace_Depth -= 1; }
                        RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines,));
                        PrecedingNewlines = 0;
                        self.Expects_Expression = false;
                        continue;
                    }
                    else
                    {
                        self.Template_Stack.pop();
                        self.Advance();
                        RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines,));
                        PrecedingNewlines = 0;

                        let (ResumeChunk, HasAnotherInterp) = self.ScanTemplateResume();
                        if ! ResumeChunk.is_empty()
                        {
                            RawTokens.push(Token_Info::New(Token_Kind::Template_Lit(ResumeChunk.clone()), ResumeChunk, self.Line, self.Column, 0,));
                        }
                        if HasAnotherInterp
                        {
                            self.Template_Stack.push(Template_Context { Brace_Depth: 0 });
                            RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "${".to_string(), self.Line, self.Column, 0,));
                            self.Expects_Expression = true;
                        }
                        else
                        { self.Expects_Expression = false; }
                        continue;
                    }
                }
                else
                {
                    self.Advance();
                    if self.In_JSX_Tag_Header > 0 && self.JSX_Brace_Depth > 0
                    { self.JSX_Brace_Depth -= 1; }
                    RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = false;
                    continue;
                }
            }

            if Ch == '['
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Bracket), "[".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = true;
                continue;
            }

            if Ch == ']'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Bracket), "]".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = false;
                continue;
            }
            // 8. Numbers
            if Ch.is_ascii_digit() || (Ch == '.' && self.PeekAhead(1).map_or(false, | C | C.is_ascii_digit()))
            {
                let NumberText = self.ScanNumber();
                RawTokens.push(Token_Info::New(Token_Kind::Number(NumberText.clone()), NumberText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = false;
                continue;
            }
            // 8.5. ECMAScript Private Identifier (#ident)
            if Ch == '#' && !(self.Line == 1 && self.Column == 0 && self.PeekAhead(1) == Some('!'))
            {
                let mut Offset = 1;
                while let Some(Ws) = self.PeekAhead(Offset)
                { if Ws == ' ' || Ws == '\t' { Offset += 1; } else { break ; } }
                if self.PeekAhead(Offset).map_or(false, | C | C.is_alphabetic() || C == '_')
                {
                    self.Advance();
                    while let Some(Ws) = self.Peek()
                    { if Ws == ' ' || Ws == '\t' { self.Advance(); } else { break ; } }
                    let IdentText = self.ScanIdentifier();
                    let FullIdent = format!("#{}", IdentText);
                    RawTokens.push(Token_Info::New(Token_Kind::Identifier(FullIdent.clone()), FullIdent, StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = false;
                    continue;
                }
            }
            // 9. Identifiers and Keywords
            if Ch.is_alphabetic() || Ch == '_' || Ch == '$'
            {
                let IdentText = self.ScanIdentifier();
                let IsKw = IsJSKeyword( & IdentText);
                let Kind = if IsKw { Token_Kind::Keyword(IdentText.clone()) } else { Token_Kind::Identifier(IdentText.clone()) };
                // Expression context tracking
                if IsKw
                {
                    self.Expects_Expression = matches!(IdentText.as_str(), "return" | "throw" | "yield" | "case" | "typeof" | "delete" | "void" | "in" | "instanceof" | "of" | "new" | "await");
                }
                else { self.Expects_Expression = false; }

                RawTokens.push(Token_Info::New(Kind, IdentText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            // 10. Punctuation
            if Ch == ';' || Ch == ',' || Ch == '.' || Ch == '?' || Ch == ':'
            {
                if Ch == '.' && self.PeekAhead(1) == Some('.') && self.PeekAhead(2) == Some('.')
                {
                    self.Advance();
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("...".to_string()), "...".to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = true;
                    continue;
                }

                if Ch == '?' && self.PeekAhead(1) == Some('.')
                {
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("?.".to_string()), "?.".to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = false;
                    continue;
                }

                if Ch == '?' && self.PeekAhead(1) == Some('?')
                {
                    self.Advance();
                    self.Advance();
                    let OpStr = if self.Peek() == Some('=') { self.Advance(); "??=" }
                    else { "??" };
                    RawTokens.push(Token_Info::New(Token_Kind::Operator(OpStr.to_string()), OpStr.to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = true;
                    continue;
                }

                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Punctuation(Ch), Ch.to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = matches!(Ch, ';' | ',' | ':' | '?');
                continue;
            }
            // 11. Operators and JSX
            if Ch == '<'
            {
                self.Advance();
                if self.Peek() == Some('/')
                {
                    self.Advance();
                    while let Some(Ws) = self.Peek()
                    { if Ws == ' ' || Ws == '\t' { self.Advance(); } else { break ; } }
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("</".to_string()), "</".to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    self.Expects_Expression = true;
                    continue;
                }
                if self.Expects_Expression
                {
                    let mut Offset = 0;
                    while let Some(Ws) = self.PeekAhead(Offset)
                    { if Ws == ' ' || Ws == '\t' { Offset += 1; } else { break ; } }
                    if self.PeekAhead(Offset).map_or(false, | C | C.is_ascii_alphabetic() || C == '>')
                    {
                        for _ in 0..Offset { self.Advance(); }
                        self.In_JSX_Tag_Header += 1;
                    }
                }
                RawTokens.push(Token_Info::New(Token_Kind::Operator("<".to_string()), "<".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                self.Expects_Expression = true;
                continue;
            }

            let OpText = self.ScanOperator();
            RawTokens.push(Token_Info::New(Token_Kind::Operator(OpText.clone()), OpText.clone(), StartLine, StartCol, PrecedingNewlines,));
            PrecedingNewlines = 0;
            self.Expects_Expression = true;
        }
        // Post-process generic angle brackets in TypeScript
        if self.Language.SupportsAngleGenerics() { ResolveJSAngleBrackets( & mut RawTokens); }

        RawTokens
    }

    fn ScanLineComment( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek() { if Ch == '\n' { break ; } Result.push(self.Advance().unwrap()); }
        Result
    }

    fn ScanBlockComment( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // '/'
        Result.push(self.Advance().unwrap());
        // '*'

        while let Some(Ch) = self.Peek()
        {
            if Ch == '*' && self.PeekAhead(1) == Some('/')
            {
                Result.push(self.Advance().unwrap());
                Result.push(self.Advance().unwrap());
                break;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanQuotedString( & mut self, QuoteChar: char) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // Opening quote

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if self.Peek().is_some() { Result.push(self.Advance().unwrap()); }
                continue;
            }
            if Ch == QuoteChar { Result.push(self.Advance().unwrap()); break ; }
            if Ch == '\n'
            {
                // Unclosed string on newline
                break;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanTemplateStart( & mut self) -> (String, bool)
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // '`'

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if self.Peek().is_some() { Result.push(self.Advance().unwrap()); }
                continue;
            }
            if Ch == '$' && self.PeekAhead(1) == Some('{')
            { self.Advance(); self.Advance(); return (Result, true); }
            if Ch == '`'
            { Result.push(self.Advance().unwrap()); return (Result, false); }
            Result.push(self.Advance().unwrap());
        }
        (Result, false)
    }

    fn ScanTemplateResume( & mut self) -> (String, bool)
    {
        let mut Result = String:: new ();

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if self.Peek().is_some() { Result.push(self.Advance().unwrap()); }
                continue;
            }
            if Ch == '$' && self.PeekAhead(1) == Some('{')
            { self.Advance(); self.Advance(); return (Result, true); }
            if Ch == '`'
            { Result.push(self.Advance().unwrap()); return (Result, false); }
            Result.push(self.Advance().unwrap());
        }
        (Result, false)
    }

    fn ScanRegexLiteral( & mut self) -> Option<String>
    {
        let mut TempIndex = self.Index;
        if TempIndex >= self.Chars.len() || self.Chars[TempIndex] != '/' { return None; }
        TempIndex += 1;
        // skip '/'

        let mut InCharClass = false;

        while TempIndex < self.Chars.len()
        {
            let Ch = self.Chars[TempIndex];
            if Ch == '\n'
            {
                // Regex literals cannot contain raw unescaped newlines
                return None;
            }
            if Ch == '\\'
            {
                TempIndex += 2;
                // skip escape and escaped char
                continue;
            }
            if Ch == '[' { InCharClass = true; TempIndex += 1; continue ; }
            if Ch == ']' && InCharClass { InCharClass = false; TempIndex += 1; continue ; }
            if Ch == '/' && ! InCharClass
            {
                TempIndex += 1;
                // Scan trailing flags: g, i, m, s, u, y, d, v
                while TempIndex < self.Chars.len() && self.Chars[TempIndex].is_ascii_alphabetic() { TempIndex += 1; }
                // Valid regex! Advance lexer to TempIndex and build string
                let mut Result = String:: new ();
                while self.Index < TempIndex { Result.push(self.Advance().unwrap()); }
                return Some(Result);
            }
            TempIndex += 1;
        }

        None
    }

    fn ScanNumber( & mut self) -> String
    {
        let mut Result = String:: new ();
        // Check for 0x, 0b, 0o
        if self.Peek() == Some('0')
        {
            Result.push(self.Advance().unwrap());
            if let Some(Prefix) = self.Peek()
            {
                if matches!(Prefix, 'x' | 'X' | 'b' | 'B' | 'o' | 'O')
                {
                    Result.push(self.Advance().unwrap());
                    while let Some(Ch) = self.Peek() { if Ch.is_ascii_hexdigit() || Ch == '_' { Result.push(self.Advance().unwrap()); } else { break ; } }
                    if self.Peek() == Some('n') { Result.push(self.Advance().unwrap()); }
                    return Result;
                }
            }
        }

        while let Some(Ch) = self.Peek()
        {
            if Ch.is_ascii_digit() || Ch == '.' || Ch == '_' { Result.push(self.Advance().unwrap()); }
            else if Ch == 'e' || Ch == 'E'
            {
                Result.push(self.Advance().unwrap());
                if let Some(Sign) = self.Peek() { if Sign == '+' || Sign == '-' { Result.push(self.Advance().unwrap()); } }
            }
            else if Ch == 'n'
            {
                // BigInt suffix
                Result.push(self.Advance().unwrap());
                break;
            }
            else { break ; }
        }
        Result
    }

    fn ScanIdentifier( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek()
        {
            if Ch.is_alphanumeric() || Ch == '_' || Ch == '$'
            { Result.push(self.Advance().unwrap()); }
            else if Ch == '-' && self.In_JSX_Tag_Header > 0 && self.JSX_Brace_Depth == 0
            {
                let mut Offset = 1;
                while let Some(Ws) = self.PeekAhead(Offset)
                { if Ws == ' ' || Ws == '\t' { Offset += 1; } else { break ; } }
                if self.PeekAhead(Offset).map_or(false, | C | C.is_alphabetic() || C == '_')
                {
                    self.Advance();
                    while let Some(Ws) = self.Peek()
                    { if Ws == ' ' || Ws == '\t' { self.Advance(); } else { break ; } }
                    Result.push('-');
                }
                else { break ; }
            }
            else { break ; }
        }
        Result
    }

    fn ScanOperator( & mut self) -> String
    {
        let mut Result = String:: new ();
        let First = self.Advance().unwrap();
        Result.push(First);

        let Next = self.Peek();
        if First == '/' && Next == Some('>')
        {
            Result.push(self.Advance().unwrap());
            if self.In_JSX_Tag_Header > 0 && self.JSX_Brace_Depth == 0
            { self.In_JSX_Tag_Header -= 1; }
            return Result;
        }

        match (First, Next)
        {
            ('=', Some('='))
            =>
            { Result.push(self.Advance().unwrap()); if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); } }
            ('!', Some('='))
            =>
            { Result.push(self.Advance().unwrap()); if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); } }
            ('=', Some('>'))
            => { Result.push(self.Advance().unwrap()); }
            ('&', Some('&'))
            =>
            { Result.push(self.Advance().unwrap()); if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); } }
            ('|', Some('|'))
            =>
            { Result.push(self.Advance().unwrap()); if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); } }
            ('?', Some('?'))
            =>
            { Result.push(self.Advance().unwrap()); if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); } }
            ('<', Some('<'))
            | ('>', Some('>')) =>
            {
                Result.push(self.Advance().unwrap());
                if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); }
            }
            ('<', Some('='))
            | ('>', Some('=')) => { Result.push(self.Advance().unwrap()); }
            ('+', Some('+'))
            | ('-', Some('-')) => { Result.push(self.Advance().unwrap()); }
            ('*', Some('*'))
            =>
            { Result.push(self.Advance().unwrap()); if self.Peek() == Some('=') { Result.push(self.Advance().unwrap()); } }
            ('+', Some('='))
            | ('-', Some('=')) | ('*', Some('=')) | ('/', Some('=')) | ('%', Some('=')) | ('&', Some('=')) | ('|', Some('=')) | ('^', Some('='))
            => { Result.push(self.Advance().unwrap()); }
            _ => {}
        }

        if Result == ">" && self.In_JSX_Tag_Header > 0 && self.JSX_Brace_Depth == 0
        { self.In_JSX_Tag_Header -= 1; }

        Result
    }
}

fn IsJSKeyword(Text: & str) -> bool
{
    matches!
    (
        Text, "async" | "await" | "break" | "case" | "catch" | "class" | "const" | "continue" | "debugger" | "default" | "delete" | "do" | "else" | "enum" | "export" | "extends" | "false" | "finally" | "for" | "from" | "function" | "get" | "if" | "implements" | "import" | "in" | "instanceof" | "interface" | "let" | "new" | "null" | "of" | "package" | "private" | "protected" | "public" | "readonly" | "return" | "set" | "static" | "super" | "switch" | "this" | "throw" | "true" | "try" | "type" | "typeof" | "var" | "void" | "while" | "with" | "yield" | "as" | "declare" | "namespace" | "module"
    )
}

fn OutputEndsWithControlKeyword(Output: & str) -> bool
{
    let ControlKeywords =
    [
        "if", "while", "for", "switch", "catch", "with", "return", "throw", "yield", "await", "case", "typeof", "void", "delete", "instanceof", "in", "of",
    ];

    for Kw in ControlKeywords
    {
        if Output.ends_with(Kw)
        {
            let PrefixLen = Output.len() - Kw.len();
            if PrefixLen == 0 { return true; }
            if let Some(Ch) = Output[..PrefixLen].chars().last()
            { if ! Ch.is_alphanumeric() && Ch != '_' && Ch != '$' { return true; } }
        }
    }

    false
}

fn IsHtmlTagName(name: &str) -> bool
{
    matches!(
        name,
        "a" | "abbr" | "address" | "area" | "article" | "aside" | "audio" | "b" | "base"
        | "bdi" | "bdo" | "blockquote" | "body" | "br" | "button" | "canvas" | "caption"
        | "cite" | "code" | "col" | "colgroup" | "data" | "datalist" | "dd" | "del"
        | "details" | "dfn" | "dialog" | "div" | "dl" | "dt" | "em" | "embed" | "fieldset"
        | "figcaption" | "figure" | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5"
        | "h6" | "head" | "header" | "hgroup" | "hr" | "html" | "i" | "iframe" | "img"
        | "input" | "ins" | "kbd" | "label" | "legend" | "li" | "link" | "main" | "map"
        | "mark" | "menu" | "meta" | "meter" | "nav" | "noscript" | "object" | "ol"
        | "optgroup" | "option" | "output" | "p" | "picture" | "pre" | "progress" | "q"
        | "rp" | "rt" | "ruby" | "s" | "samp" | "script" | "section" | "select" | "slot"
        | "small" | "source" | "span" | "strong" | "style" | "sub" | "summary" | "sup"
        | "table" | "tbody" | "td" | "template" | "textarea" | "tfoot" | "th" | "thead"
        | "time" | "title" | "tr" | "track" | "u" | "ul" | "var" | "video" | "wbr"
        | "svg" | "path" | "circle" | "rect" | "line" | "polyline" | "polygon"
    )
}

fn ResolveJSAngleBrackets(Tokens: & mut Vec<Token_Info>)
{
    let mut I = 0;
    while I < Tokens.len()
    {
        if Tokens[I].Text != "<" { I += 1; continue ; }

        if I + 1 < Tokens.len() && IsHtmlTagName(&Tokens[I + 1].Text)
        {
            I += 1;
            continue;
        }

        let PrecedingValid = if I == 0 { false }
        else
        {
            let Prev = & Tokens[I - 1];
            let Text = Prev.Text.as_str();
            matches!(Text, "class" | "interface" | "type" | "enum")
            || matches!(Prev.Kind, Token_Kind::Identifier(_))
        };

        if ! PrecedingValid { I += 1; continue ; }

        let mut Depth: usize = 0;
        let mut MatchIndex: Option<usize> = None;
        let mut J = I;
        let mut ContainsInvalidTypeToken = false;
        let mut ParenDepth: usize = 0;
        let mut BracketDepth: usize = 0;
        let mut BraceDepth: usize = 0;

        while J < Tokens.len()
        {
            let TText = Tokens[J].Text.as_str();
            if TText == "("
            { ParenDepth += 1; }
            else if TText == ")"
            { if ParenDepth > 0 { ParenDepth -= 1; } }
            else if TText == "["
            { BracketDepth += 1; }
            else if TText == "]"
            { if BracketDepth > 0 { BracketDepth -= 1; } }
            else if TText == "{" || TText == "${"
            { BraceDepth += 1; }
            else if TText == "}"
            { if BraceDepth > 0 { BraceDepth -= 1; } }
            else if TText == "<"
            { Depth += 1; }
            else if TText == ">"
            {
                if Depth > 0
                {
                    Depth -= 1;
                    if Depth == 0 && ParenDepth == 0 && BracketDepth == 0 && BraceDepth == 0 { MatchIndex = Some(J); break ; }
                }
            }
            else if Tokens[J].Text.chars().all( | c | c == '>') && Tokens[J].Text.len() > 1 && Depth >= 1
            {
                let Count = Tokens[J].Text.len();
                let Tok = Tokens[J].clone();
                Tokens[J].Kind = Token_Kind::Operator(">".to_string());
                Tokens[J].Text = ">".to_string();
                for Offset in 1..Count
                {
                    Tokens.insert(J + Offset, Token_Info::New(Token_Kind::Operator(">".to_string()), ">".to_string(), Tok.Line, Tok.Column + Offset, 0));
                }
                Depth -= 1;
                if Depth == 0 && ParenDepth == 0 && BracketDepth == 0 && BraceDepth == 0 { MatchIndex = Some(J); break ; }
            }
            else if ParenDepth == 0 && BracketDepth == 0 && BraceDepth == 0
            && (TText == ";" || matches!(TText, "==" | "!=" | "<=" | ">=" | "if" | "while" | "return" | "throw" | "yield"))
            {
                ContainsInvalidTypeToken = true;
                break ;
            }
            J += 1;
        }

        if ContainsInvalidTypeToken { I += 1; continue ; }

        if let Some(M) = MatchIndex
        {
            let FollowValid = if M + 1 < Tokens.len()
            {
                let Next = & Tokens[M + 1];
                matches!
                (
                    Next.Text.as_str(),
                    "=" | ":" | ";" | "," | "." | "?" | "(" | ")" | "[" | "]" | "{" | "}"
                    | ">" | "extends" | "implements" | "as" | "is" | "=>" | "|" | "&" | "never"
                )
                || matches!(Next.Kind, Token_Kind::Identifier(_) | Token_Kind::Keyword(_))
                || Next.Preceding_Newlines > 0
            }
            else
            { true };

            if FollowValid
            {
                Tokens[I].Kind = Token_Kind::Open_Delim(Container_Kind::Angle);
                Tokens[M].Kind = Token_Kind::Close_Delim(Container_Kind::Angle);
            }
        }
        I += 1;
    }
}

#[derive(Debug, Clone)]
pub enum JS_Syntax_Node { Token(Token_Info), Container(JS_Container), }

#[derive(Debug, Clone)]
pub struct JS_Container
{
    pub Kind: Container_Kind, pub Open_Token: Token_Info, pub Close_Token: Option<Token_Info>, pub Children: Vec<JS_Syntax_Node>, pub Tail_Glue: Vec<Token_Info>,
    pub Has_Forced_Break: bool, pub Statement_Count: usize, pub Is_ASI_Hybrid: bool,
}

impl JS_Container
{
    pub fn New(Kind: Container_Kind, Open_Token: Token_Info, Is_ASI_Hybrid: bool) -> Self
    {
        JS_Container
        {
            Kind, Open_Token, Close_Token: None, Children: Vec:: new (), Tail_Glue: Vec:: new (), Has_Forced_Break: false, Statement_Count: 0, Is_ASI_Hybrid,
        }
    }

    pub fn TotalStatements(&self) -> usize
    {
        let TailStmts = self.Children.iter().filter(|c| match c {
            JS_Syntax_Node::Container(c) => c.Tail_Glue.iter().any(|g| g.Text == ";"),
            _ => false,
        }).count();
        self.Statement_Count + TailStmts
    }

    pub fn IsSoleChildChain( & self) -> bool
    {
        if self.Children.len() == 1
        {
            if let JS_Syntax_Node::Container(ref C) = self.Children[0]
            {
                return C.Tail_Glue.is_empty() && self.Close_Token.is_some() && C.Close_Token.is_some();
            }
        }
        false
    }

    pub fn GetSoleChildContainer( & self) -> Option< & JS_Container>
    {
        if self.Children.len() == 1
        {
            if let JS_Syntax_Node::Container(ref C) = self.Children[0]
            { if C.Tail_Glue.is_empty() && self.Close_Token.is_some() && C.Close_Token.is_some() { return Some(C); } }
        }
        None
    }

    pub fn CollectChain( & self) -> Vec< & JS_Container>
    {
        let mut Chain = vec![self];
        let mut Current = self;
        while let Some(Child) = Current.GetSoleChildContainer()
        {
            if (Current.Kind == Container_Kind::Angle && Child.Kind != Container_Kind::Brace) || (Child.Kind == Container_Kind::Angle && Current.Kind != Container_Kind::Angle) { break ; }
            Chain.push(Child);
            Current = Child;
        }
        Chain
    }

    pub fn CalculateSingleLineSpan( & self) -> usize
    {
        if self.Has_Forced_Break { return usize::MAX; }

        let mut Total = self.Kind.OpenGlyph().len() + self.Kind.CloseGlyph().len();
        for Glue in & self.Tail_Glue { Total += Glue.Text.len(); }

        let mut PrevTokenNeedsSpace = false;
        for Node in & self.Children
        {
            match Node
            {
                JS_Syntax_Node::Token(T) =>
                {
                    if T.IsTrivia() { continue ; }
                    if T.IsComment() && matches!(T.Kind, Token_Kind::Comment_Line(_)) { return usize::MAX; }
                    if PrevTokenNeedsSpace { Total += 1; }
                    Total += T.Text.len();
                    PrevTokenNeedsSpace = true;
                }
                JS_Syntax_Node::Container(C) =>
                {
                    let ChildSpan = C.CalculateSingleLineSpan();
                    if ChildSpan == usize::MAX { return usize::MAX; }
                    if PrevTokenNeedsSpace { Total += 1; }
                    Total += ChildSpan;
                    PrevTokenNeedsSpace = true;
                }
            }
        }

        Total
    }

    pub fn CalculateBlockDepth( & self) -> usize
    {
        let mut MaxChildDepth = 0;
        for Child in & self.Children
        {
            if let JS_Syntax_Node::Container(Sub) = Child
            {
                let SubDepth = Sub.CalculateBlockDepth();
                if SubDepth > MaxChildDepth { MaxChildDepth = SubDepth; }
            }
        }
        if self.Kind == Container_Kind::Brace { 1 + MaxChildDepth } else { MaxChildDepth }
    }

    pub fn CountTotalContainers( & self) -> usize
    {
        let mut Count = 1;
        for Child in & self.Children { if let JS_Syntax_Node::Container(Sub) = Child { Count += Sub.CountTotalContainers(); } }
        Count
    }
}

pub struct JS_Container_Builder
{ Tokens: Vec<Token_Info>, Index: usize, Enable_CAH: bool , }

impl JS_Container_Builder
{
    pub fn New(Tokens: Vec<Token_Info>, Enable_CAH: bool) -> Self
    { JS_Container_Builder { Tokens, Index: 0, Enable_CAH } }

    pub fn BuildTree( & mut self) -> Vec<JS_Syntax_Node>
    {
        let mut RootNodes = Vec:: new ();
        let mut LastSignificantWasReturn = false;
        while self.Index < self.Tokens.len()
        {
            if let Some(Node) = self.ParseNextNode(self.Enable_CAH && LastSignificantWasReturn)
            {
                match & Node
                {
                    JS_Syntax_Node::Token(T) =>
                    {
                        if ! T.IsTrivia() && ! T.IsComment()
                        {
                            LastSignificantWasReturn = matches!(T.Text.as_str(), "return" | "throw" | "yield" | "default")
                            || (T.Text == "*" && LastSignificantWasReturn);
                        }
                    }
                    JS_Syntax_Node::Container(_) =>
                    { LastSignificantWasReturn = false; }
                }
                RootNodes.push(Node);
            }
        }
        RootNodes
    }

    fn ParseNextNode( & mut self, PrecededByReturn: bool) -> Option<JS_Syntax_Node>
    {
        if self.Index >= self.Tokens.len() { return None; }
        let Token = self.Tokens[self.Index].clone();
        self.Index += 1;

        match & Token.Kind
        {
            Token_Kind::Open_Delim(Kind) =>
            {
                let mut Container = JS_Container::New( * Kind, Token, PrecededByReturn);
                self.PopulateContainerChildren( & mut Container);
                Some(JS_Syntax_Node::Container(Container))
            }
            _ => Some(JS_Syntax_Node::Token(Token)),
        }
    }

    fn PopulateContainerChildren( & mut self, Container: & mut JS_Container)
    {
        let mut StatementCount = 0;
        let mut LastSignificantWasReturn = false;

        while self.Index < self.Tokens.len()
        {
            let CurrentToken = self.Tokens[self.Index].clone();

            if CurrentToken.Text.contains('\n') { Container.Has_Forced_Break = true; }

            match & CurrentToken.Kind
            {
                Token_Kind::Close_Delim(CloseKind) if * CloseKind == Container.Kind =>
                {
                    self.Index += 1;
                    Container.Close_Token = Some(CurrentToken);
                    // Consume trailing glue (; or , or : or [] or [],)
                    while self.Index < self.Tokens.len()
                    {
                        let NextToken = & self.Tokens[self.Index];
                        if NextToken.Preceding_Newlines == 0
                        {
                            if NextToken.Text == ";" || NextToken.Text == "," || (NextToken.Text == ":" && Container.Kind == Container_Kind::Bracket)
                            {
                                Container.Tail_Glue.push(NextToken.clone());
                                self.Index += 1;
                            }
                            else if Container.Kind == Container_Kind::Angle && NextToken.Text == "[" && self.Index + 1 < self.Tokens.len() && self.Tokens[self.Index + 1].Text == "]" && self.Tokens[self.Index + 1].Preceding_Newlines == 0
                            {
                                Container.Tail_Glue.push(NextToken.clone());
                                Container.Tail_Glue.push(self.Tokens[self.Index + 1].clone());
                                self.Index += 2;
                            }
                            else { break ; }
                        }
                        else { break ; }
                    }
                    break;
                }
                Token_Kind::Open_Delim(_) =>
                {
                    if let Some(ChildNode) = self.ParseNextNode(self.Enable_CAH && LastSignificantWasReturn) { Container.Children.push(ChildNode); }
                    LastSignificantWasReturn = false;
                }
                Token_Kind::Comment_Line(_) =>
                {
                    Container.Has_Forced_Break = true;
                    Container.Children.push(JS_Syntax_Node::Token(CurrentToken));
                    self.Index += 1;
                    LastSignificantWasReturn = false;
                }
                _ =>
                {
                    if CurrentToken.Text == ";" { StatementCount += 1; }

                    if ! CurrentToken.IsTrivia() && ! CurrentToken.IsComment()
                    {
                        LastSignificantWasReturn = matches!(CurrentToken.Text.as_str(), "return" | "throw" | "yield" | "default")
                        || (CurrentToken.Text == "*" && LastSignificantWasReturn);
                    }

                    Container.Children.push(JS_Syntax_Node::Token(CurrentToken));
                    self.Index += 1;
                }
            }
        }

        Container.Statement_Count = StatementCount;
    }
}

pub struct JS_Layout_Planner<'a> { Config: & 'a Config_Settings, }

impl <'a> JS_Layout_Planner<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self { JS_Layout_Planner { Config } }

    pub fn DecideLayout( & self, Container: & JS_Container, CurrentIndentLevel: usize, CurrentColumn: usize,) -> Layout_State
    {
        let HasComma = Container.Children.iter().any(|c| match c {
            JS_Syntax_Node::Token(t) => t.Text == ",",
            JS_Syntax_Node::Container(sub) => sub.Tail_Glue.iter().any(|g| g.Text == ","),
        });

        // Variadic tuple in conditional extends
        if Container.Kind == Container_Kind::Bracket && Container.Children.iter().any(|c| match c {
            JS_Syntax_Node::Token(t) => t.Text == "infer",
            _ => false,
        }) && HasComma
        {
            return Layout_State::Expanded;
        }

        // Multiline function parameter list containing complex types / objects
        if Container.Kind == Container_Kind::Paren && Container.Children.iter().any(|c| match c {
            JS_Syntax_Node::Container(sub) => sub.Kind == Container_Kind::Brace,
            _ => false,
        }) && HasComma && Container.CalculateSingleLineSpan() > 70
        {
            return Layout_State::Expanded;
        }

        // Brace container with 3 or more statements/properties
        if Container.Kind == Container_Kind::Brace && Container.TotalStatements() >= 3
        {
            return Layout_State::Expanded;
        }

        // If a generic container is a Sole-Child Chain whose inner Brace has multiple statements
        if Container.Kind == Container_Kind::Angle && Container.IsSoleChildChain()
        {
            if let Some(inner) = Container.GetSoleChildContainer()
            {
                if inner.Kind == Container_Kind::Brace && inner.TotalStatements() > 1
                {
                    return Layout_State::Grouped;
                }
            }
        }

        // Nested generic containing multiline/multi-property child container
        if Container.Kind == Container_Kind::Angle && Container.Children.iter().any(|c| match c {
            JS_Syntax_Node::Container(sub) => sub.Kind == Container_Kind::Angle && sub.Children.iter().any(|gc| match gc {
                JS_Syntax_Node::Container(gsc) => gsc.Kind == Container_Kind::Brace && gsc.TotalStatements() > 1,
                _ => false,
            }),
            _ => false,
        })
        {
            return Layout_State::Expanded;
        }

        // 1. Monolith First (Horizontal Density)
        if ! Container.Has_Forced_Break && Container.Statement_Count <= self.Config.Max_Statements_In_Monolith && Container.CalculateBlockDepth() <= self.Config.Max_Block_Depth && Container.CountTotalContainers()
        <= self.Config.Max_Containers_In_Monolith
        {
            let SingleSpan = Container.CalculateSingleLineSpan();
            if SingleSpan != usize::MAX
            {
                let BaseCol = if CurrentColumn > 0 { CurrentColumn } else { CurrentIndentLevel * self.Config.Indent_Width };
                let ProjectedCol = BaseCol + SingleSpan;
                if ProjectedCol <= self.Config.Canvas_Width { return Layout_State::Monolith; }
            }
        }
        // 2. Brace Grouping Second (Sole-Child Chain Compound Delimiters)
        if self.Config.Allow_Brace_Grouping && Container.IsSoleChildChain()
        {
            let Chain = Container.CollectChain();
            if Chain.len() >= 2 { return Layout_State::Grouped; }
        }
        // 3. Expanded Symmetry Last (Hierarchical Architecture)
        Layout_State::Expanded
    }
}

pub struct JS_Emitter<'a>
{
    Config: & 'a Config_Settings,
    Planner: JS_Layout_Planner<'a>,
    Output: String,
    Current_Line_Len: usize,
    Line_Has_Content: bool,
    Base_Column_Offsets: Vec<usize>,
}

impl <'a> JS_Emitter<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self
    {
        JS_Emitter
        {
            Config,
            Planner: JS_Layout_Planner::New(Config),
            Output: String:: new (),
            Current_Line_Len: 0,
            Line_Has_Content: false,
            Base_Column_Offsets: Vec:: new (),
        }
    }

    pub fn Emit(mut self, Nodes: & [JS_Syntax_Node]) -> String
    {
        self.EmitNodeList(Nodes, 0);
        if ! self.Output.ends_with('\n') { self.Output.push('\n'); }
        self.Output
    }

    fn WriteNewline( & mut self)
    {
        while self.Output.ends_with(' ') { self.Output.pop(); }
        self.Output.push('\n');
        self.Current_Line_Len = 0;
        self.Line_Has_Content = false;
    }

    fn EnsureNewline( & mut self) { if self.Line_Has_Content { self.WriteNewline(); } }

    fn EmitIndent( & mut self, Level: usize)
    {
        if ! self.Line_Has_Content
        {
            let IndentStr = if let Some( & BaseCol) = self.Base_Column_Offsets.last()
            {
                let Additional = Level * self.Config.Indent_Width;
                " ".repeat(BaseCol + Additional)
            }
            else
            { self.Config.IndentString(Level) };
            self.Output.push_str( & IndentStr);
            self.Current_Line_Len = IndentStr.len();
            self.Line_Has_Content = true;
        }
    }

    fn EmitRaw( & mut self, Text: & str) { self.Output.push_str(Text); self.Current_Line_Len += Text.len(); self.Line_Has_Content = true; }

    fn EmitSpace( & mut self)
    {
        if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
        {
            self.Output.push(' ');
            self.Current_Line_Len += 1;
        }
    }

    fn EmitNodeList( & mut self, Nodes: & [JS_Syntax_Node], Level: usize)
    {
        let Total = Nodes.len();
        let mut Index = 0;
        let mut InPropertyValue = false;

        while Index < Total
        {
            let Node = & Nodes[Index];
            let EffectiveLevel = if InPropertyValue { Level + 1 } else { Level };

            match Node
            {
                JS_Syntax_Node::Token(T) =>
                {
                    if T.Text == "?" && (InPropertyValue || (Index + 1 < Total && matches!(& Nodes[Index + 1], JS_Syntax_Node::Container(c) if c.Kind == Container_Kind::Brace || c.Kind == Container_Kind::Paren)))
                    {
                        self.EnsureNewline();
                        self.EmitIndent(EffectiveLevel);
                        self.EmitRaw("?");
                        self.WriteNewline();
                    }
                    else if T.Text == ":" && ! self.Line_Has_Content
                    {
                        self.EmitIndent(EffectiveLevel);
                        self.EmitRaw(":");
                        self.EmitSpace();
                    }
                    else
                    {
                        self.EmitSingleToken(T, EffectiveLevel, Nodes, Index);
                    }

                    if T.Text == ";"
                    {
                        InPropertyValue = false;
                    }
                }
                JS_Syntax_Node::Container(C) =>
                {
                    if C.Open_Token.Text == "${"
                    { self.EmitStringInterpolation(C, EffectiveLevel); Index += 1; continue ; }

                    let NextTokenText: Option<&str> = if Index + 1 < Total
                    {
                        match & Nodes[Index + 1]
                        {
                            JS_Syntax_Node::Token(T) => Some(T.Text.as_str()),
                            JS_Syntax_Node::Container(Sub) => Some(Sub.Open_Token.Text.as_str()),
                        }
                    }
                    else
                    {
                        None
                    };

                    let Layout = self.Planner.DecideLayout(C, EffectiveLevel, self.Current_Line_Len);
                    self.EmitContainerNodeWithLayout(C, EffectiveLevel, Layout, NextTokenText);

                    if C.Tail_Glue.iter().any(|g| g.Text == ",") && EffectiveLevel > 0
                    {
                        self.WriteNewline();
                    }

                    if C.Tail_Glue.iter().any(|g| g.Text == ":")
                    {
                        let HasMultiLineValue = Nodes[Index + 1..].iter()
                            .take_while(|n| match n { JS_Syntax_Node::Token(t) => t.Text != ";" && t.Text != ",", _ => true })
                            .any(|n| match n {
                                JS_Syntax_Node::Container(_) => true,
                                JS_Syntax_Node::Token(t) => t.Preceding_Newlines > 0 || t.Text == "?" || t.Text == ":",
                            });

                        if HasMultiLineValue
                        {
                            self.EnsureNewline();
                            InPropertyValue = true;
                        }
                    }

                    // Check if followed by continuation keyword (else, catch, finally)
                    let NextContinuation = if Index + 1 < Total
                    {
                        match & Nodes[Index + 1] { JS_Syntax_Node::Token(T) if matches ! (T.Text.as_str(), "else" | "catch" | "finally") => { Some(T.Text.clone()) } _ => None, }
                    }
                    else { None };

                    if NextContinuation.is_some() { self.EnsureNewline(); }
                    else if C.Tail_Glue.iter().any( | G | G.Text == ";") || C.Kind == Container_Kind::Brace { self.EnsureNewline(); }
                }
            }

            Index += 1;
        }
    }

    fn EmitSingleToken( & mut self, Token: & Token_Info, Level: usize, Nodes: & [JS_Syntax_Node], Index: usize)
    {
        if let Token_Kind::Comment_Line(CommentText) = & Token.Kind
        {
            if self.Line_Has_Content { self.EmitSpace(); } else { self.EmitIndent(Level); }
            self.EmitRaw(CommentText);
            self.WriteNewline();
            return ;
        }

        if let Token_Kind::Comment_Block(CommentText) = & Token.Kind
        {
            if ! self.Line_Has_Content { self.EmitIndent(Level); } else { self.EmitSpace(); }
            self.EmitRaw(CommentText);
            return ;
        }

        if let Token_Kind::Template_Lit(ref Text) = & Token.Kind
        {
            if Text.starts_with("`\n")
            {
                self.WriteNewline();
                self.EmitIndent(Level);
                self.EmitRaw("`");
                let Rest = & Text[1..];
                self.EmitRaw(Rest);
                return ;
            }
        }

        if Token.IsTrivia() { return ; }

        if Token.Preceding_Newlines > 0
        {
            let BlankLines = (Token.Preceding_Newlines - 1).min(self.Config.Max_Blank_Lines);
            self.EnsureNewline();
            for _ in 0..BlankLines { self.WriteNewline(); }
        }

        if ! self.Line_Has_Content
        {
            self.EmitIndent(Level);
        }
        else
        {
            let NextIsColon = Index + 1 < Nodes.len() && match & Nodes[Index + 1] {
                JS_Syntax_Node::Token(next_t) => next_t.Text == ":",
                _ => false,
            };
            if ! (Token.Text == "?" && NextIsColon)
            {
                self.EmitSpaceBeforeToken(Token);
            }
        }

        self.EmitRaw( & Token.Text);
        if Token.Text == ";" || (Token.Text == "," && Level > 0) { self.WriteNewline(); }
        else if let Token_Kind::Keyword(K) = & Token.Kind
        {
            if matches!
            (
                K.as_str(), "if" | "while" | "for" | "switch" | "catch" | "with" | "return" | "throw" | "yield" | "case" | "new" | "typeof" | "void" | "delete" | "as" | "from" | "export" | "import" | "const" | "let" | "var" | "function" | "class" | "interface" | "type" | "enum" | "extends" | "implements" | "async" | "await" | "default" | "else" | "readonly" | "infer" | "keyof" | "in"
            )
            { self.EmitSpace(); }
        }
        else if let Token_Kind::Operator(Op) = & Token.Kind
        {
            if matches!
            (
                Op.as_str(),
                "=" | "=>" | "===" | "!==" | "==" | "!=" | "<=" | ">=" | "&&" | "||" | "??" | "??=" | "&&=" | "||=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "&=" | "|=" | "^=" | "<<" | ">>" | ">>>" | "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^"
            )
            { self.EmitSpace(); }
        }
    }

    fn EmitSpaceBeforeToken( & mut self, Token: & Token_Info)
    {
        if matches!(Token.Kind, Token_Kind::Template_Lit(_))
        {
            if self.Output.ends_with(':') || self.Output.ends_with('?') || self.Output.ends_with('=')
            {
                self.EmitSpace();
            }
            return ;
        }
        if Token.Text == ";" || Token.Text == "," || Token.Text == "." || Token.Text == "?." { return ; }
        if self.Output.ends_with('.') || self.Output.ends_with("?.") { return ; }
        if self.Output.ends_with('(') || self.Output.ends_with('[') { return ; }
        if self.Output.ends_with('<') || self.Output.ends_with("</") || self.Output.ends_with('#') { return ; }
        if self.Output.ends_with("${") { return ; }
        if Token.Text == ">" || Token.Text == "/>" { return ; }
        if Token.Text == ":"
        {
            if self.Output.ends_with('`')
            {
                self.EmitSpace();
            }
            return ;
        }
        self.EmitSpace();
    }

    fn EmitContainerNodeWithLayout( & mut self, Container: & JS_Container, Level: usize, Layout: Layout_State, NextTokenText: Option<&str>)
    {
        if Container.Open_Token.Text == "${"
        { self.EmitStringInterpolation(Container, Level); return ; }

        match Layout
        {
            Layout_State::Monolith => { self.EmitMonolith(Container, Level); }
            Layout_State::Grouped => { self.EmitGrouped(Container, Level); }
            Layout_State::Expanded => { self.EmitExpanded(Container, Level, NextTokenText); }
        }
    }

    fn EmitMonolith( & mut self, Container: & JS_Container, Level: usize)
    {
        if Container.Open_Token.Text == "${"
        { self.EmitStringInterpolation(Container, Level); return ; }

        if ! self.Line_Has_Content { self.EmitIndent(Level); }
        else if Container.Kind == Container_Kind::Brace && ! self.Output.ends_with('<') { self.EmitSpace(); }
        else if Container.Kind == Container_Kind::Paren
        {
            if ! self.Output.ends_with(' ')
            {
                if OutputEndsWithControlKeyword( & self.Output)
                { self.EmitSpace(); }
                else
                {
                    let LastChar = self.Output.chars().last();
                    if let Some(Ch) = LastChar { if ! Ch.is_alphanumeric() && Ch != '_' && Ch != '$' && Ch != ')' && Ch != ']' && Ch != '.' { self.EmitSpace(); } }
                }
            }
        }

        self.EmitRaw(Container.Kind.OpenGlyph());

        let mut PrevNeedsSpace = matches!(Container.Kind, Container_Kind::Brace);
        for Child in & Container.Children
        {
            match Child
            {
                JS_Syntax_Node::Token(T) =>
                {
                    if T.IsTrivia() { continue ; }
                    let IsTemplateLit = matches!(T.Kind, Token_Kind::Template_Lit(_));
                    let IsPunct = T.Text == ";" || T.Text == "," || T.Text == "." || (T.Text == ":" && ! self.Output.ends_with('`')) || T.Text == "?." || T.Text == ">" || T.Text == "/>";
                    let TemplateNeedsSpace = IsTemplateLit && (self.Output.ends_with('?') || self.Output.ends_with(':') || self.Output.ends_with('='));
                    if ((PrevNeedsSpace && ! IsPunct && ! IsTemplateLit) || TemplateNeedsSpace)
                    && ! self.Output.ends_with('.')
                    && ! self.Output.ends_with("?.")
                    && ! self.Output.ends_with('(')
                    && ! self.Output.ends_with('[')
                    && ! self.Output.ends_with('<')
                    && ! self.Output.ends_with("</")
                    && ! self.Output.ends_with('#')
                    { self.EmitSpace(); }
                    self.EmitRaw( & T.Text);
                    PrevNeedsSpace = ! matches!(T.Text.as_str(), "." | "?." | "(" | "[" | "!" | "~" | "<" | "</" | "#");
                }
                JS_Syntax_Node::Container(Sub) =>
                {
                    if Sub.Open_Token.Text == "${"
                    { self.EmitStringInterpolation(Sub, Level); PrevNeedsSpace = false; continue ; }
                    if PrevNeedsSpace && Sub.Kind == Container_Kind::Brace && ! self.Output.ends_with('<') { self.EmitSpace(); }
                    self.EmitMonolith(Sub, Level);
                    PrevNeedsSpace = true;
                }
            }
        }

        if matches!(Container.Kind, Container_Kind::Brace) && ! self.Output.ends_with(' ') && ! self.Output.ends_with('{') { self.EmitSpace(); }

        self.EmitRaw(Container.Kind.CloseGlyph());
        for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
    }

    fn EmitGrouped( & mut self, Container: & JS_Container, Level: usize)
    {
        let Chain = Container.CollectChain();
        let OpenRun: String = Chain.iter().map( | C | C.Kind.OpenGlyph()).collect();
        let CloseRun: String = Chain.iter().rev().map( | C | C.Kind.CloseGlyph()).collect();

        if Container.Is_ASI_Hybrid
        {
            if ! self.Output.ends_with(' ') { self.EmitSpace(); }
            let OpenCol = self.Current_Line_Len;
            self.EmitRaw( & OpenRun);
            self.WriteNewline();

            self.Base_Column_Offsets.push(OpenCol);
            if let Some(Innermost) = Chain.last()
            { self.EmitNodeList( & Innermost.Children, 1); }
            self.Base_Column_Offsets.pop();

            self.EnsureNewline();
            let CloseIndent = " ".repeat(OpenCol);
            self.EmitRaw( & CloseIndent);
            self.EmitRaw( & CloseRun);
            if let Some(Outermost) = Chain.first()
            { for Glue in & Outermost.Tail_Glue { self.EmitRaw( & Glue.Text); } }
            self.WriteNewline();
            return ;
        }

        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw( & OpenRun);
        self.WriteNewline();

        if let Some(Innermost) = Chain.last() { self.EmitNodeList( & Innermost.Children, Level + 1); }

        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw( & CloseRun);

        if let Some(Outermost) = Chain.first() { for Glue in & Outermost.Tail_Glue { self.EmitRaw( & Glue.Text); } }
        self.WriteNewline();
    }

    fn EmitExpanded( & mut self, Container: & JS_Container, Level: usize, NextTokenText: Option<&str>)
    {
        if Container.Open_Token.Text == "${"
        { self.EmitStringInterpolation(Container, Level); return ; }

        // Check Column-Anchored Hybrid Exception (ASI protection after return / throw / yield)
        if Container.Is_ASI_Hybrid
        {
            let OpenGlyph = Container.Kind.OpenGlyph();
            let CloseGlyph = Container.Kind.CloseGlyph();

            if ! self.Output.ends_with(' ') { self.EmitSpace(); }
            let OpenCol = self.Current_Line_Len;
            self.EmitRaw(OpenGlyph);
            self.WriteNewline();

            self.Base_Column_Offsets.push(OpenCol);
            self.EmitNodeList( & Container.Children, 1);
            self.Base_Column_Offsets.pop();

            self.EnsureNewline();
            let CloseIndent = " ".repeat(OpenCol);
            self.EmitRaw( & CloseIndent);
            self.EmitRaw(CloseGlyph);
            for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
            self.WriteNewline();
            return ;
        }
        // Standard LAFS Detached Delimiter Symmetry
        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();

        self.EmitNodeList( & Container.Children, Level + 1);

        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.CloseGlyph());
        for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
        if Container.Kind == Container_Kind::Angle
        {
            if NextTokenText == Some("=") || Container.Tail_Glue.iter().any(|g| g.Text == ",")
            {
                // do not write newline, keep `> =` or `>,` on same line
            }
            else
            {
                self.WriteNewline();
            }
        }
        else if NextTokenText == Some("=>")
        {
            // do not write newline before `=>`
        }
        else
        {
            self.WriteNewline();
        }
    }

    fn EmitStringInterpolation( & mut self, Container: & JS_Container, _Level: usize)
    {
        self.EmitRaw("${");

        let FirstChildHasNewline = Container.Children.first().map_or
        (
            false,
            | node | match node
            { JS_Syntax_Node::Token(t) => t.Preceding_Newlines > 0, JS_Syntax_Node::Container(c) => c.Open_Token.Preceding_Newlines > 0, }
        );

        let IsExpanded = FirstChildHasNewline || Container.Has_Forced_Break || Container.Statement_Count > 1 || Container.CalculateBlockDepth() > 1;

        if ! IsExpanded
        { self.EmitNodeList( & Container.Children, 0); if Container.Close_Token.is_some() { self.EmitRaw("}"); } }
        else
        {
            let LastNl = self.Output.rfind('\n').map_or(0, | idx | idx + 1);
            let CurrentLine = & self.Output[LastNl..];
            let LeadingSpaces = CurrentLine.chars().take_while( | c | * c == ' ').count();
            let BaseLevel = LeadingSpaces / self.Config.Indent_Width;

            self.WriteNewline();
            self.EmitNodeList( & Container.Children, BaseLevel + 1);
            self.EnsureNewline();
            if self.Output.ends_with("\n\n") { self.Output.pop(); }
            self.EmitIndent(BaseLevel);
            self.EmitRaw("}");
        }

        for Glue in & Container.Tail_Glue
        { self.EmitRaw( & Glue.Text); }
    }
}

fn AreTokensEquivalent(Orig: & str, Form: & str) -> bool
{
    if Orig == Form { return true; }
    if (Orig.starts_with("//") && Form.starts_with("//")) || (Orig.starts_with("/*") && Form.starts_with("/*"))
    {
        return Orig.trim_end() == Form.trim_end();
    }
    if (Orig.starts_with('`') || Orig.ends_with('`')) && (Form.starts_with('`') || Form.ends_with('`'))
    { return true; }
    false
}

pub fn VerifyJSTokenEquivalence(OriginalSource: & str, FormattedSource: & str, Language: Source_Language,) -> Result < (), String >
{
    let mut OriginalScanner = JS_Lexer::New(OriginalSource, Language);
    let OriginalTokens = OriginalScanner.TokenizeAll();

    let mut FormattedScanner = JS_Lexer::New(FormattedSource, Language);
    let FormattedTokens = FormattedScanner.TokenizeAll();

    let OrigSignificant: Vec < & str > = OriginalTokens.iter().filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline)).map( | T | T.Text.as_str())
   .collect();

    let FormSignificant: Vec < & str > = FormattedTokens.iter().filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline)).map( | T | T.Text.as_str())
   .collect();

    if OrigSignificant.len() != FormSignificant.len()
    {
        return Err(format ! ("Safety Gate Failed: Significant token count mismatch! (Original: {}, Formatted: {})", OrigSignificant.len(), FormSignificant.len()));
    }

    for (Index, (Orig, Form)) in OrigSignificant.iter().zip(FormSignificant.iter()).enumerate()
    {
        if Orig != Form && ! AreTokensEquivalent(Orig, Form)
        {
            return Err(format ! ("Safety Gate Failed: Token #{} mismatch! (Expected '{}', Got '{}')", Index, Orig, Form));
        }
    }

    Ok(())
}

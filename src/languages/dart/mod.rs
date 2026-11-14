#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]

use crate::types:: { Config_Settings, Container_Kind, Layout_State, Source_Language, Token_Info, Token_Kind };

pub struct Dart_Formatter;

impl Dart_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let mut Lexer = Dart_Lexer::New(Source);
        let Tokens = Lexer.TokenizeAll();

        let mut Builder = Dart_Container_Builder::New(Tokens);
        let Nodes = Builder.BuildTree();

        let Emitter = Dart_Emitter::New(Config);
        let Formatted = Emitter.Emit( & Nodes);

        // Safety gate verification
        if let Err(VerifyErr) = VerifyDartTokenEquivalence(Source, & Formatted)
        {
            return Err(format ! ("{}. Output aborted to prevent code corruption.", VerifyErr));
        }

        Ok(Formatted)
    }

    pub fn IsFormatted(Source: & str, Config: & Config_Settings) -> bool
    { match Self::Format(Source, Config) { Ok(Formatted) => Formatted == Source, Err(_) => false, } }
}

// ============================================================================
// DART LEXER (100% Character-Based Lookahead, UTF-8 Panic-Safe)
// ============================================================================

pub struct Dart_Lexer<'a>
{ Source: & 'a str, Chars: Vec<char>, Index: usize, Line: usize, Column: usize, Multiline_Stack: Vec<Multiline_Context>, }

#[derive(Clone, Debug)]
struct Multiline_Context
{ Quote_Char: char , Brace_Depth: usize, }

impl <'a> Dart_Lexer<'a>
{
    pub fn New(Source: & 'a str) -> Self
    { Dart_Lexer { Source, Chars: Source.chars().collect(), Index: 0, Line: 1, Column: 0, Multiline_Stack: Vec::new (), } }

    fn Peek( & self) -> Option<char>
    { if self.Index < self.Chars.len() { Some(self.Chars[self.Index]) } else { None } }

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
            if Ch == '\n'
            { self.Advance(); PrecedingNewlines += 1; continue ; }

            // 2. Horizontal Whitespace
            if Ch == ' ' || Ch == '\t' || Ch == '\r'
            { self.Advance(); continue ; }

            // 3. Comments (Line //, Doc ///, and Nested Block /* ... */)
            if Ch == '/' && self.PeekAhead(1) == Some('/')
            {
                let CommentText = self.ScanLineComment();
                RawTokens.push(Token_Info::New(Token_Kind::Comment_Line(CommentText.clone()), CommentText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '/' && self.PeekAhead(1) == Some('*')
            {
                let CommentText = self.ScanNestedBlockComment();
                RawTokens.push(Token_Info::New(Token_Kind::Comment_Block(CommentText.clone()), CommentText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 4. Raw Multiline Strings (r'''...''' or r"""...""")
            if Ch == 'r' && (self.PeekAhead(1) == Some('\'') || self.PeekAhead(1) == Some('"'))
            {
                let QuoteChar = self.PeekAhead(1).unwrap();
                if self.PeekAhead(2) == Some(QuoteChar) && self.PeekAhead(3) == Some(QuoteChar)
                {
                    let StringText = self.ScanRawMultilineString(QuoteChar);
                    RawTokens.push(Token_Info::New(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines));
                    PrecedingNewlines = 0;
                    continue;
                }
                else
                {
                    let StringText = self.ScanRawSingleLineString(QuoteChar);
                    RawTokens.push(Token_Info::New(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines));
                    PrecedingNewlines = 0;
                    continue;
                }
            }

            // 5. Multiline Strings ('''...''' or """...""") with ${...} chunking
            if (Ch == '\'' || Ch == '"') && self.PeekAhead(1) == Some(Ch) && self.PeekAhead(2) == Some(Ch)
            {
                let QuoteChar = Ch;
                let (ChunkText, HasInterp) = self.ScanMultilineStringStart(QuoteChar);
                if ! ChunkText.is_empty()
                {
                    RawTokens.push(Token_Info::New(Token_Kind::String_Lit(ChunkText.clone()), ChunkText, StartLine, StartCol, PrecedingNewlines));
                    PrecedingNewlines = 0;
                }
                if HasInterp
                {
                    self.Multiline_Stack.push(Multiline_Context { Quote_Char: QuoteChar, Brace_Depth: 0 });
                    RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "${".to_string(), self.Line, self.Column, 0));
                }
                continue;
            }

            // 6. Standard Strings ('...' or "...")
            if Ch == '\'' || Ch == '"'
            {
                let StringText = self.ScanStandardString(Ch);
                RawTokens.push(Token_Info::New(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 7. Cascades (.. and ?..)
            if Ch == '?' && self.PeekAhead(1) == Some('.') && self.PeekAhead(2) == Some('.')
            {
                self.Advance();
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("?..".to_string()), "?..".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '.' && self.PeekAhead(1) == Some('.')
            {
                if self.PeekAhead(2) == Some('.')
                {
                    // Spread operator (...)
                    self.Advance();
                    self.Advance();
                    self.Advance();
                    if self.Peek() == Some('?')
                    {
                        // Null-aware spread (...?)
                        self.Advance();
                        RawTokens.push(Token_Info::New(Token_Kind::Operator("...?".to_string()), "...?".to_string(), StartLine, StartCol, PrecedingNewlines));
                    }
                    else
                    {
                        RawTokens.push(Token_Info::New(Token_Kind::Operator("...".to_string()), "...".to_string(), StartLine, StartCol, PrecedingNewlines));
                    }
                    PrecedingNewlines = 0;
                    continue;
                }
                else
                {
                    // Cascade (..)
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("..".to_string()), "..".to_string(), StartLine, StartCol, PrecedingNewlines));
                    PrecedingNewlines = 0;
                    continue;
                }
            }

            // 8. Delimiters
            if Ch == '('
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Paren), "(".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == ')'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Paren), ")".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '{'
            {
                self.Advance();
                if let Some(Ctx) = self.Multiline_Stack.last_mut()
                { Ctx.Brace_Depth += 1; }
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "{".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '}'
            {
                self.Advance();
                if let Some(Ctx) = self.Multiline_Stack.last_mut()
                {
                    if Ctx.Brace_Depth > 0
                    {
                        Ctx.Brace_Depth -= 1;
                        RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines));
                        PrecedingNewlines = 0;
                        continue;
                    }
                    else
                    {
                        let QuoteChar = Ctx.Quote_Char;
                        self.Multiline_Stack.pop();
                        RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines));
                        PrecedingNewlines = 0;

                        let (ResumeChunk, HasAnotherInterp) = self.ScanMultilineStringResume(QuoteChar);
                        if ! ResumeChunk.is_empty()
                        {
                            RawTokens.push(Token_Info::New(Token_Kind::String_Lit(ResumeChunk.clone()), ResumeChunk, self.Line, self.Column, 0));
                        }
                        if HasAnotherInterp
                        {
                            self.Multiline_Stack.push(Multiline_Context { Quote_Char: QuoteChar, Brace_Depth: 0 });
                            RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "${".to_string(), self.Line, self.Column, 0));
                        }
                        continue;
                    }
                }
                else
                {
                    RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines));
                    PrecedingNewlines = 0;
                    continue;
                }
            }
            if Ch == '['
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Bracket), "[".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == ']'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Bracket), "]".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 9. Numbers (Dec, Hex, Floats, with digit separators _)
            if Ch.is_ascii_digit() || (Ch == '.' && self.PeekAhead(1).map_or(false, | C | C.is_ascii_digit()))
            {
                let NumberText = self.ScanNumber();
                RawTokens.push(Token_Info::New(Token_Kind::Number(NumberText.clone()), NumberText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 10. Dot Shorthands (.violetCore, .origin(), .new()) vs Member Access
            if Ch == '.' && self.PeekAhead(1).map_or(false, | C | C.is_alphabetic() || C == '_')
            {
                let IsDotShorthand = match RawTokens.last()
                {
                    None => true,
                    Some(Prev) =>
                    {
                        if Prev.Text == "this" || Prev.Text == "super" || Prev.Text == ">" || Prev.Text.ends_with('>')
                        { false }
                        else
                        {
                            matches!(Prev.Kind, Token_Kind::Open_Delim(_) | Token_Kind::Operator(_) | Token_Kind::Punctuation(':' | ',' | ';') | Token_Kind::Keyword(_))
                        }
                    }
                };

                if IsDotShorthand
                {
                    self.Advance();
                    // '.'
                    let IdentText = self.ScanIdentifier();
                    let FullToken = format!(".{}", IdentText);
                    RawTokens.push(Token_Info::New(Token_Kind::Identifier(FullToken.clone()), FullToken, StartLine, StartCol, PrecedingNewlines));
                    PrecedingNewlines = 0;
                    continue;
                }
            }

            // 11. Identifiers and Keywords
            if Ch.is_alphabetic() || Ch == '_' || Ch == '$'
            {
                let IdentText = self.ScanIdentifier();
                let Kind = if Self::IsKeyword( & IdentText)
                { Token_Kind::Keyword(IdentText.clone()) }
                else
                { Token_Kind::Identifier(IdentText.clone()) };

                RawTokens.push(Token_Info::New(Kind, IdentText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 12. Punctuation
            if Ch == ';' || Ch == ',' || Ch == ':'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Punctuation(Ch), Ch.to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 13. Multi-character Operators
            if let Some(OpText) = self.ScanOperator()
            {
                RawTokens.push(Token_Info::New(Token_Kind::Operator(OpText.clone()), OpText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 14. Fallback Single Character
            let FallbackCh = self.Advance().unwrap();
            RawTokens.push(Token_Info::New(Token_Kind::Operator(FallbackCh.to_string()), FallbackCh.to_string(), StartLine, StartCol, PrecedingNewlines));
            PrecedingNewlines = 0;
        }

        // Post-processing: Angle Generic Detection (<T extends Comparable<T>>)
        Self::ClassifyAngleGenerics( & mut RawTokens);

        RawTokens
    }

    fn ScanLineComment( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek()
        { if Ch == '\n' { break ; } Result.push(self.Advance().unwrap()); }
        Result
    }

    fn ScanNestedBlockComment( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // '/'
        Result.push(self.Advance().unwrap());
        // '*'
        let mut Depth: usize = 1;

        while let Some(Ch) = self.Peek()
        {
            if Ch == '/' && self.PeekAhead(1) == Some('*')
            {
                Result.push(self.Advance().unwrap());
                Result.push(self.Advance().unwrap());
                Depth += 1;
            }
            else if Ch == '*' && self.PeekAhead(1) == Some('/')
            {
                Result.push(self.Advance().unwrap());
                Result.push(self.Advance().unwrap());
                Depth -= 1;
                if Depth == 0 { break ; }
            }
            else
            { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanRawMultilineString( & mut self, QuoteChar: char) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // 'r'
        for _ in 0..3 { Result.push(self.Advance().unwrap()); }

        while let Some(Ch) = self.Peek()
        {
            if Ch == QuoteChar && self.PeekAhead(1) == Some(QuoteChar) && self.PeekAhead(2) == Some(QuoteChar)
            {
                for _ in 0..3 { Result.push(self.Advance().unwrap()); }
                break;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanRawSingleLineString( & mut self, QuoteChar: char) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // 'r'
        Result.push(self.Advance().unwrap());
        // quote

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\n' { break ; }
            if Ch == QuoteChar
            { Result.push(self.Advance().unwrap()); break ; }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanMultilineStringStart( & mut self, QuoteChar: char) -> (String, bool)
    {
        let mut Result = String:: new ();
        for _ in 0..3 { Result.push(self.Advance().unwrap()); }

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); }
            }
            else if Ch == '$' && self.PeekAhead(1) == Some('{')
            { self.Advance(); self.Advance(); return (Result, true); }
            else if Ch == QuoteChar && self.PeekAhead(1) == Some(QuoteChar) && self.PeekAhead(2) == Some(QuoteChar)
            {
                for _ in 0..3 { Result.push(self.Advance().unwrap()); }
                return (Result, false);
            }
            else
            { Result.push(self.Advance().unwrap()); }
        }
        (Result, false)
    }

    fn ScanMultilineStringResume( & mut self, QuoteChar: char) -> (String, bool)
    {
        let mut Result = String:: new ();

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); }
            }
            else if Ch == '$' && self.PeekAhead(1) == Some('{')
            { self.Advance(); self.Advance(); return (Result, true); }
            else if Ch == QuoteChar && self.PeekAhead(1) == Some(QuoteChar) && self.PeekAhead(2) == Some(QuoteChar)
            {
                for _ in 0..3 { Result.push(self.Advance().unwrap()); }
                return (Result, false);
            }
            else
            { Result.push(self.Advance().unwrap()); }
        }
        (Result, false)
    }

    fn ScanStandardString( & mut self, QuoteChar: char) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); }
            }
            else if Ch == QuoteChar
            { Result.push(self.Advance().unwrap()); break ; }
            else if Ch == '\n'
            { break ; }
            else
            { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanNumber( & mut self) -> String
    {
        let mut Result = String:: new ();

        if self.Peek() == Some('0') && (self.PeekAhead(1) == Some('x') || self.PeekAhead(1) == Some('X'))
        {
            Result.push(self.Advance().unwrap());
            // '0'
            Result.push(self.Advance().unwrap());
            // 'x'
            while let Some(Ch) = self.Peek()
            { if Ch.is_ascii_hexdigit() || Ch == '_' { Result.push(self.Advance().unwrap()); } else { break ; } }
            return Result;
        }

        let mut HasDot = false;
        let mut HasExp = false;

        while let Some(Ch) = self.Peek()
        {
            if Ch.is_ascii_digit() || Ch == '_'
            { Result.push(self.Advance().unwrap()); }
            else if Ch == '.' && ! HasDot && ! HasExp && self.PeekAhead(1).map_or(false, | C | C.is_ascii_digit())
            {
                HasDot = true;
                Result.push(self.Advance().unwrap());
            }
            else if (Ch == 'e' || Ch == 'E') && ! HasExp
            {
                HasExp = true;
                Result.push(self.Advance().unwrap());
                if self.Peek() == Some('+') || self.Peek() == Some('-')
                { Result.push(self.Advance().unwrap()); }
            }
            else
            { break ; }
        }
        Result
    }

    fn ScanIdentifier( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek()
        { if Ch.is_alphanumeric() || Ch == '_' || Ch == '$' { Result.push(self.Advance().unwrap()); } else { break ; } }
        if matches!(Result.as_str(), "sync" | "async" | "yield")
        {
            let mut Offset = 0;
            while self.PeekAhead(Offset) == Some(' ') || self.PeekAhead(Offset) == Some('\t')
            { Offset += 1; }
            if self.PeekAhead(Offset) == Some('*')
            { for _ in 0..=Offset { self.Advance(); } Result.push('*'); }
        }
        Result
    }

    fn ScanOperator( & mut self) -> Option<String>
    {
        let Ch = self.Peek()?;
        let N1 = self.PeekAhead(1);
        let N2 = self.PeekAhead(2);

        // 3-character operators
        if Ch == '>' && N1 == Some('>') && N2 == Some('>')
        {
            if self.PeekAhead(3) == Some('=')
            {
                self.Advance();
                self.Advance();
                self.Advance();
                self.Advance();
                return Some(">>>=".to_string());
            }
            self.Advance();
            self.Advance();
            self.Advance();
            return Some(">>>".to_string());
        }
        if Ch == '~' && N1 == Some('/') && N2 == Some('=')
        { self.Advance(); self.Advance(); self.Advance(); return Some("~/=".to_string()); }

        // 2-character operators
        if Ch == '=' && N1 == Some('>')
        { self.Advance(); self.Advance(); return Some("=>".to_string()); }
        if Ch == '?' && N1 == Some('?')
        {
            if N2 == Some('=')
            { self.Advance(); self.Advance(); self.Advance(); return Some("??=".to_string()); }
            self.Advance();
            self.Advance();
            return Some("??".to_string());
        }
        if Ch == '?' && N1 == Some('.')
        { self.Advance(); self.Advance(); return Some("?.".to_string()); }
        if Ch == '~' && N1 == Some('/')
        { self.Advance(); self.Advance(); return Some("~/".to_string()); }
        if (Ch == '+' || Ch == '-' || Ch == '*' || Ch == '/' || Ch == '%' || Ch == '&' || Ch == '|' || Ch == '^' || Ch == '<' || Ch == '>') && N1 == Some('=')
        {
            self.Advance();
            self.Advance();
            return Some(format ! ("{}=", Ch));
        }
        if Ch == '=' && N1 == Some('=')
        { self.Advance(); self.Advance(); return Some("==".to_string()); }
        if Ch == '!' && N1 == Some('=')
        { self.Advance(); self.Advance(); return Some("!=".to_string()); }
        if Ch == '&' && N1 == Some('&')
        { self.Advance(); self.Advance(); return Some("&&".to_string()); }
        if Ch == '|' && N1 == Some('|')
        { self.Advance(); self.Advance(); return Some("||".to_string()); }
        if Ch == '<' && N1 == Some('<')
        {
            if N2 == Some('=')
            { self.Advance(); self.Advance(); self.Advance(); return Some("<<=".to_string()); }
            self.Advance();
            self.Advance();
            return Some("<<".to_string());
        }
        if Ch == '>' && N1 == Some('>')
        {
            if N2 == Some('=')
            { self.Advance(); self.Advance(); self.Advance(); return Some(">>=".to_string()); }
            self.Advance();
            self.Advance();
            return Some(">>".to_string());
        }
        if Ch == '+' && N1 == Some('+')
        { self.Advance(); self.Advance(); return Some("++".to_string()); }
        if Ch == '-' && N1 == Some('-')
        { self.Advance(); self.Advance(); return Some("--".to_string()); }

        // Single-character operators
        if matches!(Ch, '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' | '!' | '=' | '<' | '>' | '?' | '@' | '.')
        {
            self.Advance();
            return Some(Ch.to_string());
        }

        None
    }

    pub fn IsKeyword(S: & str) -> bool
    {
        matches!
        (
            S,
            "abstract" | "as" | "assert" | "async" | "async*" | "await" | "base" | "break"
            | "case" | "catch" | "class" | "const" | "continue" | "covariant" | "default"
            | "deferred" | "do" | "dynamic" | "else" | "enum" | "export" | "extends"
            | "extension" | "external" | "factory" | "final" | "finally" | "for" | "Function"
            | "get" | "hide" | "if" | "implements" | "import" | "in" | "interface" | "is"
            | "late" | "library" | "mixin" | "new" | "null" | "of" | "on" | "operator"
            | "part" | "required" | "rethrow" | "return" | "sealed" | "set" | "show"
            | "static" | "super" | "switch" | "sync*" | "this" | "throw" | "true" | "false"
            | "try" | "type" | "typedef" | "var" | "void" | "when" | "while" | "with"
            | "yield" | "yield*"
        )
    }

    fn ClassifyAngleGenerics(Tokens: & mut Vec<Token_Info>)
    {
        let mut I = 0;
        while I < Tokens.len()
        {
            if Tokens[I].Text != "<" { I += 1; continue ; }

            // Preceding token check: valid type identifiers or constructors/keywords preceding <
            let PrecedingValid = if I == 0 { false }
            else
            {
                let Prev = & Tokens[I - 1];
                let Text = Prev.Text.as_str();
                matches!(Text, "=" | "=>" | ":" | "," | "(" | "[" | "{" | "??" | "const" | "new" | "Function")
                || matches!(Text, "class" | "enum" | "mixin" | "extension" | "typedef")
                || matches!(Text, "int" | "double" | "num" | "bool" | "dynamic" | "void" | "never" | "symbol" | "Object" | "Comparable" | "Future" | "FutureOr" | "Stream")
                || matches!(Prev.Kind, Token_Kind::Identifier(_))
            };

            if ! PrecedingValid { I += 1; continue ; }

            // Scan forward for matching '>'
            let mut Depth: usize = 0;
            let mut MatchIndex: Option < usize > = None;
            let mut J = I;
            let mut ContainsInvalidTypeToken = false;
            let mut ParenDepth: usize = 0;

            while J < Tokens.len()
            {
                if Tokens[J].Text == "("
                { ParenDepth += 1; }
                else if Tokens[J].Text == ")"
                { if ParenDepth > 0 { ParenDepth -= 1; } }
                else if Tokens[J].Text == "<"
                { Depth += 1; }
                else if Tokens[J].Text == ">"
                { Depth -= 1; if Depth == 0 && ParenDepth == 0 { MatchIndex = Some(J); break ; } }
                else if Tokens[J].Text.chars().all( | c | c == '>') && Tokens[J].Text.len() > 1 && Depth >= 1
                {
                    // Closer splitting per LAFS edge-cases §A.1 (handles >>, >>>, >>>>)
                    let Count = Tokens[J].Text.len();
                    let Tok = Tokens[J].clone();
                    Tokens[J].Kind = Token_Kind::Operator(">".to_string());
                    Tokens[J].Text = ">".to_string();
                    for Offset in 1..Count
                    {
                        Tokens.insert(J + Offset, Token_Info::New(Token_Kind::Operator(">".to_string()), ">".to_string(), Tok.Line, Tok.Column + Offset, 0));
                    }
                    Depth -= 1;
                    if Depth == 0 && ParenDepth == 0 { MatchIndex = Some(J); break ; }
                }
                else if Tokens[J].Text == ";" ||
                (
                    ParenDepth == 0 && matches!(Tokens[J].Text.as_str(), "{" | "}" | "||" | "&&" | "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<=" | ">=" | "if" | "while" | "return" | "yield")
                )
                { ContainsInvalidTypeToken = true; break ; }
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
                        "(" | ")" | "]" | "}" | ":" | ";" | "," | "." | "?" | "[" | "{"
                        | ">" | "extends" | "implements" | "with" | "on" | "when" | "as" | "is" | "async" | "="
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
}

// ============================================================================
// DART SYNTAX TREE & CONTAINER NODE
// ============================================================================

#[derive(Debug, Clone)]
pub enum Dart_Syntax_Node
{ Token(Token_Info), Container(Dart_Container_Node), }

#[derive(Debug, Clone)]
pub struct Dart_Container_Node
{
    pub Kind: Container_Kind,
    pub Open_Token: Token_Info,
    pub Close_Token: Option<Token_Info>,
    pub Children: Vec<Dart_Syntax_Node>,
    pub Tail_Glue: Vec<Token_Info>,
    pub Has_Forced_Break: bool,
    pub Statement_Count: usize,
    pub Max_Child_Depth: usize,
    pub Is_1Tuple_Record: bool,
    pub Is_IIFE: bool,
    pub IIFE_Modifier: Option<String>,
    pub IIFE_Has_Invocation: bool,
    pub Is_Control_Header: bool,
}

impl Dart_Container_Node
{
    pub fn New(Kind: Container_Kind, Open_Token: Token_Info) -> Self
    {
        Dart_Container_Node
        {
            Kind,
            Open_Token,
            Close_Token: None,
            Children: Vec:: new (),
            Tail_Glue: Vec:: new (),
            Has_Forced_Break: false,
            Statement_Count: 0,
            Max_Child_Depth: 0,
            Is_1Tuple_Record: false,
            Is_IIFE: false,
            IIFE_Modifier: None,
            IIFE_Has_Invocation: false,
            Is_Control_Header: false,
        }
    }

    pub fn GetSoleChildContainer( & self) -> Option< & Dart_Container_Node>
    {
        if self.Children.len() == 1
        {
            if let Dart_Syntax_Node::Container(ref C) = self.Children[0]
            { if C.Tail_Glue.is_empty() && self.Close_Token.is_some() && C.Close_Token.is_some() { return Some(C); } }
        }
        else if self.Is_1Tuple_Record && self.Children.len() == 2
        {
            if let Dart_Syntax_Node::Container(ref C) = self.Children[0]
            { if self.Close_Token.is_some() && C.Close_Token.is_some() { return Some(C); } }
        }
        None
    }

    pub fn CollectChain( & self) -> Vec< & Dart_Container_Node>
    {
        let mut Chain = vec![self];
        let mut Current = self;
        while let Some(Child) = Current.GetSoleChildContainer()
        {
            if Current.Kind == Container_Kind::Angle || Child.Kind == Container_Kind::Angle
            { break ; }
            Chain.push(Child);
            Current = Child;
        }
        Chain
    }

    pub fn CalculateBlockDepth( & self) -> usize
    {
        let mut MaxChildDepth = 0;
        for Child in & self.Children
        {
            if let Dart_Syntax_Node::Container(ref C) = Child
            {
                let D = C.CalculateBlockDepth();
                if D > MaxChildDepth { MaxChildDepth = D; }
            }
        }
        if self.Kind == Container_Kind::Brace { 1 + MaxChildDepth } else { MaxChildDepth }
    }

    pub fn HasNoExpansionInside( & self) -> bool
    {
        if self.Has_Forced_Break { return false; }
        if self.Statement_Count > 1 { return false; }
        for Child in & self.Children
        {
            if let Dart_Syntax_Node::Container(ref C) = Child
            {
                if C.Has_Forced_Break { return false; }
                if C.Open_Token.Preceding_Newlines > 0 && C.Kind == Container_Kind::Angle { return false; }
                if ! C.HasNoExpansionInside() { return false; }
            }
        }
        true
    }

    pub fn CalculateSingleLineSpan( & self) -> usize
    {
        if self.Has_Forced_Break { return usize::MAX; }
        if self.Kind != Container_Kind::Angle && ! (self.Is_Control_Header && self.HasNoExpansionInside()) && (self.Statement_Count > 4 || self.Max_Child_Depth > 2)
        { return usize::MAX; }

        let mut Total = self.Kind.OpenGlyph().len() + self.Kind.CloseGlyph().len();
        if self.Kind == Container_Kind::Brace && ! self.Children.is_empty()
        {
            Total += 2;
            // "{ ... }"
        }

        for (I, Child) in self.Children.iter().enumerate()
        {
            if I > 0 { Total += 1; }
            match Child
            {
                Dart_Syntax_Node::Token(T) =>
                { if T.Text.contains('\n') { return usize::MAX; } Total += T.Text.len(); }
                Dart_Syntax_Node::Container(C) =>
                {
                    let ChildSpan = C.CalculateSingleLineSpan();
                    if ChildSpan == usize::MAX { return usize::MAX; }
                    Total += ChildSpan;
                }
            }
        }

        if self.IIFE_Has_Invocation
        { Total += 2; }

        for Glue in & self.Tail_Glue
        { Total += Glue.Text.len(); }

        Total
    }
}

// ============================================================================
// DART CONTAINER TREE BUILDER
// ============================================================================

pub struct Dart_Container_Builder
{ Tokens: Vec<Token_Info>, Index: usize, }

impl Dart_Container_Builder
{
    pub fn New(Tokens: Vec<Token_Info>) -> Self
    { Dart_Container_Builder { Tokens, Index: 0 } }

    pub fn BuildTree( & mut self) -> Vec<Dart_Syntax_Node>
    {
        let mut RootNodes = Vec:: new ();
        while self.Index < self.Tokens.len()
        { if let Some(Node) = self.ParseNode() { RootNodes.push(Node); } }
        RootNodes
    }

    fn ParseNode( & mut self) -> Option<Dart_Syntax_Node>
    {
        if self.Index >= self.Tokens.len() { return None; }
        let Current = self.Tokens[self.Index].clone();

        match Current.Kind
        {
            Token_Kind::Open_Delim(Kind) =>
            {
                self.Index += 1;
                let mut Container = Dart_Container_Node::New(Kind, Current);

                if Kind == Container_Kind::Paren && self.Index >= 2
                {
                    for PrevIdx in (0..=self.Index - 2).rev()
                    {
                        if ! self.Tokens[PrevIdx].IsTrivia()
                        {
                            let PrevText = & self.Tokens[PrevIdx].Text;
                            if matches!(PrevText.as_str(), "for" | "if" | "while" | "catch" | "switch" | "when")
                            {
                                Container.Is_Control_Header = true;
                            }
                            break ;
                        }
                    }
                }

                while self.Index < self.Tokens.len()
                {
                    let Next = & self.Tokens[self.Index];
                    if let Token_Kind::Close_Delim(CloseKind) = Next.Kind
                    { if CloseKind == Kind { Container.Close_Token = Some(self.Tokens[self.Index].clone()); self.Index += 1; break ; } }

                    // Forced break conditions
                    if matches!(Next.Kind, Token_Kind::Comment_Line(_)) { Container.Has_Forced_Break = true; }
                    if Next.Text.contains('\n') { Container.Has_Forced_Break = true; }
                    if matches!(Next.Text.as_str(), ";" | "if" | "for" | "while" | "switch" | "try" | "yield" | "yield*" | "return")
                    {
                        Container.Statement_Count += 1;
                    }
                    else if Next.Text == "=>"
                    { Container.Statement_Count += 2; }

                    if let Some(ChildNode) = self.ParseNode()
                    {
                        if let Dart_Syntax_Node::Container(ref ChildC) = ChildNode
                        {
                            if ChildC.Has_Forced_Break { Container.Has_Forced_Break = true; }
                            for Glue in & ChildC.Tail_Glue { if Glue.Text == ";" { Container.Statement_Count += 1; } }
                            Container.Statement_Count += ChildC.Statement_Count;
                            Container.Max_Child_Depth = Container.Max_Child_Depth.max(ChildC.Max_Child_Depth + 1);
                        }
                        Container.Children.push(ChildNode);
                    }
                }

                // Check for 1-tuple record: (expression,) or (container,)
                if Kind == Container_Kind::Paren
                {
                    if Container.Children.len() == 2
                    {
                        if let Dart_Syntax_Node::Token(ref T) = Container.Children[1]
                        { if T.Text == "," { Container.Is_1Tuple_Record = true; } }
                    }
                    else if Container.Children.len() == 1
                    {
                        if let Dart_Syntax_Node::Container(ref mut C) = Container.Children[0]
                        { if C.Tail_Glue.len() == 1 && C.Tail_Glue[0].Text == "," { C.Tail_Glue.pop(); Container.Is_1Tuple_Record = true; } }
                    }

                    // Check if Container is an IIFE:
                    // Child 0 is a Paren container (parameters)
                    // followed by optional modifiers (async, sync*, =>)
                    // followed by a body (Brace, or another container)
                    let mut Is_Closure = false;
                    let mut Modifier = None;
                    if ! Container.Children.is_empty()
                    {
                        if let Dart_Syntax_Node::Container(ref ParamC) = Container.Children[0]
                        {
                            if ParamC.Kind == Container_Kind::Paren
                            {
                                let mut ChildIdx = 1;
                                while ChildIdx < Container.Children.len()
                                {
                                    if let Dart_Syntax_Node::Token(ref T) = Container.Children[ChildIdx]
                                    { if matches ! (T.Text.as_str(), "async" | "sync*" | "async*" | "=>") { Modifier = Some(T.Text.clone()); ChildIdx += 1; continue ; } }
                                    break;
                                }
                                if ChildIdx < Container.Children.len()
                                { if let Dart_Syntax_Node::Container(_) = Container.Children[ChildIdx] { Is_Closure = true; } }
                            }
                        }
                    }

                    if Is_Closure
                    {
                        Container.Is_IIFE = true;
                        Container.IIFE_Modifier = Modifier;

                        if self.Index + 1 < self.Tokens.len()
                        {
                            let T0 = & self.Tokens[self.Index];
                            let T1 = & self.Tokens[self.Index + 1];
                            if T0 .Text == "(" && T1 .Text == ")" && T0 .Preceding_Newlines == 0
                            {
                                Container.IIFE_Has_Invocation = true;
                                self.Index += 2;
                            }
                        }
                    }
                }

                // Collect tail glue (';', ',')
                while self.Index < self.Tokens.len()
                {
                    let Next = & self.Tokens[self.Index];
                    if Next.Preceding_Newlines > 0 { break ; }
                    if Next.Text == ";" || Next.Text == ","
                    { Container.Tail_Glue.push(self.Tokens[self.Index].clone()); self.Index += 1; }
                    else
                    { break ; }
                }

                Some(Dart_Syntax_Node::Container(Container))
            }
            _ =>
            { self.Index += 1; Some(Dart_Syntax_Node::Token(Current)) }
        }
    }
}

// ============================================================================
// DART EMITTER (100% LAFS Compliance, Monolith First, Maximal Grouping)
// ============================================================================

fn OutputEndsWithControlKeyword(Output: & str) -> bool
{
    let ControlKeywords = ["if", "while", "for", "switch", "catch", "assert", "when", "case", "return", "yield", "await", "sync*", "async*",];
    let Trimmed = Output.trim_end();
    for Kw in ControlKeywords
    {
        if Trimmed.ends_with(Kw)
        {
            let PrefixLen = Trimmed.len() - Kw.len();
            if PrefixLen == 0 { return true; }
            if let Some(Ch) = Trimmed[..PrefixLen].chars().last()
            { if ! Ch.is_alphanumeric() && Ch != '_' && Ch != '$' { return true; } }
        }
    }
    false
}

fn IsTypeNameToken(text: & str) -> bool
{
    matches!(text, "int" | "double" | "num" | "bool" | "dynamic" | "void" | "never" | "symbol" | "Object" | "Comparable" | "Future" | "FutureOr" | "Stream")
    || text.starts_with( | c: char | c.is_uppercase())
    || text.ends_with('>')
    || text.ends_with(')')
}

pub struct Dart_Emitter<'a>
{
    Config: & 'a Config_Settings,
    Output: String,
    Current_Line_Len: usize,
    Line_Has_Content: bool,
    In_Cascade: bool,
    In_Directive: bool,
    In_Typedef: bool,
    In_Ternary: usize,
}

impl <'a> Dart_Emitter<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self
    {
        Dart_Emitter
        {
            Config,
            Output: String:: new (),
            Current_Line_Len: 0,
            Line_Has_Content: false,
            In_Cascade: false,
            In_Directive: false,
            In_Typedef: false,
            In_Ternary: 0,
        }
    }

    pub fn Emit(mut self, Nodes: & [Dart_Syntax_Node]) -> String
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

    fn EnsureNewline( & mut self)
    { if self.Line_Has_Content { self.WriteNewline(); } }

    fn EnsureBlankLine( & mut self)
    {
        if self.Config.Max_Blank_Lines == 0 || self.Output.is_empty() { return ; }
        self.EnsureNewline();
        if ! self.Output.ends_with("\n\n")
        && ! self.Output.ends_with("{\n")
        && ! self.Output.ends_with("[\n")
        && ! self.Output.ends_with("(\n")
        { self.Output.push('\n'); self.Current_Line_Len = 0; self.Line_Has_Content = false; }
    }

    fn EmitIndent( & mut self, Level: usize)
    {
        if ! self.Line_Has_Content
        {
            let IndentStr = self.Config.IndentString(Level);
            self.Output.push_str( & IndentStr);
            self.Current_Line_Len = IndentStr.len();
            self.Line_Has_Content = true;
        }
    }

    fn EmitRaw( & mut self, Text: & str)
    {
        self.Output.push_str(Text);
        if let Some(LastNl) = Text.rfind('\n')
        {
            self.Current_Line_Len = Text.len() - LastNl - 1;
            self.Line_Has_Content = self.Current_Line_Len > 0;
        }
        else
        { self.Current_Line_Len += Text.len(); self.Line_Has_Content = true; }
    }

    fn EmitSpace( & mut self)
    {
        if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
        {
            self.Output.push(' ');
            self.Current_Line_Len += 1;
        }
    }

    fn EmitNodeList( & mut self, Nodes: & [Dart_Syntax_Node], Level: usize)
    {
        let SavedCascade = self.In_Cascade;
        let mut InForOrIfElement = false;

        for (I, Node) in Nodes.iter().enumerate()
        {
            match Node
            {
                Dart_Syntax_Node::Token(T) =>
                {
                    if T.Text == ".." || T.Text == "?.."
                    { self.In_Cascade = true; }
                    else if T.Text == "for" || T.Text == "if"
                    { InForOrIfElement = false; }
                    else if T.Text == "typedef"
                    { self.In_Typedef = true; }

                    let EffectiveLevel = if self.In_Cascade || (self.In_Typedef && (T.Text == "Function" || T.Text == "async" || T.Text == "sync*"))
                    { Level + 1 }
                    else { Level };
                    let NodeLevel = if InForOrIfElement { EffectiveLevel + 1 } else { EffectiveLevel };

                    if InForOrIfElement
                    { self.EnsureNewline(); }

                    self.EmitSingleToken(T, NodeLevel, Nodes, I);

                    if T.Text == ";" || T.Text == ","
                    {
                        if T.Text == ";" { self.In_Cascade = false; self.In_Typedef = false; }
                        InForOrIfElement = false;
                    }
                }
                Dart_Syntax_Node::Container(C) =>
                {
                    let EffectiveLevel = if self.In_Cascade { Level + 1 } else { Level };
                    let NodeLevel = if InForOrIfElement { EffectiveLevel + 1 } else { EffectiveLevel };

                    if InForOrIfElement
                    { self.EnsureNewline(); }
                    self.EmitContainerNode(C, NodeLevel);

                    if C.Tail_Glue.iter().any( | g | g.Text == ";")
                    { self.In_Cascade = false; self.In_Typedef = false; }

                    if InForOrIfElement
                    { InForOrIfElement = false; }
                    else if C.Kind == Container_Kind::Paren && I > 0
                    {
                        if let Dart_Syntax_Node::Token(ref PrevT) = Nodes[I - 1]
                        {
                            if PrevT.Text == "for" || PrevT.Text == "if"
                            {
                                if I + 1 < Nodes.len()
                                {
                                    match & Nodes[I + 1]
                                    {
                                        Dart_Syntax_Node::Container(_) => {}
                                        Dart_Syntax_Node::Token(ref NextT) =>
                                        { if ! matches ! (NextT.Text.as_str(), "switch" | "..." | "...?" | "for" | "if") { InForOrIfElement = true; } }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Sibling statement and enum element separation in expanded blocks
            let IsStatementOrDecl = match Node
            {
                Dart_Syntax_Node::Token(T) => T.Text == ";" || T.Text == ",",
                Dart_Syntax_Node::Container(C) =>
                { C.Open_Token.Text != "${" && (C.Tail_Glue.iter().any( | g | g.Text == ";" || g.Text == ",") || C.Kind == Container_Kind::Brace) }
            };

            if IsStatementOrDecl && I + 1 < Nodes.len()
            { if Level > 0 { self.EnsureNewline(); } }

            // Annotation completion (e.g. @override or @Deprecated(...))
            let IsAnnotationComplete = match Node
            {
                Dart_Syntax_Node::Token(_) =>
                {
                    if I > 0
                    {
                        if let Dart_Syntax_Node::Token(ref PrevT) = Nodes[I - 1]
                        {
                            if PrevT.Text == "@"
                            { if I + 1 < Nodes.len() { ! matches ! (Nodes[I + 1], Dart_Syntax_Node::Container(ref C) if C.Kind == Container_Kind::Paren) } else { true } }
                            else
                            { false }
                        }
                        else
                        { false }
                    }
                    else
                    { false }
                }
                Dart_Syntax_Node::Container(C) =>
                {
                    if C.Kind == Container_Kind::Paren && I >= 2
                    { if let Dart_Syntax_Node::Token(ref PrevPrevT) = Nodes[I - 2] { PrevPrevT.Text == "@" } else { false } }
                    else
                    { false }
                }
            };

            if IsAnnotationComplete && I + 1 < Nodes.len()
            { self.EnsureNewline(); }
        }
        self.In_Cascade = SavedCascade;
    }

    fn EmitSingleToken( & mut self, Token: & Token_Info, Level: usize, _Nodes: & [Dart_Syntax_Node], _Index: usize)
    {
        // 1. Directives (import, export, library, part)
        if Token.Text == "import" || Token.Text == "export" || Token.Text == "library" || Token.Text == "part"
        { self.In_Directive = true; }

        // 2. Preceding newlines preservation
        if Token.Preceding_Newlines > 0
        {
            if self.In_Directive
            { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
            else
            {
                let Trimmed = self.Output.trim_end();
                let IsAfterFatArrow = self.Line_Has_Content && Trimmed.ends_with("=>");
                let IsSignatureBreak = self.Line_Has_Content && match & Token.Kind
                {
                    Token_Kind::Identifier(_) =>
                    {
                        Trimmed.ends_with("void") || Trimmed.ends_with("int") || Trimmed.ends_with("String") || Trimmed.ends_with("double")
                        || Trimmed.ends_with("bool") || Trimmed.ends_with("get") || Trimmed.ends_with("set") || Trimmed.ends_with("final")
                        || Trimmed.ends_with("static") || Trimmed.ends_with("const") || Trimmed.ends_with("factory")
                        || Trimmed.ends_with('>') || Trimmed.ends_with('?')
                    }
                    _ => false,
                };

                if IsSignatureBreak || (IsAfterFatArrow && self.Current_Line_Len + 1 + Token.Text.len() <= self.Config.Canvas_Width)
                { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                else
                { self.EnsureNewline(); if Token.Preceding_Newlines >= 2 { self.EnsureBlankLine(); } }
            }
        }

        // 3. Line comment
        if let Token_Kind::Comment_Line(CommentText) = & Token.Kind
        {
            if self.Line_Has_Content { self.EmitSpace(); } else { self.EmitIndent(Level); }
            self.EmitRaw(CommentText);
            self.WriteNewline();
            return ;
        }

        // 4. Block comment
        if let Token_Kind::Comment_Block(CommentText) = & Token.Kind
        {
            if ! self.Line_Has_Content { self.EmitIndent(Level); } else { self.EmitSpace(); }
            self.EmitRaw(CommentText);
            return ;
        }

        // 5. Cascade notation (.. or ?..)
        if Token.Text == ".." || Token.Text == "?.."
        {
            if Token.Preceding_Newlines > 0
            { self.EnsureNewline(); self.EmitIndent(Level); }
            self.EmitRaw( & Token.Text);
            return ;
        }

        // 6. Dot notation (. or ?.)
        if Token.Text == "." || Token.Text == "?."
        {
            let Trimmed = self.Output.trim_end();
            if Trimmed.ends_with( | c: char | c.is_alphanumeric() || c == '_' || c == '$' || c == '>' || c == ')' || c == ']' || c == '\'' || c == '"')
            { while self.Output.ends_with(' ') { self.Output.pop(); self.Current_Line_Len -= 1; } }
            self.EmitRaw( & Token.Text);
            return ;
        }

        // 7. Operators, Identifiers, Keywords
        if ! self.Line_Has_Content { self.EmitIndent(Level); }

        match & Token.Kind
        {
            Token_Kind::Keyword(K) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with('@') && ! self.Output.ends_with('?')
                { self.EmitSpace(); }
                self.EmitRaw(K);
                let NextIsParenOrAngleOrDot = _Index + 1 < _Nodes.len() && match & _Nodes[_Index + 1]
                {
                    Dart_Syntax_Node::Token(NextT) => NextT.Text == "(" || NextT.Text == "<" || NextT.Text == "." || NextT.Text == "?.",
                    Dart_Syntax_Node::Container(NextC) => NextC.Kind == Container_Kind::Paren || NextC.Kind == Container_Kind::Angle,
                };
                if (K == "this" || K == "super") && NextIsParenOrAngleOrDot
                {
                    // No space after `this` or `super` before `.` or `(`
                }
                else if K != "Function" || ! NextIsParenOrAngleOrDot
                { self.EmitSpace(); }
            }
            Token_Kind::Identifier(Ident) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with('.') && ! self.Output.ends_with("..") && ! self.Output.ends_with("?..")
                && ! self.Output.ends_with("?.") && ! self.Output.ends_with('@') && ! self.Output.ends_with('?')
                { self.EmitSpace(); }
                self.EmitRaw(Ident);
            }
            Token_Kind::Number(Num) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with(':') && ! self.Output.ends_with('-')
                { self.EmitSpace(); }
                self.EmitRaw(Num);
            }
            Token_Kind::Operator(Op) =>
            {
                if matches!
                (
                    Op.as_str(), "=>" | "=" | "==" | "!=" | "&&" | "||" | "+" | "-" | "*" | "/" | "~/" | "%" | "+=" | "-=" | "*=" | "/=" | "~/=" | "%=" | "??=" | "??" | "?:" | "<=" | ">=" | "<" | ">" | "|" | "&" | "^" | "<<" | ">>" | ">>>"
                )
                {
                    if self.Line_Has_Content
                    && ! self.Output.ends_with(' ')
                    && ! self.Output.ends_with('\n')
                    && ! self.Output.ends_with('(')
                    && ! self.Output.ends_with('[')
                    && ! self.Output.ends_with('{')
                    { self.EmitSpace(); }
                    self.EmitRaw(Op);
                    if Op == "=>"
                    {
                        if _Index + 1 < _Nodes.len()
                        {
                            if let Dart_Syntax_Node::Token(ref NextT) = _Nodes[_Index + 1]
                            { if NextT.Preceding_Newlines > 0 { self.WriteNewline(); self.EmitIndent(Level + 1); return ; } }
                        }
                    }
                    self.EmitSpace();
                }
                else if Op == "..." || Op == "...?" || Op == "!" || Op == "~" || Op == "@"
                { self.EmitRaw(Op); }
                else if Op == "?"
                {
                    let PrecededByDelimiter = self.Output.ends_with('(') || self.Output.ends_with('[') || self.Output.ends_with('{') || self.Output.ends_with(',') || self.Output.ends_with(':')
                    ||
                    (
                        self.Output.ends_with(' ') &&
                        {
                            let Trimmed = self.Output.trim_end();
                            Trimmed.ends_with('(') || Trimmed.ends_with('[') || Trimmed.ends_with('{') || Trimmed.ends_with(',') || Trimmed.ends_with(':')
                        }
                    );

                    if PrecededByDelimiter
                    { if self.Output.ends_with(',') { self.EmitSpace(); } self.EmitRaw("?"); }
                    else
                    {
                        let PrecedingIsType = _Index > 0 && match & _Nodes[_Index - 1]
                        {
                            Dart_Syntax_Node::Token(prev_t) => IsTypeNameToken( & prev_t.Text),
                            Dart_Syntax_Node::Container(prev_c) => prev_c.Kind == Container_Kind::Angle || prev_c.Kind == Container_Kind::Paren,
                        };

                        let NextFollower = _Index + 1 < _Nodes.len();
                        let NextIsDeclOrCloser = if NextFollower
                        {
                            match & _Nodes[_Index + 1]
                            {
                                Dart_Syntax_Node::Token(next_t) =>
                                {
                                    next_t.Text == ">" || next_t.Text == "," || next_t.Text == ")" || next_t.Text == "Function"
                                    || matches!(next_t.Kind, Token_Kind::Identifier(_))
                                }
                                Dart_Syntax_Node::Container(next_c) => next_c.Kind == Container_Kind::Angle,
                            }
                        }
                        else { false };

                        let IsNullableType = PrecedingIsType && NextIsDeclOrCloser;

                        if IsNullableType
                        {
                            while self.Output.ends_with(' ') { self.Output.pop(); self.Current_Line_Len -= 1; }
                            self.EmitRaw("?");
                            let NextIsCloser = _Index + 1 < _Nodes.len() && match & _Nodes[_Index + 1] { Dart_Syntax_Node::Token(next_t) => next_t.Text == ">" || next_t.Text == "," || next_t.Text == ")", _ => false, };
                            if ! NextIsCloser { self.EmitSpace(); }
                        }
                        else
                        {
                            if ! self.Output.ends_with(' ') { self.EmitSpace(); }
                            self.EmitRaw("?");
                            self.EmitSpace();
                            self.In_Ternary += 1;
                        }
                    }
                }
                else
                { self.EmitRaw(Op); }
            }
            Token_Kind::Punctuation(P) =>
            {
                if * P == ',' || * P == ';'
                {
                    if * P == ';' { self.In_Ternary = 0; }
                    while self.Output.ends_with(' ') { self.Output.pop(); }
                    self.EmitRaw( & P.to_string());
                    if * P == ';' && self.In_Directive
                    { self.In_Directive = false; self.WriteNewline(); }
                    else if Level > 0
                    { self.WriteNewline(); }
                    else
                    { self.EmitSpace(); }
                }
                else if * P == ':'
                {
                    if self.In_Ternary > 0
                    {
                        if ! self.Output.ends_with(' ') { self.EmitSpace(); }
                        self.EmitRaw(":");
                        self.EmitSpace();
                        self.In_Ternary -= 1;
                    }
                    else
                    {
                        let IsPatternGetter = self.Output.ends_with('(') || self.Output.ends_with('{')
                        ||
                        (
                            self.Output.ends_with(' ') &&
                            {
                                let Trimmed = self.Output.trim_end();
                                Trimmed.ends_with(',') || Trimmed.ends_with('(') || Trimmed.ends_with('{')
                            }
                        );
                        let NextIsIdentOrKeyword = _Index + 1 < _Nodes.len() && match & _Nodes[_Index + 1] { Dart_Syntax_Node::Token(t) => matches ! (t.Kind, Token_Kind::Identifier(_) | Token_Kind::Keyword(_)), _ => false, };

                        if IsPatternGetter && NextIsIdentOrKeyword
                        {
                            if ! self.Output.ends_with(' ') && ! self.Output.ends_with('(') && ! self.Output.ends_with('{')
                            { self.EmitSpace(); }
                            self.EmitRaw(":");
                        }
                        else
                        {
                            let Trimmed = self.Output.trim_end();
                            if Trimmed.ends_with("this") || Trimmed.ends_with(')')
                            { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                            else if Trimmed.ends_with('?') || self.Output.ends_with('?')
                            { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                            else
                            { while self.Output.ends_with(' ') { self.Output.pop(); } }
                            self.EmitRaw(":");
                            self.EmitSpace();
                        }
                    }
                }
                else
                { self.EmitRaw( & P.to_string()); }
            }
            Token_Kind::String_Lit(Text) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with('}')
                && ! self.Output.ends_with(':')
                { self.EmitSpace(); }
                self.EmitRaw(Text);
            }
            _ =>
            { self.EmitRaw( & Token.Text); }
        }
    }

    fn DetermineLayoutState( & self, Container: & Dart_Container_Node) -> Layout_State
    {
        // 1. Empty container -> State 1 Monolith
        if Container.Children.is_empty() { return Layout_State::Monolith; }

        // Angle containers: expanded when starting on a newline, otherwise monolith
        if Container.Kind == Container_Kind::Angle
        {
            if Container.Open_Token.Preceding_Newlines > 0
            { return Layout_State::Expanded; }
            return Layout_State::Monolith;
        }

        // 2. Monolith Check:
        // Must fit Canvas AND not exceed complexity budget AND have no forced breaks
        let WithinComplexity = Container.Statement_Count <= self.Config.Max_Statements_In_Monolith
        && (Container.Max_Child_Depth <= self.Config.Max_Block_Depth || (Container.Is_Control_Header && Container.HasNoExpansionInside()));

        let Span = Container.CalculateSingleLineSpan();
        if WithinComplexity && ! Container.Has_Forced_Break && (self.Current_Line_Len + Span <= self.Config.Canvas_Width)
        { return Layout_State::Monolith; }

        // 3. State 2: Container Grouping / Simplified Symmetry
        // If it's a Sole-Child Chain (continuous structural boundary run), group it!
        let Chain = Container.CollectChain();
        if Chain.len() > 1
        { return Layout_State::Grouped; }

        // 4. State 3: Expanded Symmetry (Hierarchical)
        Layout_State::Expanded
    }

    fn EmitContainerNode( & mut self, Container: & Dart_Container_Node, Level: usize)
    {
        if Container.Open_Token.Text == "${"
        { self.EmitStringInterpolation(Container, Level); return ; }

        if Container.Is_IIFE
        {
            let Layout = self.DetermineLayoutState(Container);
            if Layout == Layout_State::Monolith
            { self.EmitMonolith(Container, Level); }
            else
            { self.EmitIIFE(Container, Level); }
            return ;
        }

        let Layout = self.DetermineLayoutState(Container);

        match Layout
        {
            Layout_State::Monolith => self.EmitMonolith(Container, Level),
            Layout_State::Grouped => self.EmitGrouped(Container, Level),
            Layout_State::Expanded => self.EmitExpanded(Container, Level),
        }
    }

    fn EmitMonolith( & mut self, Container: & Dart_Container_Node, Level: usize)
    {
        if ! self.Line_Has_Content { self.EmitIndent(Level); }

        let NeedSpace = self.Line_Has_Content && self.Current_Line_Len > 0
        && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
        && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
        && ! self.Output.ends_with('@') && ! self.Output.ends_with('.') && ! self.Output.ends_with("..")
        && ! self.Output.ends_with("?..") && ! self.Output.ends_with("...") && ! self.Output.ends_with("...?")
        && ! self.Output.ends_with('?');

        if NeedSpace && (Container.Kind == Container_Kind::Brace || Container.Kind == Container_Kind::Angle) { self.EmitSpace(); }

        self.EmitRaw(Container.Kind.OpenGlyph());

        if Container.Kind == Container_Kind::Brace && ! Container.Children.is_empty()
        { self.EmitRaw(" "); }

        let mut PrevNeedsSpace = false;
        let mut In_Ternary: usize = 0;
        for (ChildIdx, Child) in Container.Children.iter().enumerate()
        {
            match Child
            {
                Dart_Syntax_Node::Token(T) =>
                {
                    let IsPunct = T.Text == ";"
                    || T.Text == ","
                    || T.Text == "."
                    || T.Text == "?."
                    || T.Text == ".."
                    || T.Text == "?.."
                    || T.Text == "++"
                    || T.Text == "--"
                    || T.Text == "!";

                    let OutputEndsWithNoSpace = self.Output.ends_with('.')
                    || self.Output.ends_with("?.")
                    || self.Output.ends_with("..")
                    || self.Output.ends_with("?..")
                    || self.Output.ends_with("...")
                    || self.Output.ends_with("...?")
                    || self.Output.ends_with('(')
                    || self.Output.ends_with('[')
                    || self.Output.ends_with('{')
                    || self.Output.ends_with('@')
                    || self.Output.ends_with('$')
                    || self.Output.ends_with('?');

                    if T.Text == ":"
                    {
                        if In_Ternary > 0
                        {
                            if ! self.Output.ends_with(' ') { self.EmitSpace(); }
                            self.EmitRaw(":");
                            self.EmitSpace();
                            In_Ternary -= 1;
                            PrevNeedsSpace = false;
                            continue;
                        }

                        let IsPatternGetter = self.Output.ends_with('(') || self.Output.ends_with('{')
                        ||
                        (
                            self.Output.ends_with(' ') &&
                            {
                                let Trimmed = self.Output.trim_end();
                                Trimmed.ends_with(',') || Trimmed.ends_with('(') || Trimmed.ends_with('{')
                            }
                        );
                        if IsPatternGetter
                        {
                            if self.Output.ends_with(',') { self.EmitSpace(); }
                            self.EmitRaw(":");
                            PrevNeedsSpace = false;
                            continue;
                        }
                        let Trimmed = self.Output.trim_end();
                        if Trimmed.ends_with("this") || Trimmed.ends_with(')')
                        { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                        else if Trimmed.ends_with('?') || self.Output.ends_with('?')
                        { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                        else
                        { while self.Output.ends_with(' ') { self.Output.pop(); } }
                        self.EmitRaw(":");
                        self.EmitSpace();
                        PrevNeedsSpace = false;
                        continue;
                    }

                    if T.Text == ";"
                    {
                        In_Ternary = 0;
                        while self.Output.ends_with(' ') { self.Output.pop(); self.Current_Line_Len -= 1; }
                        self.EmitRaw(";");
                        self.EmitSpace();
                        PrevNeedsSpace = false;
                        continue;
                    }

                    if T.Text == "." || T.Text == "?."
                    {
                        let Trimmed = self.Output.trim_end();
                        if Trimmed.ends_with( | c: char | c.is_alphanumeric() || c == '_' || c == '$' || c == '>' || c == ')' || c == ']' || c == '\'' || c == '"')
                        { while self.Output.ends_with(' ') { self.Output.pop(); self.Current_Line_Len -= 1; } }
                        self.EmitRaw( & T.Text);
                        PrevNeedsSpace = false;
                        continue;
                    }

                    if T.Text == "?"
                    {
                        let PrecededByDelimiter = self.Output.ends_with('(') || self.Output.ends_with('[') || self.Output.ends_with('{') || self.Output.ends_with(',') || self.Output.ends_with(':')
                        ||
                        (
                            self.Output.ends_with(' ') &&
                            {
                                let Trimmed = self.Output.trim_end();
                                Trimmed.ends_with('(') || Trimmed.ends_with('[') || Trimmed.ends_with('{') || Trimmed.ends_with(',') || Trimmed.ends_with(':')
                            }
                        );

                        if PrecededByDelimiter
                        {
                            if self.Output.ends_with(',') { self.EmitSpace(); }
                            self.EmitRaw("?");
                            PrevNeedsSpace = false;
                            continue;
                        }

                        let PrecedingIsType = ChildIdx > 0 && match & Container.Children[ChildIdx - 1]
                        {
                            Dart_Syntax_Node::Token(prev_t) => IsTypeNameToken( & prev_t.Text),
                            Dart_Syntax_Node::Container(prev_c) => prev_c.Kind == Container_Kind::Angle || prev_c.Kind == Container_Kind::Paren,
                        };

                        let NextIsCloser = if ChildIdx + 1 < Container.Children.len()
                        {
                            match & Container.Children[ChildIdx + 1] { Dart_Syntax_Node::Token(next_t) => next_t.Text == ">" || next_t.Text == "," || next_t.Text == ")", _ => false, }
                        }
                        else { Container.Close_Token.is_some() };

                        let NextIsDeclOrCloser = if ChildIdx + 1 < Container.Children.len()
                        {
                            match & Container.Children[ChildIdx + 1]
                            {
                                Dart_Syntax_Node::Token(next_t) =>
                                {
                                    next_t.Text == ">" || next_t.Text == "," || next_t.Text == ")" || next_t.Text == "Function"
                                    || matches!(next_t.Kind, Token_Kind::Identifier(_))
                                }
                                Dart_Syntax_Node::Container(next_c) => next_c.Kind == Container_Kind::Angle,
                            }
                        }
                        else { Container.Close_Token.is_some() };

                        let IsNullableType = PrecedingIsType && NextIsDeclOrCloser;

                        if IsNullableType
                        {
                            while self.Output.ends_with(' ') { self.Output.pop(); self.Current_Line_Len -= 1; }
                            self.EmitRaw("?");
                            if ! NextIsCloser { self.EmitSpace(); }
                            PrevNeedsSpace = false;
                            continue;
                        }
                        else
                        {
                            if ! self.Output.ends_with(' ') { self.EmitSpace(); }
                            self.EmitRaw("?");
                            self.EmitSpace();
                            In_Ternary += 1;
                            PrevNeedsSpace = false;
                            continue;
                        }
                    }

                    if PrevNeedsSpace && ! IsPunct && ! OutputEndsWithNoSpace
                    { self.EmitSpace(); }

                    // Binary operators inside Monolith
                    if matches!
                    (
                        T.Text.as_str(), "=>" | "=" | "==" | "!=" | "&&" | "||" | "+" | "-" | "*" | "/" | "~/" | "%" | "+=" | "-=" | "*=" | "/=" | "~/=" | "%=" | "??=" | "??" | "<=" | ">=" | "<" | ">" | "|" | "&" | "^"
                    )
                    { if ! self.Output.ends_with(' ') && ! OutputEndsWithNoSpace { self.EmitSpace(); } }

                    self.EmitRaw( & T.Text);

                    if T.Text == "," || T.Text == ";"
                    { self.EmitSpace(); PrevNeedsSpace = false; }
                    else if matches!
                    (
                        T.Text.as_str(), "=>" | "=" | "==" | "!=" | "&&" | "||" | "+" | "-" | "*" | "/" | "~/" | "%" | "+=" | "-=" | "*=" | "/=" | "~/=" | "%=" | "??=" | "??" | "<=" | ">=" | "<" | ">" | "|" | "&" | "^"
                    )
                    { self.EmitSpace(); PrevNeedsSpace = false; }
                    else if Dart_Lexer::IsKeyword( & T.Text)
                    {
                        if T.Text == "Function" { PrevNeedsSpace = true; }
                        else if T.Text == "this" || T.Text == "super" { PrevNeedsSpace = false; self.EmitSpace(); }
                        else { self.EmitSpace(); PrevNeedsSpace = false; }
                    }
                    else
                    {
                        PrevNeedsSpace = ! matches!(T.Text.as_str(), "." | "?." | ".." | "?.." | "..." | "...?" | "(" | "[" | "!" | "~" | "@" | "$" | "?");
                    }
                }
                Dart_Syntax_Node::Container(Sub) =>
                {
                    if Sub.Kind == Container_Kind::Brace
                    {
                        if PrevNeedsSpace || ( ! self.Output.ends_with(' ') && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{'))
                        {
                            self.EmitSpace();
                        }
                    }
                    else if Sub.Kind == Container_Kind::Paren
                    {
                        if OutputEndsWithControlKeyword( & self.Output)
                        || self.Output.ends_with('=')
                        || self.Output.ends_with("=>")
                        || self.Output.ends_with(':')
                        || self.Output.ends_with(',')
                        || self.Output.ends_with("||")
                        || self.Output.ends_with("&&")
                        { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                    }
                    else if Sub.Kind == Container_Kind::Bracket
                    {
                        if self.Output.ends_with('=')
                        || self.Output.ends_with("=>")
                        || self.Output.ends_with(':')
                        || self.Output.ends_with(',')
                        { if ! self.Output.ends_with(' ') { self.EmitSpace(); } }
                    }
                    else if Sub.Kind == Container_Kind::Angle
                    {
                        if self.Output.ends_with("??") { self.EmitSpace(); }
                        else if ! self.Output.ends_with(' ') && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{') && ! self.Output.ends_with('<') && ! self.Output.ends_with('@') && ! self.Output.ends_with('.')
                        {
                            self.EmitSpace();
                        }
                    }

                    self.EmitMonolith(Sub, Level);
                    PrevNeedsSpace = true;
                }
            }
        }

        if Container.Kind == Container_Kind::Brace && ! Container.Children.is_empty() && ! self.Output.ends_with(' ')
        { self.EmitRaw(" "); }

        if Container.Close_Token.is_some()
        {
            if (Container.Kind == Container_Kind::Paren || Container.Kind == Container_Kind::Bracket || Container.Kind == Container_Kind::Angle)
            && self.Output.ends_with(' ')
            { self.Output.pop(); self.Current_Line_Len -= 1; }

            self.EmitRaw(Container.Kind.CloseGlyph());

            if Container.IIFE_Has_Invocation
            { self.EmitRaw("()"); }

            for Glue in & Container.Tail_Glue
            { self.EmitRaw( & Glue.Text); }
        }
    }

    fn EmitGrouped( & mut self, Container: & Dart_Container_Node, Level: usize)
    {
        let Chain = Container.CollectChain();
        let OpenRun: String = Chain.iter().map( | C | C.Kind.OpenGlyph()).collect();
        let mut CloseRun = String:: new ();
        for C in Chain.iter().rev()
        {
            if C.Close_Token.is_some()
            { if C.Is_1Tuple_Record { CloseRun.push(','); } CloseRun.push_str(C.Kind.CloseGlyph()); }
        }
        let Innermost = Chain.last().unwrap();

        // 1. Break line before compound opening run
        self.EnsureNewline();

        // 2. Compound opening run on its own line at Level indent
        self.EmitIndent(Level);
        self.EmitRaw( & OpenRun);
        self.WriteNewline();

        // 3. Innermost children indented at Level + 1
        let SavedCascade = self.In_Cascade;
        self.In_Cascade = false;
        self.EmitNodeList( & Innermost.Children, Level + 1);
        self.In_Cascade = SavedCascade;

        if ! CloseRun.is_empty()
        {
            self.EnsureNewline();
            if self.Output.ends_with("\n\n") { self.Output.pop(); }

            // 4. Compound closing run at Level indent
            self.EmitIndent(Level);
            self.EmitRaw( & CloseRun);

            if Container.IIFE_Has_Invocation
            { self.EmitRaw("()"); }

            // 5. Tail glue from outermost container
            for Glue in & Container.Tail_Glue
            { self.EmitRaw( & Glue.Text); }
            self.WriteNewline();
        }
    }

    fn EmitExpanded( & mut self, Container: & Dart_Container_Node, Level: usize)
    {
        if Container.Open_Token.Text == "${"
        { self.EmitStringInterpolation(Container, Level); return ; }

        // 1. Break line before opening delimiter
        self.EnsureNewline();

        // 2. Opening delimiter on its own line at Level indent
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();

        // 3. Inner children indented at Level + 1
        let SavedCascade = self.In_Cascade;
        self.In_Cascade = false;
        self.EmitNodeList( & Container.Children, Level + 1);
        self.In_Cascade = SavedCascade;

        if Container.Close_Token.is_some()
        {
            self.EnsureNewline();
            if self.Output.ends_with("\n\n") { self.Output.pop(); }

            // 4. Closing delimiter on its own line at Level indent
            self.EmitIndent(Level);
            self.EmitRaw(Container.Kind.CloseGlyph());

            // 5. Tail glue
            for Glue in & Container.Tail_Glue
            { self.EmitRaw( & Glue.Text); }
            if Container.Kind != Container_Kind::Angle
            { self.WriteNewline(); }
        }
    }

    fn EmitIIFE( & mut self, Container: & Dart_Container_Node, Level: usize)
    {
        // 1. Break line before opening delimiter
        self.EnsureNewline();

        // 2. Opening delimiter on its own line: `(` at Level
        self.EmitIndent(Level);
        self.EmitRaw("(");
        self.WriteNewline();

        // 3. Closure parameter list + modifier at Level + 1:
        // E.g. `() async` or `()`
        self.EmitIndent(Level + 1);
        self.EmitRaw("()");
        if let Some(ref Mod) = Container.IIFE_Modifier
        { self.EmitSpace(); self.EmitRaw(Mod); }
        self.WriteNewline();

        // 4. Find the body container
        let mut BodyIdx = 1;
        while BodyIdx < Container.Children.len()
        {
            if let Dart_Syntax_Node::Token(ref T) = Container.Children[BodyIdx]
            { if matches ! (T.Text.as_str(), "async" | "sync*" | "async*" | "=>") { BodyIdx += 1; continue ; } }
            break;
        }

        let SavedCascade = self.In_Cascade;
        self.In_Cascade = false;

        if BodyIdx < Container.Children.len()
        {
            match & Container.Children[BodyIdx]
            {
                Dart_Syntax_Node::Container(ref BodyC) =>
                {
                    if BodyC.Kind == Container_Kind::Brace
                    {
                        self.EmitIndent(Level + 1);
                        self.EmitRaw("{");
                        self.WriteNewline();
                        self.EmitNodeList( & BodyC.Children, Level + 2);
                        self.EnsureNewline();
                        if self.Output.ends_with("\n\n") { self.Output.pop(); }
                        self.EmitIndent(Level + 1);
                        self.EmitRaw("}");
                        self.WriteNewline();
                    }
                    else
                    { self.EmitContainerNode(BodyC, Level + 1); self.WriteNewline(); }
                }
                Dart_Syntax_Node::Token(_) =>
                { self.EmitNodeList( & Container.Children[BodyIdx..], Level + 2); self.EnsureNewline(); }
            }
        }

        self.In_Cascade = SavedCascade;

        // 5. Closing delimiter on its own line at Level:
        // `)` + optional invocation `()` + tail glue
        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw(")");
        if Container.IIFE_Has_Invocation
        { self.EmitRaw("()"); }

        for Glue in & Container.Tail_Glue
        { self.EmitRaw( & Glue.Text); }
        self.WriteNewline();
    }

    fn EmitStringInterpolation( & mut self, Container: & Dart_Container_Node, _Level: usize)
    {
        // Open glyph is "${" directly attached to preceding chunk
        self.EmitRaw("${");

        let FirstChildHasNewline = Container.Children.first().map_or
        (
            false, | node | match node { Dart_Syntax_Node::Token(t) => t.Preceding_Newlines > 0, Dart_Syntax_Node::Container(c) => c.Open_Token.Preceding_Newlines > 0, }
        );

        let IsExpanded = FirstChildHasNewline || Container.Has_Forced_Break || Container.Statement_Count > 1 || Container.Max_Child_Depth > 1;

        if ! IsExpanded
        { self.EmitNodeList( & Container.Children, 0); if Container.Close_Token.is_some() { self.EmitRaw("}"); } }
        else
        {
            let LastNl = self.Output.rfind('\n').map_or(0, | idx | idx + 1);
            let CurrentLine = & self.Output[LastNl..];
            let LeadingSpaces = CurrentLine.chars().take_while( | c | * c == ' ').count();
            let BaseLevel = LeadingSpaces / 4;

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

// ============================================================================
// DART SAFETY GATE TOKEN EQUIVALENCE VERIFIER
// ============================================================================

pub fn VerifyDartTokenEquivalence(OriginalSource: & str, FormattedSource: & str) -> Result < (), String >
{
    let mut OrigScanner = Dart_Lexer::New(OriginalSource);
    let OrigTokens = OrigScanner.TokenizeAll();

    let mut FormScanner = Dart_Lexer::New(FormattedSource);
    let FormTokens = FormScanner.TokenizeAll();

    let OrigSignificant: Vec < & str > = OrigTokens.iter()
   .filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
   .map( | T | T.Text.as_str())
   .collect();

    let FormSignificant: Vec < & str > = FormTokens.iter()
   .filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
   .map( | T | T.Text.as_str())
   .collect();

    if OrigSignificant.len() != FormSignificant.len()
    {
        let DiffLen = std::cmp::min(OrigSignificant.len(), FormSignificant.len());
        for I in 0..DiffLen
        {
            if OrigSignificant[I] != FormSignificant[I] && ! AreTokensEquivalent(OrigSignificant[I], FormSignificant[I])
            {
                eprintln!("Mismatch at index {}:\nOriginal: {:?}\nFormatted: {:?}", I, OrigSignificant[I], FormSignificant[I]);
                break;
            }
        }
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

fn AreTokensEquivalent(Orig: & str, Form: & str) -> bool
{
    if Orig == Form { return true; }

    if (Orig.starts_with("//") && Form.starts_with("//"))
    || (Orig.starts_with("/*") && Form.starts_with("/*"))
    { if Orig.trim_end() == Form.trim_end() { return true; } }

    if
    (
        Orig.starts_with('\'') || Orig.starts_with('"') || Orig.starts_with("r'") || Orig.starts_with("r\"") || Orig.ends_with("'''") || Orig.ends_with("\"\"\"")
    )
    &&
    (
        Form.starts_with('\'') || Form.starts_with('"') || Form.starts_with("r'") || Form.starts_with("r\"") || Form.ends_with("'''") || Form.ends_with("\"\"\"")
    )
    {
        let OrigLines: Vec < & str > = Orig.lines().collect();
        let FormLines: Vec < & str > = Form.lines().collect();
        if OrigLines.len() == FormLines.len()
        { if OrigLines.iter().zip(FormLines.iter()).all( | (OL, FL) | OL.trim() == FL.trim()) { return true; } }
    }

    false
}

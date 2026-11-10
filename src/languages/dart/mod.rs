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
    {
        match Self::Format(Source, Config)
        {
            Ok(Formatted) => Formatted == Source,
            Err(_) => false,
        }
    }
}

// ============================================================================
// DART LEXER (100% Character-Based Lookahead, UTF-8 Panic-Safe)
// ============================================================================

pub struct Dart_Lexer<'a>
{
    Source: & 'a str,
    Chars: Vec<char>,
    Index: usize,
    Line: usize,
    Column: usize,
}

impl <'a> Dart_Lexer<'a>
{
    pub fn New(Source: & 'a str) -> Self
    {
        Dart_Lexer
        {
            Source,
            Chars: Source.chars().collect(),
            Index: 0,
            Line: 1,
            Column: 0,
        }
    }

    fn Peek( & self) -> Option<char>
    {
        if self.Index < self.Chars.len() { Some(self.Chars[self.Index]) } else { None }
    }

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
        let mut RawTokens: Vec<Token_Info> = Vec::new ();
        let mut PrecedingNewlines: usize = 0;

        while let Some(Ch) = self.Peek()
        {
            let StartLine = self.Line;
            let StartCol = self.Column;

            // 1. Newline
            if Ch == '\n'
            {
                self.Advance();
                PrecedingNewlines += 1;
                continue;
            }

            // 2. Horizontal Whitespace
            if Ch == ' ' || Ch == '\t' || Ch == '\r'
            {
                self.Advance();
                continue;
            }

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

            // 5. Multiline Strings ('''...''' or """...""")
            if (Ch == '\'' || Ch == '"') && self.PeekAhead(1) == Some(Ch) && self.PeekAhead(2) == Some(Ch)
            {
                let StringText = self.ScanMultilineString(Ch);
                RawTokens.push(Token_Info::New(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
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
                self.Advance(); self.Advance(); self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("?..".to_string()), "?..".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '.' && self.PeekAhead(1) == Some('.')
            {
                if self.PeekAhead(2) == Some('.')
                {
                    // Spread operator (...)
                    self.Advance(); self.Advance(); self.Advance();
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
                    self.Advance(); self.Advance();
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
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "{".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '}'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
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
            if Ch.is_ascii_digit() || (Ch == '.' && self.PeekAhead(1).map_or(false, |C| C.is_ascii_digit()))
            {
                let NumberText = self.ScanNumber();
                RawTokens.push(Token_Info::New(Token_Kind::Number(NumberText.clone()), NumberText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 10. Dot Shorthands (.violetCore, .origin(), .new()) vs Member Access
            if Ch == '.' && self.PeekAhead(1).map_or(false, |C| C.is_alphabetic() || C == '_')
            {
                let IsDotShorthand = match RawTokens.last()
                {
                    None => true,
                    Some(Prev) => matches!(
                        Prev.Kind,
                        Token_Kind::Open_Delim(_)
                        | Token_Kind::Operator(_)
                        | Token_Kind::Punctuation(':' | ',' | ';')
                        | Token_Kind::Keyword(_)
                    ),
                };

                if IsDotShorthand
                {
                    self.Advance(); // '.'
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
                let Kind = if Self::IsKeyword(&IdentText)
                {
                    Token_Kind::Keyword(IdentText.clone())
                }
                else
                {
                    Token_Kind::Identifier(IdentText.clone())
                };

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
        Self::ClassifyAngleGenerics(&mut RawTokens);

        RawTokens
    }

    fn ScanLineComment(&mut self) -> String
    {
        let mut Result = String::new();
        while let Some(Ch) = self.Peek()
        {
            if Ch == '\n' { break; }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanNestedBlockComment(&mut self) -> String
    {
        let mut Result = String::new();
        Result.push(self.Advance().unwrap()); // '/'
        Result.push(self.Advance().unwrap()); // '*'
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
                if Depth == 0 { break; }
            }
            else
            {
                Result.push(self.Advance().unwrap());
            }
        }
        Result
    }

    fn ScanRawMultilineString(&mut self, QuoteChar: char) -> String
    {
        let mut Result = String::new();
        Result.push(self.Advance().unwrap()); // 'r'
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

    fn ScanRawSingleLineString(&mut self, QuoteChar: char) -> String
    {
        let mut Result = String::new();
        Result.push(self.Advance().unwrap()); // 'r'
        Result.push(self.Advance().unwrap()); // quote

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\n' { break; }
            if Ch == QuoteChar
            {
                Result.push(self.Advance().unwrap());
                break;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanMultilineString(&mut self, QuoteChar: char) -> String
    {
        let mut Result = String::new();
        for _ in 0..3 { Result.push(self.Advance().unwrap()); }

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); }
            }
            else if Ch == QuoteChar && self.PeekAhead(1) == Some(QuoteChar) && self.PeekAhead(2) == Some(QuoteChar)
            {
                for _ in 0..3 { Result.push(self.Advance().unwrap()); }
                break;
            }
            else
            {
                Result.push(self.Advance().unwrap());
            }
        }
        Result
    }

    fn ScanStandardString(&mut self, QuoteChar: char) -> String
    {
        let mut Result = String::new();
        Result.push(self.Advance().unwrap());

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); }
            }
            else if Ch == QuoteChar
            {
                Result.push(self.Advance().unwrap());
                break;
            }
            else if Ch == '\n'
            {
                break;
            }
            else
            {
                Result.push(self.Advance().unwrap());
            }
        }
        Result
    }

    fn ScanNumber(&mut self) -> String
    {
        let mut Result = String::new();

        if self.Peek() == Some('0') && (self.PeekAhead(1) == Some('x') || self.PeekAhead(1) == Some('X'))
        {
            Result.push(self.Advance().unwrap()); // '0'
            Result.push(self.Advance().unwrap()); // 'x'
            while let Some(Ch) = self.Peek()
            {
                if Ch.is_ascii_hexdigit() || Ch == '_'
                {
                    Result.push(self.Advance().unwrap());
                }
                else
                {
                    break;
                }
            }
            return Result;
        }

        let mut HasDot = false;
        let mut HasExp = false;

        while let Some(Ch) = self.Peek()
        {
            if Ch.is_ascii_digit() || Ch == '_'
            {
                Result.push(self.Advance().unwrap());
            }
            else if Ch == '.' && !HasDot && !HasExp && self.PeekAhead(1).map_or(false, |C| C.is_ascii_digit())
            {
                HasDot = true;
                Result.push(self.Advance().unwrap());
            }
            else if (Ch == 'e' || Ch == 'E') && !HasExp
            {
                HasExp = true;
                Result.push(self.Advance().unwrap());
                if self.Peek() == Some('+') || self.Peek() == Some('-')
                {
                    Result.push(self.Advance().unwrap());
                }
            }
            else
            {
                break;
            }
        }
        Result
    }

    fn ScanIdentifier(&mut self) -> String
    {
        let mut Result = String::new();
        while let Some(Ch) = self.Peek()
        {
            if Ch.is_alphanumeric() || Ch == '_' || Ch == '$'
            {
                Result.push(self.Advance().unwrap());
            }
            else
            {
                break;
            }
        }
        Result
    }

    fn ScanOperator(&mut self) -> Option<String>
    {
        let Ch = self.Peek()?;
        let N1 = self.PeekAhead(1);
        let N2 = self.PeekAhead(2);

        // 3-character operators
        if Ch == '>' && N1 == Some('>') && N2 == Some('>')
        {
            if self.PeekAhead(3) == Some('=')
            {
                self.Advance(); self.Advance(); self.Advance(); self.Advance();
                return Some(">>>=".to_string());
            }
            self.Advance(); self.Advance(); self.Advance();
            return Some(">>>".to_string());
        }
        if Ch == '~' && N1 == Some('/') && N2 == Some('=')
        {
            self.Advance(); self.Advance(); self.Advance();
            return Some("~/=".to_string());
        }

        // 2-character operators
        if Ch == '=' && N1 == Some('>')
        {
            self.Advance(); self.Advance();
            return Some("=>".to_string());
        }
        if Ch == '?' && N1 == Some('?')
        {
            if N2 == Some('=')
            {
                self.Advance(); self.Advance(); self.Advance();
                return Some("??=".to_string());
            }
            self.Advance(); self.Advance();
            return Some("??".to_string());
        }
        if Ch == '?' && N1 == Some('.')
        {
            self.Advance(); self.Advance();
            return Some("?.".to_string());
        }
        if Ch == '~' && N1 == Some('/')
        {
            self.Advance(); self.Advance();
            return Some("~/".to_string());
        }
        if (Ch == '+' || Ch == '-' || Ch == '*' || Ch == '/' || Ch == '%' || Ch == '&' || Ch == '|' || Ch == '^' || Ch == '<' || Ch == '>') && N1 == Some('=')
        {
            self.Advance(); self.Advance();
            return Some(format!("{}=", Ch));
        }
        if Ch == '=' && N1 == Some('=')
        {
            self.Advance(); self.Advance();
            return Some("==".to_string());
        }
        if Ch == '!' && N1 == Some('=')
        {
            self.Advance(); self.Advance();
            return Some("!=".to_string());
        }
        if Ch == '&' && N1 == Some('&')
        {
            self.Advance(); self.Advance();
            return Some("&&".to_string());
        }
        if Ch == '|' && N1 == Some('|')
        {
            self.Advance(); self.Advance();
            return Some("||".to_string());
        }
        if Ch == '<' && N1 == Some('<')
        {
            if N2 == Some('=')
            {
                self.Advance(); self.Advance(); self.Advance();
                return Some("<<=".to_string());
            }
            self.Advance(); self.Advance();
            return Some("<<".to_string());
        }
        if Ch == '>' && N1 == Some('>')
        {
            if N2 == Some('=')
            {
                self.Advance(); self.Advance(); self.Advance();
                return Some(">>=".to_string());
            }
            self.Advance(); self.Advance();
            return Some(">>".to_string());
        }
        if Ch == '+' && N1 == Some('+')
        {
            self.Advance(); self.Advance();
            return Some("++".to_string());
        }
        if Ch == '-' && N1 == Some('-')
        {
            self.Advance(); self.Advance();
            return Some("--".to_string());
        }

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
        matches!(
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

    fn ClassifyAngleGenerics(Tokens: &mut [Token_Info])
    {
        for I in 0..Tokens.len()
        {
            if Tokens[I].Text != "<" { continue; }

            // Preceding token check
            let PrecedingValid = if I == 0 { false } else {
                let Prev = &Tokens[I - 1];
                matches!(Prev.Kind, Token_Kind::Identifier(_) | Token_Kind::Keyword(_))
                || Prev.Text == ")" || Prev.Text == "]"
            };

            if !PrecedingValid { continue; }

            // Scan forward for matching '>'
            let mut Depth: usize = 0;
            let mut MatchIndex: Option<usize> = None;
            let mut J = I;

            while J < Tokens.len()
            {
                if Tokens[J].Text == "<"
                {
                    Depth += 1;
                }
                else if Tokens[J].Text == ">"
                {
                    Depth -= 1;
                    if Depth == 0
                    {
                        MatchIndex = Some(J);
                        break;
                    }
                }
                else if Tokens[J].Text == ";" || Tokens[J].Text == "{" || Tokens[J].Text == "}"
                {
                    break;
                }
                J += 1;
            }

            if let Some(M) = MatchIndex
            {
                let FollowValid = if M + 1 < Tokens.len()
                {
                    let Next = &Tokens[M + 1];
                    matches!(
                        Next.Text.as_str(),
                        "(" | ")" | "]" | "}" | ":" | ";" | "," | "." | "?" | "==" | "!=" | "[" | "{"
                        | ">" | "extends" | "implements" | "with" | "on" | "when" | "as" | "is" | "async"
                    )
                    || matches!(Next.Kind, Token_Kind::Identifier(_) | Token_Kind::Keyword(_))
                    || Next.Preceding_Newlines > 0
                }
                else
                {
                    true
                };

                if FollowValid
                {
                    Tokens[I].Kind = Token_Kind::Open_Delim(Container_Kind::Angle);
                    Tokens[M].Kind = Token_Kind::Close_Delim(Container_Kind::Angle);
                }
            }
        }
    }
}

// ============================================================================
// DART SYNTAX TREE & CONTAINER NODE
// ============================================================================

#[derive(Debug, Clone)]
pub enum Dart_Syntax_Node
{
    Token(Token_Info),
    Container(Dart_Container_Node),
}

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
            Children: Vec::new(),
            Tail_Glue: Vec::new(),
            Has_Forced_Break: false,
            Statement_Count: 0,
            Max_Child_Depth: 0,
            Is_1Tuple_Record: false,
        }
    }

    pub fn GetSoleChildContainer(&self) -> Option<&Dart_Container_Node>
    {
        if self.Children.len() == 1
        {
            if let Dart_Syntax_Node::Container(ref C) = self.Children[0]
            {
                if C.Tail_Glue.is_empty() && self.Close_Token.is_some() && C.Close_Token.is_some()
                {
                    return Some(C);
                }
            }
        }
        else if self.Is_1Tuple_Record && self.Children.len() == 2
        {
            if let Dart_Syntax_Node::Container(ref C) = self.Children[0]
            {
                if self.Close_Token.is_some() && C.Close_Token.is_some()
                {
                    return Some(C);
                }
            }
        }
        None
    }

    pub fn CollectChain(&self) -> Vec<&Dart_Container_Node>
    {
        let mut Chain = vec![self];
        let mut Current = self;
        while let Some(Child) = Current.GetSoleChildContainer()
        {
            if Current.Kind == Container_Kind::Angle || Child.Kind == Container_Kind::Angle
            {
                break;
            }
            Chain.push(Child);
            Current = Child;
        }
        Chain
    }

    pub fn CalculateSingleLineSpan(&self) -> usize
    {
        if self.Has_Forced_Break { return usize::MAX; }

        let mut Total = self.Kind.OpenGlyph().len() + self.Kind.CloseGlyph().len();
        if self.Kind == Container_Kind::Brace && !self.Children.is_empty()
        {
            Total += 2; // "{ ... }"
        }

        for (I, Child) in self.Children.iter().enumerate()
        {
            if I > 0 { Total += 1; }
            match Child
            {
                Dart_Syntax_Node::Token(T) =>
                {
                    if T.Text.contains('\n') { return usize::MAX; }
                    Total += T.Text.len();
                }
                Dart_Syntax_Node::Container(C) =>
                {
                    let ChildSpan = C.CalculateSingleLineSpan();
                    if ChildSpan == usize::MAX { return usize::MAX; }
                    Total += ChildSpan;
                }
            }
        }

        for Glue in &self.Tail_Glue
        {
            Total += Glue.Text.len();
        }

        Total
    }
}

// ============================================================================
// DART CONTAINER TREE BUILDER
// ============================================================================

pub struct Dart_Container_Builder
{
    Tokens: Vec<Token_Info>,
    Index: usize,
}

impl Dart_Container_Builder
{
    pub fn New(Tokens: Vec<Token_Info>) -> Self
    {
        Dart_Container_Builder { Tokens, Index: 0 }
    }

    pub fn BuildTree(&mut self) -> Vec<Dart_Syntax_Node>
    {
        let mut RootNodes = Vec::new();
        while self.Index < self.Tokens.len()
        {
            if let Some(Node) = self.ParseNode()
            {
                RootNodes.push(Node);
            }
        }
        RootNodes
    }

    fn ParseNode(&mut self) -> Option<Dart_Syntax_Node>
    {
        if self.Index >= self.Tokens.len() { return None; }
        let Current = self.Tokens[self.Index].clone();

        match Current.Kind
        {
            Token_Kind::Open_Delim(Kind) =>
            {
                self.Index += 1;
                let mut Container = Dart_Container_Node::New(Kind, Current);

                while self.Index < self.Tokens.len()
                {
                    let Next = &self.Tokens[self.Index];
                    if let Token_Kind::Close_Delim(CloseKind) = Next.Kind
                    {
                        if CloseKind == Kind
                        {
                            Container.Close_Token = Some(self.Tokens[self.Index].clone());
                            self.Index += 1;
                            break;
                        }
                    }

                    // Forced break conditions
                    if matches!(Next.Kind, Token_Kind::Comment_Line(_)) { Container.Has_Forced_Break = true; }
                    if Next.Text.contains('\n') { Container.Has_Forced_Break = true; }
                    if Next.Text == ";" { Container.Statement_Count += 1; }

                    if let Some(ChildNode) = self.ParseNode()
                    {
                        if let Dart_Syntax_Node::Container(ref ChildC) = ChildNode
                        {
                            if ChildC.Has_Forced_Break { Container.Has_Forced_Break = true; }
                            for Glue in &ChildC.Tail_Glue { if Glue.Text == ";" { Container.Statement_Count += 1; } }
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
                        {
                            if T.Text == "," { Container.Is_1Tuple_Record = true; }
                        }
                    }
                    else if Container.Children.len() == 1
                    {
                        if let Dart_Syntax_Node::Container(ref mut C) = Container.Children[0]
                        {
                            if C.Tail_Glue.len() == 1 && C.Tail_Glue[0].Text == ","
                            {
                                C.Tail_Glue.pop();
                                Container.Is_1Tuple_Record = true;
                            }
                        }
                    }
                }

                // Collect tail glue (';', ',')
                while self.Index < self.Tokens.len()
                {
                    let Next = &self.Tokens[self.Index];
                    if Next.Preceding_Newlines > 0 { break; }
                    if Next.Text == ";" || Next.Text == ","
                    {
                        Container.Tail_Glue.push(self.Tokens[self.Index].clone());
                        self.Index += 1;
                    }
                    else
                    {
                        break;
                    }
                }

                Some(Dart_Syntax_Node::Container(Container))
            }
            _ =>
            {
                self.Index += 1;
                Some(Dart_Syntax_Node::Token(Current))
            }
        }
    }
}

// ============================================================================
// DART EMITTER (100% LAFS Compliance, Monolith First, Maximal Grouping)
// ============================================================================

fn OutputEndsWithControlKeyword(Output: &str) -> bool
{
    let ControlKeywords = [
        "if", "while", "for", "switch", "catch", "assert", "when", "case", "return", "yield", "await", "sync*", "async*",
    ];
    let Trimmed = Output.trim_end();
    for Kw in ControlKeywords
    {
        if Trimmed.ends_with(Kw)
        {
            let PrefixLen = Trimmed.len() - Kw.len();
            if PrefixLen == 0 { return true; }
            if let Some(Ch) = Trimmed[..PrefixLen].chars().last()
            {
                if !Ch.is_alphanumeric() && Ch != '_' && Ch != '$' { return true; }
            }
        }
    }
    false
}

pub struct Dart_Emitter<'a>
{
    Config: &'a Config_Settings,
    Output: String,
    Current_Line_Len: usize,
    Line_Has_Content: bool,
}

impl <'a> Dart_Emitter<'a>
{
    pub fn New(Config: &'a Config_Settings) -> Self
    {
        Dart_Emitter
        {
            Config,
            Output: String::new(),
            Current_Line_Len: 0,
            Line_Has_Content: false,
        }
    }

    pub fn Emit(mut self, Nodes: &[Dart_Syntax_Node]) -> String
    {
        self.EmitNodeList(Nodes, 0);
        if !self.Output.ends_with('\n') { self.Output.push('\n'); }
        self.Output
    }

    fn WriteNewline(&mut self)
    {
        while self.Output.ends_with(' ') { self.Output.pop(); }
        self.Output.push('\n');
        self.Current_Line_Len = 0;
        self.Line_Has_Content = false;
    }

    fn EnsureNewline(&mut self)
    {
        if self.Line_Has_Content { self.WriteNewline(); }
    }

    fn EnsureBlankLine(&mut self)
    {
        if self.Config.Max_Blank_Lines == 0 || self.Output.is_empty() { return; }
        self.EnsureNewline();
        if !self.Output.ends_with("\n\n")
            && !self.Output.ends_with("{\n")
            && !self.Output.ends_with("[\n")
            && !self.Output.ends_with("(\n")
        {
            self.Output.push('\n');
            self.Current_Line_Len = 0;
            self.Line_Has_Content = false;
        }
    }

    fn EmitIndent(&mut self, Level: usize)
    {
        if !self.Line_Has_Content
        {
            let IndentStr = self.Config.IndentString(Level);
            self.Output.push_str(&IndentStr);
            self.Current_Line_Len = IndentStr.len();
            self.Line_Has_Content = true;
        }
    }

    fn EmitRaw(&mut self, Text: &str)
    {
        self.Output.push_str(Text);
        if let Some(LastNl) = Text.rfind('\n')
        {
            self.Current_Line_Len = Text.len() - LastNl - 1;
            self.Line_Has_Content = self.Current_Line_Len > 0;
        }
        else
        {
            self.Current_Line_Len += Text.len();
            self.Line_Has_Content = true;
        }
    }

    fn EmitSpace(&mut self)
    {
        if self.Line_Has_Content && self.Current_Line_Len > 0 && !self.Output.ends_with(' ') && !self.Output.ends_with('\n')
        {
            self.Output.push(' ');
            self.Current_Line_Len += 1;
        }
    }

    fn EmitNodeList(&mut self, Nodes: &[Dart_Syntax_Node], Level: usize)
    {
        for (I, Node) in Nodes.iter().enumerate()
        {
            match Node
            {
                Dart_Syntax_Node::Token(T) =>
                {
                    self.EmitSingleToken(T, Level, Nodes, I);
                }
                Dart_Syntax_Node::Container(C) =>
                {
                    self.EmitContainerNode(C, Level);
                }
            }
        }
    }

    fn EmitSingleToken(&mut self, Token: &Token_Info, Level: usize, _Nodes: &[Dart_Syntax_Node], _Index: usize)
    {
        // 1. Preceding newlines preservation
        if Token.Preceding_Newlines > 0
        {
            self.EnsureNewline();
            if Token.Preceding_Newlines >= 2 { self.EnsureBlankLine(); }
        }

        // 2. Line comment
        if let Token_Kind::Comment_Line(CommentText) = &Token.Kind
        {
            if self.Line_Has_Content { self.EmitSpace(); } else { self.EmitIndent(Level); }
            self.EmitRaw(CommentText);
            self.WriteNewline();
            return;
        }

        // 3. Block comment
        if let Token_Kind::Comment_Block(CommentText) = &Token.Kind
        {
            if !self.Line_Has_Content { self.EmitIndent(Level); } else { self.EmitSpace(); }
            self.EmitRaw(CommentText);
            return;
        }

        // 4. Cascade notation (.. or ?..)
        if Token.Text == ".." || Token.Text == "?.."
        {
            if Token.Preceding_Newlines > 0
            {
                self.EnsureNewline();
                self.EmitIndent(Level + 1);
            }
            self.EmitRaw(&Token.Text);
            return;
        }

        // 5. Operators, Identifiers, Keywords
        if !self.Line_Has_Content { self.EmitIndent(Level); }

        match &Token.Kind
        {
            Token_Kind::Keyword(K) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                    && !self.Output.ends_with(' ') && !self.Output.ends_with('\n')
                    && !self.Output.ends_with('(') && !self.Output.ends_with('[') && !self.Output.ends_with('{')
                    && !self.Output.ends_with('@') && !self.Output.ends_with('?')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(K);
                self.EmitSpace();
            }
            Token_Kind::Identifier(Ident) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                    && !self.Output.ends_with(' ') && !self.Output.ends_with('\n')
                    && !self.Output.ends_with('(') && !self.Output.ends_with('[') && !self.Output.ends_with('{')
                    && !self.Output.ends_with('.') && !self.Output.ends_with("..") && !self.Output.ends_with("?..")
                    && !self.Output.ends_with("?.") && !self.Output.ends_with('@') && !self.Output.ends_with('?')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(Ident);
            }
            Token_Kind::Number(Num) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                    && !self.Output.ends_with(' ') && !self.Output.ends_with('\n')
                    && !self.Output.ends_with('(') && !self.Output.ends_with('[') && !self.Output.ends_with('{')
                    && !self.Output.ends_with(':') && !self.Output.ends_with('-')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(Num);
            }
            Token_Kind::Operator(Op) =>
            {
                if matches!(Op.as_str(), "=>" | "=" | "==" | "!=" | "&&" | "||" | "+" | "-" | "*" | "/" | "~/" | "%" | "+=" | "-=" | "*=" | "/=" | "~/=" | "%=" | "??=" | "??" | "?:" | "<=" | ">=" | "|" | "&" | "^" | "<<" | ">>" | ">>>")
                {
                    if self.Line_Has_Content
                        && !self.Output.ends_with(' ')
                        && !self.Output.ends_with('\n')
                        && !self.Output.ends_with('(')
                        && !self.Output.ends_with('[')
                        && !self.Output.ends_with('{')
                    {
                        self.EmitSpace();
                    }
                    self.EmitRaw(Op);
                    self.EmitSpace();
                }
                else if Op == "..." || Op == "...?" || Op == "." || Op == "?." || Op == ".." || Op == "?.." || Op == "!" || Op == "~" || Op == "@"
                {
                    self.EmitRaw(Op);
                }
                else if Op == "?"
                {
                    if self.Output.ends_with('(') || self.Output.ends_with('[') || self.Output.ends_with(',')
                    {
                        self.EmitRaw("?");
                    }
                    else if self.Output.ends_with(' ')
                    {
                        self.EmitRaw("?");
                        self.EmitSpace();
                    }
                    else
                    {
                        self.EmitRaw("?");
                    }
                }
                else
                {
                    self.EmitRaw(Op);
                }
            }
            Token_Kind::Punctuation(P) =>
            {
                if *P == ',' || *P == ';'
                {
                    while self.Output.ends_with(' ') { self.Output.pop(); }
                    self.EmitRaw(&P.to_string());
                    self.EmitSpace();
                }
                else if *P == ':'
                {
                    let Trimmed = self.Output.trim_end();
                    if Trimmed.ends_with("this") || Trimmed.ends_with(')')
                    {
                        if !self.Output.ends_with(' ') { self.EmitSpace(); }
                    }
                    else
                    {
                        while self.Output.ends_with(' ') { self.Output.pop(); }
                    }
                    self.EmitRaw(":");
                    self.EmitSpace();
                }
                else
                {
                    self.EmitRaw(&P.to_string());
                }
            }
            Token_Kind::String_Lit(Text) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0
                    && !self.Output.ends_with(' ') && !self.Output.ends_with('\n')
                    && !self.Output.ends_with('(') && !self.Output.ends_with('[') && !self.Output.ends_with('{')
                    && !self.Output.ends_with(':')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(Text);
            }
            _ =>
            {
                self.EmitRaw(&Token.Text);
            }
        }
    }

    fn DetermineLayoutState(&self, Container: &Dart_Container_Node) -> Layout_State
    {
        // 1. Empty container -> State 1 Monolith
        if Container.Children.is_empty() { return Layout_State::Monolith; }

        // 2. Complexity budget check (Max 4 statements, Max depth 2)
        if Container.Statement_Count > self.Config.Max_Statements_In_Monolith
            || Container.Max_Child_Depth > self.Config.Max_Block_Depth
        {
            return Layout_State::Expanded;
        }

        // 3. Canvas Limit check for Monolith
        let Span = Container.CalculateSingleLineSpan();
        if !Container.Has_Forced_Break && (self.Current_Line_Len + Span <= self.Config.Canvas_Width)
        {
            return Layout_State::Monolith;
        }

        // 4. Sole-Child Chain -> State 2 Grouped
        let Chain = Container.CollectChain();
        if Chain.len() > 1
        {
            return Layout_State::Grouped;
        }

        // 5. Default -> State 3 Expanded
        Layout_State::Expanded
    }

    fn EmitContainerNode(&mut self, Container: &Dart_Container_Node, Level: usize)
    {
        let Layout = self.DetermineLayoutState(Container);

        match Layout
        {
            Layout_State::Monolith => self.EmitMonolith(Container, Level),
            Layout_State::Grouped => self.EmitGrouped(Container, Level),
            Layout_State::Expanded => self.EmitExpanded(Container, Level),
        }
    }

    fn EmitMonolith(&mut self, Container: &Dart_Container_Node, Level: usize)
    {
        if !self.Line_Has_Content { self.EmitIndent(Level); }

        let NeedSpace = self.Line_Has_Content && self.Current_Line_Len > 0
            && !self.Output.ends_with(' ') && !self.Output.ends_with('\n')
            && !self.Output.ends_with('(') && !self.Output.ends_with('[') && !self.Output.ends_with('{')
            && !self.Output.ends_with('@') && !self.Output.ends_with('.') && !self.Output.ends_with("..")
            && !self.Output.ends_with("?..") && !self.Output.ends_with("...") && !self.Output.ends_with("...?")
            && !self.Output.ends_with('?');

        if NeedSpace && Container.Kind == Container_Kind::Brace { self.EmitSpace(); }

        self.EmitRaw(Container.Kind.OpenGlyph());

        if Container.Kind == Container_Kind::Brace && !Container.Children.is_empty()
        {
            self.EmitRaw(" ");
        }

        let mut PrevNeedsSpace = false;
        for Child in &Container.Children
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
                        let Trimmed = self.Output.trim_end();
                        if Trimmed.ends_with("this") || Trimmed.ends_with(')')
                        {
                            if !self.Output.ends_with(' ') { self.EmitSpace(); }
                        }
                        else
                        {
                            while self.Output.ends_with(' ') { self.Output.pop(); }
                        }
                        self.EmitRaw(":");
                        self.EmitSpace();
                        PrevNeedsSpace = false;
                        continue;
                    }

                    if PrevNeedsSpace && !IsPunct && !OutputEndsWithNoSpace
                    {
                        self.EmitSpace();
                    }

                    // Binary operators inside Monolith
                    if matches!(T.Text.as_str(), "=>" | "=" | "==" | "!=" | "&&" | "||" | "+" | "-" | "*" | "/" | "~/" | "%" | "+=" | "-=" | "*=" | "/=" | "~/=" | "%=" | "??=" | "??" | "<=" | ">=" | "|" | "&" | "^")
                    {
                        if !self.Output.ends_with(' ') && !OutputEndsWithNoSpace
                        {
                            self.EmitSpace();
                        }
                    }

                    self.EmitRaw(&T.Text);

                    if T.Text == "," || T.Text == ";"
                    {
                        self.EmitSpace();
                        PrevNeedsSpace = false;
                    }
                    else if matches!(T.Text.as_str(), "=>" | "=" | "==" | "!=" | "&&" | "||" | "+" | "-" | "*" | "/" | "~/" | "%" | "+=" | "-=" | "*=" | "/=" | "~/=" | "%=" | "??=" | "??" | "<=" | ">=" | "|" | "&" | "^")
                    {
                        self.EmitSpace();
                        PrevNeedsSpace = false;
                    }
                    else if Dart_Lexer::IsKeyword(&T.Text)
                    {
                        self.EmitSpace();
                        PrevNeedsSpace = false;
                    }
                    else
                    {
                        PrevNeedsSpace = !matches!(
                            T.Text.as_str(),
                            "." | "?." | ".." | "?.." | "..." | "...?" | "(" | "[" | "!" | "~" | "@" | "$" | "?"
                        );
                    }
                }
                Dart_Syntax_Node::Container(Sub) =>
                {
                    if Sub.Kind == Container_Kind::Brace
                    {
                        if PrevNeedsSpace || (!self.Output.ends_with(' ') && !self.Output.ends_with('(') && !self.Output.ends_with('[') && !self.Output.ends_with('{'))
                        {
                            self.EmitSpace();
                        }
                    }
                    else if Sub.Kind == Container_Kind::Paren
                    {
                        if OutputEndsWithControlKeyword(&self.Output)
                            || self.Output.ends_with('=')
                            || self.Output.ends_with("=>")
                            || self.Output.ends_with(':')
                            || self.Output.ends_with(',')
                            || self.Output.ends_with("||")
                            || self.Output.ends_with("&&")
                        {
                            if !self.Output.ends_with(' ') { self.EmitSpace(); }
                        }
                    }
                    else if Sub.Kind == Container_Kind::Bracket
                    {
                        if self.Output.ends_with('=')
                            || self.Output.ends_with("=>")
                            || self.Output.ends_with(':')
                            || self.Output.ends_with(',')
                        {
                            if !self.Output.ends_with(' ') { self.EmitSpace(); }
                        }
                    }

                    self.EmitMonolith(Sub, Level);
                    PrevNeedsSpace = true;
                }
            }
        }

        if Container.Kind == Container_Kind::Brace && !Container.Children.is_empty() && !self.Output.ends_with(' ')
        {
            self.EmitRaw(" ");
        }

        if Container.Close_Token.is_some()
        {
            if (Container.Kind == Container_Kind::Paren || Container.Kind == Container_Kind::Bracket || Container.Kind == Container_Kind::Angle)
                && self.Output.ends_with(' ')
            {
                self.Output.pop();
                self.Current_Line_Len -= 1;
            }

            self.EmitRaw(Container.Kind.CloseGlyph());

            for Glue in &Container.Tail_Glue
            {
                self.EmitRaw(&Glue.Text);
            }
        }
    }

    fn EmitGrouped(&mut self, Container: &Dart_Container_Node, Level: usize)
    {
        let Chain = Container.CollectChain();
        let OpenRun: String = Chain.iter().map(|C| C.Kind.OpenGlyph()).collect();
        let mut CloseRun = String::new();
        for C in Chain.iter().rev()
        {
            if C.Close_Token.is_some()
            {
                if C.Is_1Tuple_Record
                {
                    CloseRun.push(',');
                }
                CloseRun.push_str(C.Kind.CloseGlyph());
            }
        }
        let Innermost = Chain.last().unwrap();

        // 1. Break line before compound opening run
        self.EnsureNewline();

        // 2. Compound opening run on its own line at Level indent
        self.EmitIndent(Level);
        self.EmitRaw(&OpenRun);
        self.WriteNewline();

        // 3. Innermost children indented at Level + 1
        self.EmitNodeList(&Innermost.Children, Level + 1);

        if !CloseRun.is_empty()
        {
            self.EnsureNewline();
            if self.Output.ends_with("\n\n") { self.Output.pop(); }

            // 4. Compound closing run at Level indent
            self.EmitIndent(Level);
            self.EmitRaw(&CloseRun);

            // 5. Tail glue from outermost container
            for Glue in &Container.Tail_Glue
            {
                self.EmitRaw(&Glue.Text);
            }
            self.WriteNewline();
        }
    }

    fn EmitExpanded(&mut self, Container: &Dart_Container_Node, Level: usize)
    {
        // 1. Break line before opening delimiter
        self.EnsureNewline();

        // 2. Opening delimiter on its own line at Level indent
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();

        // 3. Inner children indented at Level + 1
        self.EmitNodeList(&Container.Children, Level + 1);

        if Container.Close_Token.is_some()
        {
            self.EnsureNewline();
            if self.Output.ends_with("\n\n") { self.Output.pop(); }

            // 4. Closing delimiter on its own line at Level indent
            self.EmitIndent(Level);
            self.EmitRaw(Container.Kind.CloseGlyph());

            // 5. Tail glue
            for Glue in &Container.Tail_Glue
            {
                self.EmitRaw(&Glue.Text);
            }
            self.WriteNewline();
        }
    }
}

// ============================================================================
// DART SAFETY GATE TOKEN EQUIVALENCE VERIFIER
// ============================================================================

pub fn VerifyDartTokenEquivalence(OriginalSource: &str, FormattedSource: &str) -> Result<(), String>
{
    let mut OrigScanner = Dart_Lexer::New(OriginalSource);
    let OrigTokens = OrigScanner.TokenizeAll();

    let mut FormScanner = Dart_Lexer::New(FormattedSource);
    let FormTokens = FormScanner.TokenizeAll();

    let OrigSignificant: Vec<&str> = OrigTokens.iter()
        .filter(|T| !matches!(T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
        .map(|T| T.Text.as_str())
        .collect();

    let FormSignificant: Vec<&str> = FormTokens.iter()
        .filter(|T| !matches!(T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
        .map(|T| T.Text.as_str())
        .collect();

    if OrigSignificant.len() != FormSignificant.len()
    {
        let DiffLen = std::cmp::min(OrigSignificant.len(), FormSignificant.len());
        for I in 0..DiffLen
        {
            if OrigSignificant[I] != FormSignificant[I] && !AreTokensEquivalent(OrigSignificant[I], FormSignificant[I])
            {
                eprintln!("Mismatch at index {}:\nOriginal: {:?}\nFormatted: {:?}", I, OrigSignificant[I], FormSignificant[I]);
                break;
            }
        }
        return Err(format!(
            "Safety Gate Failed: Significant token count mismatch! (Original: {}, Formatted: {})",
            OrigSignificant.len(),
            FormSignificant.len()
        ));
    }

    for (Index, (Orig, Form)) in OrigSignificant.iter().zip(FormSignificant.iter()).enumerate()
    {
        if Orig != Form && !AreTokensEquivalent(Orig, Form)
        {
            return Err(format!("Safety Gate Failed: Token #{} mismatch! (Expected '{}', Got '{}')", Index, Orig, Form));
        }
    }

    Ok(())
}

fn AreTokensEquivalent(Orig: &str, Form: &str) -> bool
{
    if Orig == Form { return true; }

    if (Orig.starts_with("//") && Form.starts_with("//"))
        || (Orig.starts_with("/*") && Form.starts_with("/*"))
    {
        if Orig.trim_end() == Form.trim_end() { return true; }
    }

    if (Orig.starts_with('\'') || Orig.starts_with('"') || Orig.starts_with("r'") || Orig.starts_with("r\""))
        && (Form.starts_with('\'') || Form.starts_with('"') || Form.starts_with("r'") || Form.starts_with("r\""))
    {
        let OrigLines: Vec<&str> = Orig.lines().collect();
        let FormLines: Vec<&str> = Form.lines().collect();
        if OrigLines.len() == FormLines.len()
        {
            if OrigLines.iter().zip(FormLines.iter()).all(|(OL, FL)| OL.trim() == FL.trim())
            {
                return true;
            }
        }
    }

    false
}

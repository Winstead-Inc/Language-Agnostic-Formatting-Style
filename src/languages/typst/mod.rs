#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]

use crate::types:: { Config_Settings, Container_Kind, Layout_State, Source_Language, Token_Info, Token_Kind };
use crate::verifier::Safety_Verifier;

pub struct Typst_Formatter;

impl Typst_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let mut Lexer = Typst_Lexer::New(Source);
        let Tokens = Lexer.TokenizeAll();

        let mut Builder = Typst_Container_Builder::New(Tokens);
        let Nodes = Builder.BuildTree()?;

        let Emitter = Typst_Emitter::New(Config);
        let Formatted = Emitter.Emit( & Nodes);

        if let Err(VerifyErr) = VerifyTypstTokenEquivalence(Source, & Formatted)
        {
            return Err(format ! ("{}. Output aborted to prevent code corruption.", VerifyErr));
        }

        if let Err(VerifyErr) = Safety_Verifier::VerifyPairingStability(Source, & Formatted, Source_Language::Typst)
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
// TYPST SYNTAX NODES
// ============================================================================

#[derive(Debug, Clone)]
pub enum Typst_Syntax_Node
{
    Token(Token_Info),
    Container(Typst_Container_Node),
}

#[derive(Debug, Clone)]
pub struct Typst_Container_Node
{
    pub Kind: Container_Kind,
    pub Open_Token: Token_Info,
    pub Close_Token: Option<Token_Info>,
    pub Children: Vec<Typst_Syntax_Node>,
    pub Tail_Glue: Vec<Token_Info>,
    pub Has_Forced_Break: bool,
    pub Statement_Count: usize,
}

impl Typst_Container_Node
{
    pub fn New(Kind: Container_Kind, Open_Token: Token_Info) -> Self
    {
        Typst_Container_Node
        {
            Kind,
            Open_Token,
            Close_Token: None,
            Children: Vec::new (),
            Tail_Glue: Vec::new (),
            Has_Forced_Break: false,
            Statement_Count: 0,
        }
    }

    pub fn IsSoleChildChain( & self) -> bool
    {
        if self.Children.len() == 1
        {
            matches ! (self.Children[0], Typst_Syntax_Node::Container(_))
        }
        else
        {
            false
        }
    }

    pub fn GetSoleChildContainer( & self) -> Option< & Typst_Container_Node>
    {
        if self.Children.len() == 1
        {
            if let Typst_Syntax_Node::Container(ref C) = self.Children[0]
            {
                if C.Tail_Glue.is_empty()
                {
                    return Some(C);
                }
            }
        }
        None
    }

    pub fn CollectChain( & self) -> Vec< & Typst_Container_Node>
    {
        let mut Chain = vec![self];
        let mut Current = self;
        while let Some(Child) = Current.GetSoleChildContainer()
        {
            if Current.Close_Token.is_none() || Child.Close_Token.is_none()
            {
                break ;
            }
            Chain.push(Child);
            Current = Child;
        }
        Chain
    }

    pub fn CalculateSingleLineSpan( & self) -> usize
    {
        if self.Has_Forced_Break
        {
            return usize::MAX;
        }

        let mut Total = self.Kind.OpenGlyph().len() + self.Kind.CloseGlyph().len();
        if self.Kind == Container_Kind::Brace && ! self.Children.is_empty()
        {
            Total += 2;
        }

        for Child in & self.Children
        {
            match Child
            {
                Typst_Syntax_Node::Token(T) =>
                {
                    Total += T.Text.len() + 1;
                }
                Typst_Syntax_Node::Container(C) =>
                {
                    let SubSpan = C.CalculateSingleLineSpan();
                    if SubSpan == usize::MAX
                    {
                        return usize::MAX;
                    }
                    Total += SubSpan + 1;
                }
            }
        }

        for Glue in & self.Tail_Glue
        {
            Total += Glue.Text.len();
        }

        Total
    }

    pub fn CalculateBlockDepth( & self) -> usize
    {
        let mut MaxChildDepth = 0;
        for Child in & self.Children
        {
            if let Typst_Syntax_Node::Container(C) = Child
            {
                let D = C.CalculateBlockDepth();
                if D > MaxChildDepth
                {
                    MaxChildDepth = D;
                }
            }
        }
        if self.Kind == Container_Kind::Brace
        {
            1 + MaxChildDepth
        }
        else
        {
            MaxChildDepth
        }
    }

    pub fn CountTotalContainers( & self) -> usize
    {
        let mut Total = 1;
        for Child in & self.Children
        {
            if let Typst_Syntax_Node::Container(C) = Child
            {
                Total += C.CountTotalContainers();
            }
        }
        Total
    }
}

// ============================================================================
// TYPST LEXER
// ============================================================================

pub struct Typst_Lexer<'a>
{
    Source: & 'a str,
    Chars: Vec<char>,
    Index: usize,
    Line: usize,
    Column: usize,
}

impl <'a> Typst_Lexer<'a>
{
    pub fn New(Source: & 'a str) -> Self
    {
        Typst_Lexer
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
        if self.Index < self.Chars.len()
        {
            Some(self.Chars[self.Index])
        }
        else
        {
            None
        }
    }

    fn PeekAhead( & self, Offset: usize) -> Option<char>
    {
        let Target = self.Index + Offset;
        if Target < self.Chars.len()
        {
            Some(self.Chars[Target])
        }
        else
        {
            None
        }
    }

    fn Advance( & mut self) -> Option<char>
    {
        if self.Index >= self.Chars.len()
        {
            return None;
        }
        let Ch = self.Chars[self.Index];
        self.Index += 1;
        if Ch == '\n'
        {
            self.Line += 1;
            self.Column = 0;
        }
        else
        {
            self.Column += 1;
        }
        Some(Ch)
    }

    pub fn TokenizeAll( & mut self) -> Vec<Token_Info>
    {
        let mut RawTokens: Vec<Token_Info> = Vec:: new ();
        let mut PrecedingNewlines: usize = 0;
        let mut CurrentLeadingWhitespace = String:: new ();
        let mut AtLineStart = true;

        while let Some(Ch) = self.Peek()
        {
            let StartLine = self.Line;
            let StartCol = self.Column;

            // 1. Newline
            if Ch == '\n'
            {
                self.Advance();
                PrecedingNewlines += 1;
                AtLineStart = true;
                CurrentLeadingWhitespace.clear();
                continue;
            }

            // 2. Horizontal Whitespace
            if Ch == ' ' || Ch == '\t' || Ch == '\r'
            {
                let WS = self.Advance().unwrap();
                if AtLineStart && (WS == ' ' || WS == '\t')
                {
                    CurrentLeadingWhitespace.push(WS);
                }
                continue;
            }

            let TokenIndent = if AtLineStart
            {
                AtLineStart = false;
                let Indent = CurrentLeadingWhitespace.clone();
                CurrentLeadingWhitespace.clear();
                Indent
            }
            else
            {
                String::new ()
            };

            // 3. Comments (Line // and Nested Block /* ... */)
            if Ch == '/' && self.PeekAhead(1) == Some('/')
            {
                let CommentText = self.ScanLineComment();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Comment_Line(CommentText.clone()), CommentText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '/' && self.PeekAhead(1) == Some('*')
            {
                let CommentText = self.ScanNestedBlockComment();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Comment_Block(CommentText.clone()), CommentText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 4. Strings: "..."
            if Ch == '"'
            {
                let StringText = self.ScanString();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 5. Raw Code / Backtick Blocks: `...` or ```...```
            if Ch == '`'
            {
                let RawText = self.ScanRawCode();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::String_Lit(RawText.clone()), RawText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 6. Labels: <sec:intro> or <eq:1>
            if Ch == '<'
            {
                if let Some(LabelText) = self.TryScanLabel()
                {
                    RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Identifier(LabelText.clone()), LabelText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                    PrecedingNewlines = 0;
                    continue;
                }
            }

            // 7. Containers / Delimiters: (), {}, []
            if Ch == '('
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Open_Delim(Container_Kind::Paren), "(".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == ')'
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Close_Delim(Container_Kind::Paren), ")".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '{'
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Open_Delim(Container_Kind::Brace), "{".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '}'
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '['
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Open_Delim(Container_Kind::Bracket), "[".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == ']'
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Close_Delim(Container_Kind::Bracket), "]".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 8. Numbers (including units like pt, em, cm, mm, in, fr, %, deg, rad)
            if Ch.is_ascii_digit() || (Ch == '.' && self.PeekAhead(1).map_or(false, | c | c.is_ascii_digit()))
            {
                let NumText = self.ScanNumber();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Number(NumText.clone()), NumText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 9. Identifiers (starts with alpha/underscore, supports hyphens when followed by alpha/underscore)
            if Ch.is_alphabetic() || Ch == '_'
            {
                let IdentText = self.ScanIdentifier();
                let Kind = if IsTypstKeyword( & IdentText)
                {
                    Token_Kind::Keyword(IdentText.clone())
                }
                else
                {
                    Token_Kind::Identifier(IdentText.clone())
                };
                RawTokens.push(Token_Info::NewWithIndent(Kind, IdentText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 10. Code switch marker #
            if Ch == '#'
            {
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Operator("#".to_string()), "#".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 11. Punctuation
            if Ch == ';' || Ch == ',' || Ch == ':'
            {
                if Ch == ':' && self.PeekAhead(1) == Some(':')
                {
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Operator("::".to_string()), "::".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                    PrecedingNewlines = 0;
                    continue;
                }
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Punctuation(Ch), Ch.to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '.'
            {
                if self.PeekAhead(1) == Some('.')
                {
                    if self.PeekAhead(2) == Some('=')
                    {
                        self.Advance();
                        self.Advance();
                        self.Advance();
                        RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Operator("..=".to_string()), "..=".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                        PrecedingNewlines = 0;
                        continue;
                    }
                    if self.PeekAhead(2) == Some('.')
                    {
                        self.Advance();
                        self.Advance();
                        self.Advance();
                        RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Operator("...".to_string()), "...".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                        PrecedingNewlines = 0;
                        continue;
                    }
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Operator("..".to_string()), "..".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                    PrecedingNewlines = 0;
                    continue;
                }
                self.Advance();
                RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Punctuation('.'), ".".to_string(), StartLine, StartCol, PrecedingNewlines, TokenIndent));
                PrecedingNewlines = 0;
                continue;
            }

            // 12. Multi-character and single-character operators
            let OpText = self.ScanOperator();
            RawTokens.push(Token_Info::NewWithIndent(Token_Kind::Operator(OpText.clone()), OpText, StartLine, StartCol, PrecedingNewlines, TokenIndent));
            PrecedingNewlines = 0;
        }

        RawTokens
    }

    fn ScanLineComment( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek()
        {
            if Ch == '\n'
            {
                break ;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanNestedBlockComment( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap()); // '/'
        Result.push(self.Advance().unwrap()); // '*'
        let mut Depth = 1;

        while Depth > 0 && self.Index < self.Chars.len()
        {
            if self.Peek() == Some('/') && self.PeekAhead(1) == Some('*')
            {
                Result.push(self.Advance().unwrap());
                Result.push(self.Advance().unwrap());
                Depth += 1;
                continue;
            }
            if self.Peek() == Some('*') && self.PeekAhead(1) == Some('/')
            {
                Result.push(self.Advance().unwrap());
                Result.push(self.Advance().unwrap());
                Depth -= 1;
                continue;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanString( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap()); // '"'

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            {
                Result.push(self.Advance().unwrap());
                if self.Peek().is_some()
                {
                    Result.push(self.Advance().unwrap());
                }
                continue;
            }
            if Ch == '"'
            {
                Result.push(self.Advance().unwrap());
                break ;
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn ScanRawCode( & mut self) -> String
    {
        let mut Result = String:: new ();
        let mut BacktickCount = 0;
        while self.Peek() == Some('`')
        {
            Result.push(self.Advance().unwrap());
            BacktickCount += 1;
        }

        while self.Index < self.Chars.len()
        {
            if self.Peek() == Some('`')
            {
                let mut Match = true;
                for I in 0..BacktickCount
                {
                    if self.PeekAhead(I) != Some('`')
                    {
                        Match = false;
                        break ;
                    }
                }
                if Match
                {
                    for _ in 0..BacktickCount
                    {
                        Result.push(self.Advance().unwrap());
                    }
                    break ;
                }
            }
            Result.push(self.Advance().unwrap());
        }
        Result
    }

    fn TryScanLabel( & mut self) -> Option<String>
    {
        let mut Offset = 1;
        let mut HasIdent = false;

        while let Some(C) = self.PeekAhead(Offset)
        {
            if C == '>' && HasIdent
            {
                let mut LabelText = String:: new ();
                for _ in 0..= Offset
                {
                    LabelText.push(self.Advance().unwrap());
                }
                return Some(LabelText);
            }
            if C.is_alphanumeric() || C == '_' || C == ':' || C == '-'
            {
                HasIdent = true;
                Offset += 1;
            }
            else
            {
                break ;
            }
        }
        None
    }

    fn ScanNumber( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek()
        {
            if Ch.is_ascii_digit() || Ch == '.'
            {
                Result.push(self.Advance().unwrap());
            }
            else
            {
                break ;
            }
        }

        // Optional unit suffix: pt, em, cm, mm, in, fr, %, deg, rad
        if let Some(Ch) = self.Peek()
        {
            if Ch == '%'
            {
                Result.push(self.Advance().unwrap());
            }
            else if Ch.is_alphabetic()
            {
                let mut UnitOffset = 0;
                let mut UnitCandidate = String:: new ();
                while let Some(UC) = self.PeekAhead(UnitOffset)
                {
                    if UC.is_alphabetic()
                    {
                        UnitCandidate.push(UC);
                        UnitOffset += 1;
                    }
                    else
                    {
                        break ;
                    }
                }
                match UnitCandidate.as_str()
                {
                    "pt" | "em" | "cm" | "mm" | "in" | "fr" | "deg" | "rad" =>
                    {
                        for _ in 0..UnitOffset
                        {
                            Result.push(self.Advance().unwrap());
                        }
                    }
                    _ => {}
                }
            }
        }

        Result
    }

    fn ScanIdentifier( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek()
        {
            if Ch.is_alphanumeric() || Ch == '_'
            {
                Result.push(self.Advance().unwrap());
            }
            else if Ch == '-' && self.PeekAhead(1).map_or(false, | C | C.is_alphabetic() || C == '_')
            {
                Result.push(self.Advance().unwrap());
            }
            else
            {
                break ;
            }
        }
        Result
    }

    fn ScanOperator( & mut self) -> String
    {
        let First = match self.Advance()
        {
            Some(C) => C,
            None => return String::new (),
        };

        let Next = self.Peek();
        let _Next2 = self.PeekAhead(1);

        match First
        {
            '=' =>
            {
                if Next == Some('>')
                {
                    self.Advance();
                    "=>".to_string()
                }
                else if Next == Some('=')
                {
                    self.Advance();
                    "==".to_string()
                }
                else
                {
                    "=".to_string()
                }
            }
            '!' =>
            {
                if Next == Some('=')
                {
                    self.Advance();
                    "!=".to_string()
                }
                else
                {
                    "!".to_string()
                }
            }
            '<' =>
            {
                if Next == Some('=')
                {
                    self.Advance();
                    "<=".to_string()
                }
                else if Next == Some('-')
                {
                    self.Advance();
                    "<-".to_string()
                }
                else if Next == Some('<')
                {
                    self.Advance();
                    "<<".to_string()
                }
                else
                {
                    "<".to_string()
                }
            }
            '>' =>
            {
                if Next == Some('=')
                {
                    self.Advance();
                    ">=".to_string()
                }
                else if Next == Some('>')
                {
                    self.Advance();
                    ">>".to_string()
                }
                else
                {
                    ">".to_string()
                }
            }
            '-' =>
            {
                if Next == Some('>')
                {
                    self.Advance();
                    "->".to_string()
                }
                else
                {
                    "-".to_string()
                }
            }
            '+' => "+".to_string(),
            '*' => "*".to_string(),
            '/' => "/".to_string(),
            '\\' => "\\".to_string(),
            '&' =>
            {
                if Next == Some('&')
                {
                    self.Advance();
                    "&&".to_string()
                }
                else
                {
                    "&".to_string()
                }
            }
            '|' =>
            {
                if Next == Some('|')
                {
                    self.Advance();
                    "||".to_string()
                }
                else
                {
                    "|".to_string()
                }
            }
            _ => First.to_string(),
        }
    }
}

fn IsTypstKeyword(Word: & str) -> bool
{
    matches!
    (
        Word,
        "let" | "set" | "show" | "import" | "include" | "context" | "return" | "if" | "else" | "while" | "for"
        | "in" | "not" | "and" | "or" | "as" | "break" | "continue" | "none" | "auto" | "true" | "false"
    )
}

// ============================================================================
// TYPST CONTAINER BUILDER
// ============================================================================

pub struct Typst_Container_Builder
{
    Tokens: Vec<Token_Info>,
    Index: usize,
}

impl Typst_Container_Builder
{
    pub fn New(Tokens: Vec<Token_Info>) -> Self
    {
        Typst_Container_Builder { Tokens, Index: 0 }
    }

    pub fn BuildTree( & mut self) -> Result<Vec<Typst_Syntax_Node>, String>
    {
        let mut RootNodes = Vec:: new ();

        while self.Index < self.Tokens.len()
        {
            let Node = self.ParseNode()?;
            RootNodes.push(Node);
        }

        Ok(RootNodes)
    }

    fn ParseNode( & mut self) -> Result<Typst_Syntax_Node, String>
    {
        let Token = self.Tokens[self.Index].clone();

        match & Token.Kind
        {
            Token_Kind::Open_Delim(Kind) =>
            {
                let DelimKind = * Kind;
                self.Index += 1;
                let mut Container = Typst_Container_Node::New(DelimKind, Token);

                while self.Index < self.Tokens.len()
                {
                    let Next = & self.Tokens[self.Index];

                    if let Token_Kind::Close_Delim(CloseKind) = & Next.Kind
                    {
                        if * CloseKind == DelimKind
                        {
                            Container.Close_Token = Some(Next.clone());
                            self.Index += 1;
                            break ;
                        }
                        else
                        {
                            return Err
                            (
                                format!
                                (
                                    "E_MISMATCH: Mismatched delimiter: opener '{}' at line {}, column {} closed by incompatible '{}' at line {}, column {}",
                                    Container.Open_Token.Text, Container.Open_Token.Line, Container.Open_Token.Column,
                                    Next.Text, Next.Line, Next.Column
                                )
                            );
                        }
                    }

                    if matches!(Next.Kind, Token_Kind::Comment_Line(_) | Token_Kind::Comment_Block(_))
                    {
                        Container.Has_Forced_Break = true;
                    }
                    if Next.Preceding_Newlines >= 1
                    {
                        Container.Has_Forced_Break = true;
                    }
                    if Next.Text == ";"
                    {
                        Container.Statement_Count += 1;
                    }

                    let ChildNode = self.ParseNode()?;
                    if let Typst_Syntax_Node::Container(ref ChildC) = ChildNode
                    {
                        if ChildC.Has_Forced_Break
                        {
                            Container.Has_Forced_Break = true;
                        }
                    }
                    Container.Children.push(ChildNode);
                }

                if Container.Close_Token.is_none()
                {
                    return Err
                    (
                        format!
                        (
                            "E_UNCLOSED: Unclosed delimiter '{}' at line {}, column {} reaches end of file",
                            Container.Open_Token.Text, Container.Open_Token.Line, Container.Open_Token.Column
                        )
                    );
                }

                // Collect tail glue (',', ';', '=') following closing delimiter
                while self.Index < self.Tokens.len()
                {
                    let Next = & self.Tokens[self.Index];
                    if Next.Preceding_Newlines > 0
                    {
                        break ;
                    }
                    if Next.Text == "," || Next.Text == ";" || Next.Text == "="
                    {
                        Container.Tail_Glue.push(Next.clone());
                        self.Index += 1;
                    }
                    else
                    {
                        break ;
                    }
                }

                Ok(Typst_Syntax_Node::Container(Container))
            }
            Token_Kind::Close_Delim(_) =>
            {
                Err
                (
                    format!
                    (
                        "E_ORPHAN: Stray closing delimiter '{}' at line {}, column {} has no opening partner",
                        Token.Text, Token.Line, Token.Column
                    )
                )
            }
            _ =>
            {
                self.Index += 1;
                Ok(Typst_Syntax_Node::Token(Token))
            }
        }
    }
}

// ============================================================================
// TYPST EMITTER
// ============================================================================

pub struct Typst_Emitter<'a>
{
    Config: & 'a Config_Settings,
    Output: String,
    Current_Line_Len: usize,
    Line_Has_Content: bool,
}

impl <'a> Typst_Emitter<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self
    {
        Typst_Emitter
        {
            Config,
            Output: String::new (),
            Current_Line_Len: 0,
            Line_Has_Content: false,
        }
    }

    pub fn Emit(mut self, Nodes: & [Typst_Syntax_Node]) -> String
    {
        self.EmitNodeList(Nodes, 0);
        if ! self.Output.ends_with('\n') && ! self.Output.is_empty()
        {
            self.Output.push('\n');
        }
        self.Output
    }

    fn WriteNewline( & mut self)
    {
        while self.Output.ends_with(' ') || self.Output.ends_with('\t')
        {
            self.Output.pop();
        }
        self.Output.push('\n');
        self.Current_Line_Len = 0;
        self.Line_Has_Content = false;
    }

    fn EnsureNewline( & mut self)
    {
        if self.Line_Has_Content
        {
            self.WriteNewline();
        }
    }

    fn EnsureBlankLine( & mut self)
    {
        if self.Config.Max_Blank_Lines == 0 || self.Output.is_empty()
        {
            return ;
        }
        self.EnsureNewline();
        if ! self.Output.ends_with("\n\n")
        {
            self.Output.push('\n');
            self.Current_Line_Len = 0;
            self.Line_Has_Content = false;
        }
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
        {
            self.Current_Line_Len += Text.len();
            self.Line_Has_Content = true;
        }
    }

    fn EmitSpace( & mut self)
    {
        if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
        {
            self.Output.push(' ');
            self.Current_Line_Len += 1;
        }
    }

    fn DecideLayout( & self, Container: & Typst_Container_Node, CurrentIndentLevel: usize, CurrentColumn: usize) -> Layout_State
    {
        // 1. Monolith First
        if ! Container.Has_Forced_Break && Container.Statement_Count <= self.Config.Max_Statements_In_Monolith
        && Container.CalculateBlockDepth() <= self.Config.Max_Block_Depth
        && Container.CountTotalContainers() <= self.Config.Max_Containers_In_Monolith
        {
            let SingleSpan = Container.CalculateSingleLineSpan();
            if SingleSpan != usize::MAX
            {
                let BaseCol = if CurrentColumn > 0 { CurrentColumn } else { CurrentIndentLevel * self.Config.Indent_Width };
                let ProjectedCol = BaseCol + SingleSpan;
                if ProjectedCol <= self.Config.Canvas_Width
                {
                    return Layout_State::Monolith;
                }
                if Container.Statement_Count == 0 && SingleSpan <= self.Config.Canvas_Width
                {
                    return Layout_State::Monolith;
                }
            }
        }

        // 2. Container Grouping / Simplified Symmetry
        if self.Config.Allow_Brace_Grouping && Container.IsSoleChildChain()
        {
            let Chain = Container.CollectChain();
            if Chain.len() >= 2
            {
                return Layout_State::Grouped;
            }
        }

        // 3. Expanded Symmetry
        Layout_State::Expanded
    }

    fn EmitNodeList( & mut self, Nodes: & [Typst_Syntax_Node], Level: usize)
    {
        let Total = Nodes.len();
        let mut Index = 0;

        while Index < Total
        {
            let Node = & Nodes[Index];

            match Node
            {
                Typst_Syntax_Node::Token(T) =>
                {
                    self.EmitSingleToken(T, Level);
                }
                Typst_Syntax_Node::Container(C) =>
                {
                    let IsPrecededByEquals = Index > 0 && match & Nodes[Index - 1]
                    {
                        Typst_Syntax_Node::Token(PrevT) => PrevT.Text == "=",
                        _ => false,
                    };

                    let Layout = self.DecideLayout(C, Level, self.Current_Line_Len);

                    match Layout
                    {
                        Layout_State::Monolith =>
                        {
                            if C.Open_Token.Preceding_Newlines > 0 && ! IsPrecededByEquals
                            {
                                self.EnsureNewline();
                                if C.Open_Token.Preceding_Newlines >= 2 && self.Config.Preserve_Empty_Lines
                                {
                                    self.EnsureBlankLine();
                                }
                            }
                            self.EmitMonolith(C, Level);
                        }
                        Layout_State::Grouped =>
                        {
                            if C.Open_Token.Preceding_Newlines >= 2 && self.Config.Preserve_Empty_Lines
                            {
                                self.EnsureBlankLine();
                            }
                            self.EmitGrouped(C, Level);
                        }
                        Layout_State::Expanded =>
                        {
                            if C.Open_Token.Preceding_Newlines >= 2 && self.Config.Preserve_Empty_Lines
                            {
                                self.EnsureBlankLine();
                            }
                            self.EmitExpanded(C, Level);
                        }
                    }
                }
            }
            Index += 1;
        }
    }

    fn EmitSingleToken( & mut self, Token: & Token_Info, Level: usize)
    {
        if Token.Preceding_Newlines > 0
        {
            self.EnsureNewline();
            if Token.Preceding_Newlines >= 2 && self.Config.Preserve_Empty_Lines
            {
                self.EnsureBlankLine();
            }
        }

        if let Token_Kind::Comment_Line(CommentText) = & Token.Kind
        {
            if self.Line_Has_Content
            {
                self.EmitSpace();
            }
            else
            {
                self.EmitIndent(Level);
            }
            self.EmitRaw(CommentText);
            self.WriteNewline();
            return ;
        }

        if let Token_Kind::Comment_Block(CommentText) = & Token.Kind
        {
            if ! self.Line_Has_Content
            {
                self.EmitIndent(Level);
            }
            else
            {
                self.EmitSpace();
            }
            self.EmitRaw(CommentText);
            return ;
        }

        if ! self.Line_Has_Content
        {
            self.EmitIndent(Level);
        }

        match & Token.Kind
        {
            Token_Kind::Punctuation(';') =>
            {
                while self.Output.ends_with(' ')
                {
                    self.Output.pop();
                    self.Current_Line_Len -= 1;
                }
                self.EmitRaw(";");
                self.EmitSpace();
            }
            Token_Kind::Punctuation(',') =>
            {
                while self.Output.ends_with(' ')
                {
                    self.Output.pop();
                    self.Current_Line_Len -= 1;
                }
                self.EmitRaw(",");
                self.EmitSpace();
            }
            Token_Kind::Punctuation(':') =>
            {
                while self.Output.ends_with(' ')
                {
                    self.Output.pop();
                    self.Current_Line_Len -= 1;
                }
                self.EmitRaw(":");
                self.EmitSpace();
            }
            Token_Kind::Punctuation('.') =>
            {
                while self.Output.ends_with(' ')
                {
                    self.Output.pop();
                    self.Current_Line_Len -= 1;
                }
                self.EmitRaw(".");
            }
            Token_Kind::Operator(Op) =>
            {
                if Op == "#"
                {
                    // '#' binds directly to the following identifier or keyword without spaces
                    self.EmitRaw("#");
                }
                else if Op == ".." || Op == "..=" || Op == "..."
                {
                    while self.Output.ends_with(' ')
                    {
                        self.Output.pop();
                        self.Current_Line_Len -= 1;
                    }
                    self.EmitRaw(Op);
                }
                else if Op == "::"
                {
                    while self.Output.ends_with(' ')
                    {
                        self.Output.pop();
                        self.Current_Line_Len -= 1;
                    }
                    self.EmitRaw("::");
                }
                else if Op == "$"
                {
                    // Math delimiter
                    if self.Output.ends_with(' ') || ! self.Line_Has_Content
                    {
                        self.EmitRaw("$");
                    }
                    else
                    {
                        self.EmitSpace();
                        self.EmitRaw("$");
                    }
                }
                else if Op == "\\"
                {
                    self.EmitRaw("\\");
                }
                else
                {
                    self.EmitSpace();
                    self.EmitRaw(Op);
                    self.EmitSpace();
                }
            }
            Token_Kind::Keyword(K) =>
            {
                if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('#') && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(K);
                self.EmitSpace();
            }
            Token_Kind::Identifier(Ident) =>
            {
                if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('#') && ! self.Output.ends_with('.') && ! self.Output.ends_with("::")
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with('$') && ! self.Output.ends_with('@')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(Ident);
            }
            Token_Kind::Number(Num) =>
            {
                if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with(':') && ! self.Output.ends_with('=') && ! self.Output.ends_with('+')
                && ! self.Output.ends_with('-') && ! self.Output.ends_with('*') && ! self.Output.ends_with('/')
                && ! self.Output.ends_with('$')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(Num);
            }
            Token_Kind::String_Lit(S) =>
            {
                if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
                && ! self.Output.ends_with(':') && ! self.Output.ends_with('=') && ! self.Output.ends_with('#')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(S);
            }
            _ =>
            {
                self.EmitRaw( & Token.Text);
            }
        }
    }

    fn EmitMonolith( & mut self, Container: & Typst_Container_Node, Level: usize)
    {
        if ! self.Line_Has_Content
        {
            self.EmitIndent(Level);
        }

        let NeedSpace = self.Line_Has_Content
        && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
        && ! self.Output.ends_with('#') && ! self.Output.ends_with('.') && ! self.Output.ends_with("::")
        && ! self.Output.ends_with('(') && ! self.Output.ends_with('[') && ! self.Output.ends_with('{')
        && (Container.Kind == Container_Kind::Brace || (Container.Kind == Container_Kind::Bracket && self.Output.ends_with('=')));

        if NeedSpace
        {
            self.EmitSpace();
        }

        self.EmitRaw(Container.Kind.OpenGlyph());

        if Container.Kind == Container_Kind::Brace && ! Container.Children.is_empty()
        {
            self.EmitRaw(" ");
        }

        for Child in & Container.Children
        {
            match Child
            {
                Typst_Syntax_Node::Token(T) =>
                {
                    if T.Text == ";"
                    {
                        self.EmitRaw(";");
                        self.EmitSpace();
                    }
                    else if T.Text == ","
                    {
                        self.EmitRaw(",");
                        self.EmitSpace();
                    }
                    else if T.Text == ":"
                    {
                        self.EmitRaw(":");
                        if Container.Kind != Container_Kind::Paren || Container.Children.len() > 1
                        {
                            self.EmitSpace();
                        }
                    }
                    else if T.Text == "."
                    {
                        while self.Output.ends_with(' ')
                        {
                            self.Output.pop();
                            self.Current_Line_Len -= 1;
                        }
                        self.EmitRaw(".");
                    }
                    else if T.Text == "#"
                    {
                        self.EmitRaw("#");
                    }
                    else if T.Text == "," || T.Text == ";" || T.Text == ":"
                    {
                        while self.Output.ends_with(' ')
                        {
                            self.Output.pop();
                            self.Current_Line_Len -= 1;
                        }
                        self.EmitRaw( & T.Text);
                        self.EmitSpace();
                    }
                    else if let Token_Kind::Operator(ref Op) = T.Kind
                    {
                        if Op == ".." || Op == "..=" || Op == "..."
                        {
                            while self.Output.ends_with(' ')
                            {
                                self.Output.pop();
                                self.Current_Line_Len -= 1;
                            }
                            self.EmitRaw(Op);
                        }
                        else
                        {
                            self.EmitSpace();
                            self.EmitRaw(Op);
                            self.EmitSpace();
                        }
                    }
                    else
                    {
                        let NeedInnerSpace = if self.Output.ends_with('(') || self.Output.ends_with('[') || self.Output.ends_with('{')
                        || self.Output.ends_with(' ') || self.Output.ends_with('#') || self.Output.ends_with('.')
                        {
                            false
                        }
                        else
                        {
                            true
                        };

                        if NeedInnerSpace
                        {
                            self.EmitSpace();
                        }
                        self.EmitRaw( & T.Text);
                        if matches!(T.Kind, Token_Kind::Keyword(_))
                        {
                            self.EmitSpace();
                        }
                    }
                }
                Typst_Syntax_Node::Container(SubC) =>
                {
                    self.EmitMonolith(SubC, Level);
                }
            }
        }

        if Container.Kind == Container_Kind::Brace && ! Container.Children.is_empty() && ! self.Output.ends_with(' ')
        {
            self.EmitRaw(" ");
        }

        if Container.Close_Token.is_some()
        {
            if (Container.Kind == Container_Kind::Paren || Container.Kind == Container_Kind::Bracket) && self.Output.ends_with(' ')
            {
                self.Output.pop();
                self.Current_Line_Len -= 1;
            }

            self.EmitRaw(Container.Kind.CloseGlyph());

            for Glue in & Container.Tail_Glue
            {
                if Glue.Text == "="
                {
                    self.EmitSpace();
                    self.EmitRaw("=");
                    self.EmitSpace();
                }
                else
                {
                    self.EmitRaw( & Glue.Text);
                }
            }
        }
    }

    fn EmitGrouped( & mut self, Container: & Typst_Container_Node, Level: usize)
    {
        let Chain = Container.CollectChain();
        let OpenRun: String = Chain.iter().map( | C | C.Kind.OpenGlyph()).collect();
        let CloseRun: String = Chain.iter().rev().filter( | C | C.Close_Token.is_some()).map( | C | C.Kind.CloseGlyph()).collect();
        let Innermost = Chain.last().unwrap();

        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw( & OpenRun);
        self.WriteNewline();

        self.EmitNodeList( & Innermost.Children, Level + 1);

        if ! CloseRun.is_empty()
        {
            self.EnsureNewline();
            let HasPrecedingBlank = Container.Close_Token.as_ref().map_or(false, | T | T.Preceding_Newlines >= 2);
            if HasPrecedingBlank && self.Config.Preserve_Empty_Lines
            {
                self.EnsureBlankLine();
            }
            else if self.Output.ends_with("\n\n")
            {
                self.Output.pop();
            }

            self.EmitIndent(Level);
            self.EmitRaw( & CloseRun);

            for Glue in & Container.Tail_Glue
            {
                if Glue.Text == "="
                {
                    self.EmitSpace();
                    self.EmitRaw("=");
                    self.EmitSpace();
                }
                else
                {
                    self.EmitRaw( & Glue.Text);
                }
            }
            self.WriteNewline();
        }
    }

    fn EmitExpanded( & mut self, Container: & Typst_Container_Node, Level: usize)
    {
        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();

        self.EmitNodeList( & Container.Children, Level + 1);

        if Container.Close_Token.is_some()
        {
            self.EnsureNewline();
            let HasPrecedingBlank = Container.Close_Token.as_ref().map_or(false, | T | T.Preceding_Newlines >= 2);
            if HasPrecedingBlank && self.Config.Preserve_Empty_Lines
            {
                self.EnsureBlankLine();
            }
            else if self.Output.ends_with("\n\n")
            {
                self.Output.pop();
            }

            self.EmitIndent(Level);
            self.EmitRaw(Container.Kind.CloseGlyph());

            for Glue in & Container.Tail_Glue
            {
                if Glue.Text == "="
                {
                    self.EmitSpace();
                    self.EmitRaw("=");
                    self.EmitSpace();
                }
                else
                {
                    self.EmitRaw( & Glue.Text);
                }
            }
            self.WriteNewline();
        }
    }
}

// ============================================================================
// VERIFICATION
// ============================================================================

fn VerifyTypstTokenEquivalence(Source: & str, Formatted: & str) -> Result<(), String>
{
    let mut SourceLexer = Typst_Lexer::New(Source);
    let SourceTokens = SourceLexer.TokenizeAll();

    let mut FormattedLexer = Typst_Lexer::New(Formatted);
    let FormattedTokens = FormattedLexer.TokenizeAll();

    let CleanSource: Vec< & Token_Info> = SourceTokens.iter().filter( | T | ! T.IsTrivia()).collect();
    let CleanFormatted: Vec< & Token_Info> = FormattedTokens.iter().filter( | T | ! T.IsTrivia()).collect();

    if CleanSource.len() != CleanFormatted.len()
    {
        return Err
        (
            format!
            (
                "E_VERIFY: Token count mismatch: original has {} non-trivia tokens, formatted has {}",
                CleanSource.len(), CleanFormatted.len()
            )
        );
    }

    for (I, (Orig, Form)) in CleanSource.iter().zip(CleanFormatted.iter()).enumerate()
    {
        if Orig.Text != Form.Text
        {
            return Err
            (
                format!
                (
                    "E_VERIFY: Token mismatch at non-trivia index {}: original '{}' (line {}, col {}) vs formatted '{}' (line {}, col {})",
                    I, Orig.Text, Orig.Line, Orig.Column, Form.Text, Form.Line, Form.Column
                )
            );
        }
    }

    Ok(())
}

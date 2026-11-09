#![allow(
non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
use crate::types:: { Config_Settings, Container_Kind, Kotlin_Style, Layout_State, Source_Language, Token_Info, Token_Kind, };

pub struct Kotlin_Formatter;

impl Kotlin_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let EffectiveStyle = if Config.Language == Source_Language::KotlinOfficial { Kotlin_Style::Official }
        else { Config.Kotlin_Style };

        let mut Lexer = Kotlin_Lexer::New(Source);
        let Tokens = Lexer.TokenizeAll();

        let mut Builder = Kotlin_Container_Builder::New(Tokens);
        let Nodes = Builder.BuildTree();

        let Emitter = Kotlin_Emitter::New(Config, EffectiveStyle);
        let Formatted = Emitter.Emit( & Nodes);

        // Safety gate verification
        if let Err(VerifyErr) = VerifyKotlinTokenEquivalence(Source, & Formatted)
        { return Err(format ! ("{}. Output aborted to prevent code corruption.", VerifyErr)); }

        Ok(Formatted)
    }

    pub fn IsFormatted(Source: & str, Config: & Config_Settings) -> bool { match Self::Format(Source, Config) { Ok(Formatted) => Formatted == Source, Err(_) => false, } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kotlin_Container_Role
{
    // Braces { ... }
    Class_Body,
    Function_Body,
    Init_Block,
    Getter_Setter_Body,
    Control_Structure_Body,
    When_Body,
    Trailing_Lambda,
    Lambda_Literal,
    String_Template_Expr,

    // Parentheses ( ... )
    Call_Arguments,
    Function_Parameters,
    Constructor_Parameters,
    Control_Condition,
    Parenthesized_Expression,
    Parenthesized_Type,
    Destructuring_Declaration,

    // Brackets [ ... ]
    Indexing_Suffix,
    Collection_Literal,
    Annotation_List,

    // Angle brackets < ... >
    Type_Parameters,
    Type_Arguments,
}

#[derive(Debug, Clone)]
pub enum Kotlin_Syntax_Node { Token(Token_Info), Container(Kotlin_Container), }

#[derive(Debug, Clone)]
pub struct Kotlin_Container
{
    pub Kind: Container_Kind,
    pub Role: Kotlin_Container_Role,
    pub Open_Token: Token_Info,
    pub Close_Token: Option<Token_Info>,
    pub Children: Vec<Kotlin_Syntax_Node>,
    pub Tail_Glue: Vec<Token_Info>,
    pub Has_Forced_Break: bool,
    pub Statement_Count: usize,
    pub Is_Preceded_By_Return_Or_Throw: bool,
}

impl Kotlin_Container
{
    pub fn New(Kind: Container_Kind, Role: Kotlin_Container_Role, Open_Token: Token_Info, Is_Preceded_By_Return_Or_Throw: bool ,) -> Self
    {
        Kotlin_Container
        {
            Kind,
            Role,
            Open_Token,
            Close_Token: None,
            Children: Vec:: new (),
            Tail_Glue: Vec:: new (),
            Has_Forced_Break: false,
            Statement_Count: 0,
            Is_Preceded_By_Return_Or_Throw,
        }
    }

    pub fn ShouldHug( & self, Style: Kotlin_Style) -> bool
    {
        match Style
        {
            Kotlin_Style::Allman => false,
            Kotlin_Style::Official =>
            {
                if self.Is_Preceded_By_Return_Or_Throw { return true; }
                match self.Role
                {
                    Kotlin_Container_Role::Trailing_Lambda
                    | Kotlin_Container_Role::Call_Arguments
                    | Kotlin_Container_Role::Function_Parameters
                    | Kotlin_Container_Role::Constructor_Parameters
                    | Kotlin_Container_Role::Indexing_Suffix
                    | Kotlin_Container_Role::Type_Arguments
                    | Kotlin_Container_Role::String_Template_Expr => true,
                    _ => false,
                }
            }
        }
    }

    pub fn IsSoleChildChain( & self) -> bool
    {
        let NonCommentChildren: Vec <& Kotlin_Syntax_Node > = self
        .Children
        .iter()
        .filter( | Node | match Node { Kotlin_Syntax_Node::Token(T) => ! T.IsTrivia() && ! T.IsComment(), Kotlin_Syntax_Node::Container(_) => true, })
        .collect();

        if NonCommentChildren.len() == 1 { matches ! (NonCommentChildren[0], Kotlin_Syntax_Node::Container(_)) }
        else { false }
    }

    pub fn GetSoleChildContainer( & self) -> Option <& Kotlin_Container >
    {
        let NonCommentChildren: Vec <& Kotlin_Syntax_Node > = self
        .Children
        .iter()
        .filter( | Node | match Node { Kotlin_Syntax_Node::Token(T) => ! T.IsTrivia() && ! T.IsComment(), Kotlin_Syntax_Node::Container(_) => true, })
        .collect();

        if NonCommentChildren.len() == 1 { if let Kotlin_Syntax_Node::Container(C) = NonCommentChildren[0] { return Some(C); } }
        None
    }

    pub fn CollectChain( & self) -> Vec <& Kotlin_Container >
    {
        let mut Chain = vec![self];
        let mut Current = self;
        while let Some(Child) = Current.GetSoleChildContainer()
        { if Current.Kind == Container_Kind::Angle || Child.Kind == Container_Kind::Angle { break ; } Chain.push(Child); Current = Child; }
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
                Kotlin_Syntax_Node::Token(T) =>
                {
                    if T.IsTrivia() { continue ; }
                    if T.IsComment() && matches!(T.Kind, Token_Kind::Comment_Line(_)) { return usize::MAX; }
                    if PrevTokenNeedsSpace { Total += 1; }
                    Total += T.Text.len();
                    PrevTokenNeedsSpace = true;
                }
                Kotlin_Syntax_Node::Container(C) =>
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
            if let Kotlin_Syntax_Node::Container(Sub) = Child
            { let SubDepth = Sub.CalculateBlockDepth(); if SubDepth > MaxChildDepth { MaxChildDepth = SubDepth; } }
        }
        1 + MaxChildDepth
    }

    pub fn CountTotalContainers( & self) -> usize
    {
        let mut Count = 1;
        for Child in & self.Children { if let Kotlin_Syntax_Node::Container(Sub) = Child { Count += Sub.CountTotalContainers(); } }
        Count
    }
}

pub struct Kotlin_Lexer<'a> { Source: & 'a str, Chars: Vec<char>, Index: usize, Line: usize, Column: usize, }

impl <'a> Kotlin_Lexer<'a>
{
    pub fn New(Source: & 'a str) -> Self { Kotlin_Lexer { Source, Chars: Source.chars().collect(), Index: 0, Line: 1, Column: 0, } }

    fn Peek( & self) -> Option<char> { if self.Index < self.Chars.len() { Some(self.Chars[self.Index]) } else { None } }

    fn PeekAhead( & self, Offset: usize) -> Option<char>
    { let Target = self.Index + Offset; if Target < self.Chars.len() { Some(self.Chars[Target]) } else { None } }

    fn Advance( & mut self) -> Option<char>
    {
        if self.Index >= self.Chars.len() { return None; }
        let Ch = self.Chars[self.Index];
        self.Index += 1;
        if Ch == '\n' { self.Line += 1; self.Column = 0; }
        else { self.Column += 1; }
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

            // 3. Comments (with Kotlin nested block comment support)
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

            // 4. Raw Multiline Strings """...""" vs Standard Strings "..."
            if Ch == '"'
            {
                if self.PeekAhead(1) == Some('"') && self.PeekAhead(2) == Some('"')
                {
                    let RawStringText = self.ScanRawMultilineString();
                    RawTokens.push(Token_Info::New(Token_Kind::String_Lit(RawStringText.clone()), RawStringText, StartLine, StartCol, PrecedingNewlines,));
                }
                else
                {
                    let StringText = self.ScanStandardString();
                    RawTokens.push(Token_Info::New(Token_Kind::String_Lit(StringText.clone()), StringText, StartLine, StartCol, PrecedingNewlines,));
                }
                PrecedingNewlines = 0;
                continue;
            }

            // 5. Backticked Identifiers `identifier`
            if Ch == '`'
            {
                let IdentText = self.ScanBacktickIdentifier();
                RawTokens.push(Token_Info::New(Token_Kind::Identifier(IdentText.clone()), IdentText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            // 6. Character Literals 'c'
            if Ch == '\''
            {
                let CharText = self.ScanCharLiteral();
                RawTokens.push(Token_Info::New(Token_Kind::Char_Lit(CharText.clone()), CharText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            // 7. Delimiters
            if Ch == '('
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Paren),"(".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == ')'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Paren),")".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '{'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace),"{".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '}'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace),"}".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '['
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Bracket),"[".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == ']'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Bracket),"]".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            // 8. Numbers (Dec, Hex, Bin, Floats)
            if Ch.is_ascii_digit() || (Ch == '.' && self.PeekAhead(1).map_or(false, | C | C.is_ascii_digit()))
            {
                let NumberText = self.ScanNumber();
                RawTokens.push(Token_Info::New(Token_Kind::Number(NumberText.clone()), NumberText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            // 9. Identifiers and Keywords
            if Ch.is_alphabetic() || Ch == '_' || Ch == '$'
            {
                let IdentText = self.ScanIdentifier();
                let Kind = if IsKotlinKeyword( & IdentText) { Token_Kind::Keyword(IdentText.clone()) }
                else { Token_Kind::Identifier(IdentText.clone()) };
                RawTokens.push(Token_Info::New(Kind, IdentText, StartLine, StartCol, PrecedingNewlines));
                PrecedingNewlines = 0;
                continue;
            }

            // 10. Multi-Character and Single-Character Operators & Punctuation
            if Ch == '?' && self.PeekAhead(1) == Some('.')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("?.".to_string()),"?.".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '?' && self.PeekAhead(1) == Some(':')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("?:".to_string()),"?:".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '!' && self.PeekAhead(1) == Some('!')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("!!".to_string()),"!!".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == ':' && self.PeekAhead(1) == Some(':')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("::".to_string()),"::".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '.' && self.PeekAhead(1) == Some('.')
            {
                if self.PeekAhead(2) == Some('<')
                {
                    self.Advance();
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("..<".to_string()),"..<".to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    continue;
                }
                else
                {
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("..".to_string()),"..".to_string(), StartLine, StartCol, PrecedingNewlines,));
                    PrecedingNewlines = 0;
                    continue;
                }
            }
            if Ch == '-' && self.PeekAhead(1) == Some('>')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("->".to_string()),"->".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '=' && self.PeekAhead(1) == Some('=')
            {
                if self.PeekAhead(2) == Some('=')
                {
                    self.Advance();
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("===".to_string()),"===".to_string(), StartLine, StartCol, PrecedingNewlines,));
                }
                else
                {
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("==".to_string()),"==".to_string(), StartLine, StartCol, PrecedingNewlines,));
                }
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '!' && self.PeekAhead(1) == Some('=')
            {
                if self.PeekAhead(2) == Some('=')
                {
                    self.Advance();
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("!==".to_string()),"!== ".to_string(), StartLine, StartCol, PrecedingNewlines,));
                }
                else
                {
                    self.Advance();
                    self.Advance();
                    RawTokens.push(Token_Info::New(Token_Kind::Operator("!=".to_string()),"!=".to_string(), StartLine, StartCol, PrecedingNewlines,));
                }
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '<' && self.PeekAhead(1) == Some('=')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("<=".to_string()),"<=".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '>' && self.PeekAhead(1) == Some('=')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator(">=".to_string()),">=".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '&' && self.PeekAhead(1) == Some('&')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("&&".to_string()),"&&".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '|' && self.PeekAhead(1) == Some('|')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("||".to_string()),"||".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if (Ch == '+' || Ch == '-' || Ch == '*' || Ch == '/' || Ch == '%') && self.PeekAhead(1) == Some('=')
            {
                let OpText = format!("{}=", Ch);
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator(OpText.clone()), OpText, StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '+' && self.PeekAhead(1) == Some('+')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("++".to_string()),"++".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }
            if Ch == '-' && self.PeekAhead(1) == Some('-')
            {
                self.Advance();
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator("--".to_string()),"--".to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            // Punctuation and single operators
            if Ch == ';' || Ch == ',' || Ch == '.' || Ch == ':' || Ch == '?' || Ch == '@'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Punctuation(Ch), Ch.to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            if Ch == '=' || Ch == '+' || Ch == '-' || Ch == '*' || Ch == '/' || Ch == '%' || Ch == '!' || Ch == '<' || Ch == '>'
            {
                self.Advance();
                RawTokens.push(Token_Info::New(Token_Kind::Operator(Ch.to_string()), Ch.to_string(), StartLine, StartCol, PrecedingNewlines,));
                PrecedingNewlines = 0;
                continue;
            }

            // Fallback for any other characters
            self.Advance();
            RawTokens.push(Token_Info::New(Token_Kind::Punctuation(Ch), Ch.to_string(), StartLine, StartCol, PrecedingNewlines,));
            PrecedingNewlines = 0;
        }

        // Post-process to resolve generic angle brackets < > in Kotlin
        ResolveKotlinAngleBrackets( & mut RawTokens);

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
        let mut Depth = 1;

        while let Some(Ch) = self.Peek()
        {
            if Ch == '/' && self.PeekAhead(1) == Some('*')
            { Result.push(self.Advance().unwrap()); Result.push(self.Advance().unwrap()); Depth += 1; }
            else if Ch == '*' && self.PeekAhead(1) == Some('/')
            { Result.push(self.Advance().unwrap()); Result.push(self.Advance().unwrap()); Depth -= 1; if Depth == 0 { break ; } }
            else { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanRawMultilineString( & mut self) -> String
    {
        let mut Result = String:: new ();
        for _ in 0..3 { Result.push(self.Advance().unwrap()); }

        while let Some(Ch) = self.Peek()
        {
            if Ch == '"' && self.PeekAhead(1) == Some('"') && self.PeekAhead(2) == Some('"')
            {
                for _ in 0..3 { Result.push(self.Advance().unwrap()); }
                while self.Peek() == Some('"') { Result.push(self.Advance().unwrap()); }
                break;
            }
            else { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanStandardString( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // '"'

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            { Result.push(self.Advance().unwrap()); if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); } }
            else if Ch == '"' { Result.push(self.Advance().unwrap()); break ; }
            else { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanBacktickIdentifier( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // '`'

        while let Some(Ch) = self.Peek()
        {
            if Ch == '`' { Result.push(self.Advance().unwrap()); break ; }
            else if Ch == '\n' || Ch == '\r' { break ; }
            else { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanCharLiteral( & mut self) -> String
    {
        let mut Result = String:: new ();
        Result.push(self.Advance().unwrap());
        // '\''

        while let Some(Ch) = self.Peek()
        {
            if Ch == '\\'
            { Result.push(self.Advance().unwrap()); if let Some(_) = self.Peek() { Result.push(self.Advance().unwrap()); } }
            else if Ch == '\'' { Result.push(self.Advance().unwrap()); break ; }
            else if Ch == '\n' || Ch == '\r' { break ; }
            else { Result.push(self.Advance().unwrap()); }
        }
        Result
    }

    fn ScanIdentifier( & mut self) -> String
    {
        let mut Result = String:: new ();
        while let Some(Ch) = self.Peek() { if Ch.is_alphanumeric() || Ch == '_' || Ch == '$' { Result.push(self.Advance().unwrap()); } else { break ; } }
        Result
    }

    fn ScanNumber( & mut self) -> String
    {
        let mut Result = String:: new ();
        if self.Peek() == Some('0')
        {
            if let Some(Next) = self.PeekAhead(1)
            {
                if Next == 'x' || Next == 'X'
                {
                    Result.push(self.Advance().unwrap());
                    Result.push(self.Advance().unwrap());
                    while let Some(Ch) = self.Peek() { if Ch.is_ascii_hexdigit() || Ch == '_' { Result.push(self.Advance().unwrap()); } else { break ; } }
                    if let Some(Suffix) = self.Peek() { if Suffix == 'L' || Suffix == 'u' || Suffix == 'U' { Result.push(self.Advance().unwrap()); } }
                    return Result;
                }
                else if Next == 'b' || Next == 'B'
                {
                    Result.push(self.Advance().unwrap());
                    Result.push(self.Advance().unwrap());
                    while let Some(Ch) = self.Peek() { if Ch == '0' || Ch == '1' || Ch == '_' { Result.push(self.Advance().unwrap()); } else { break ; } }
                    if let Some(Suffix) = self.Peek() { if Suffix == 'L' || Suffix == 'u' || Suffix == 'U' { Result.push(self.Advance().unwrap()); } }
                    return Result;
                }
            }
        }

        while let Some(Ch) = self.Peek()
        {
            if Ch.is_ascii_digit() || Ch == '_' { Result.push(self.Advance().unwrap()); }
            else if Ch == '.'
            {
                if self.PeekAhead(1) == Some('.')
                {
                    // Range operator `..` or `..<`, not a decimal point!
                    break;
                }
                if let Some(Next) = self.PeekAhead(1)
                {
                    if Next.is_ascii_digit()
                    {
                        Result.push(self.Advance().unwrap());
                        while let Some(D) = self.Peek() { if D.is_ascii_digit() || D == '_' { Result.push(self.Advance().unwrap()); } else { break ; } }
                    }
                    else { break ; }
                }
                else { break ; }
            }
            else if Ch == 'e' || Ch == 'E'
            {
                Result.push(self.Advance().unwrap());
                if let Some(Sign) = self.Peek() { if Sign == '+' || Sign == '-' { Result.push(self.Advance().unwrap()); } }
                while let Some(D) = self.Peek() { if D.is_ascii_digit() || D == '_' { Result.push(self.Advance().unwrap()); } else { break ; } }
            }
            else if matches!(Ch, 'f' | 'F' | 'L' | 'u' | 'U') { Result.push(self.Advance().unwrap()); break ; }
            else { break ; }
        }
        Result
    }
}

fn IsKotlinKeyword(Text: & str) -> bool
{
    matches!
    (
        Text,
        "as" | "as?" | "break" | "class" | "continue" | "do" | "else" | "false" | "for"
        | "fun" | "if" | "in" | "!in" | "interface" | "is" | "!is" | "null" | "object"
        | "package" | "return" | "super" | "this" | "throw" | "true" | "try" | "typealias"
        | "val" | "var" | "when" | "while" | "by" | "catch" | "constructor" | "delegate"
        | "dynamic" | "field" | "file" | "finally" | "get" | "import" | "init" | "param"
        | "property" | "receiver" | "set" | "setparam" | "value" | "where" | "abstract"
        | "actual" | "annotation" | "companion" | "const" | "crossinline" | "data"
        | "enum" | "expect" | "external" | "final" | "infix" | "inline" | "inner"
        | "internal" | "lateinit" | "noinline" | "open" | "operator" | "out" | "override"
        | "private" | "protected" | "public" | "reified" | "sealed" | "suspend"
        | "tailrec" | "vararg"
    )
}

fn ResolveKotlinAngleBrackets(Tokens: & mut Vec<Token_Info>)
{
    let mut I = 0;
    while I < Tokens.len()
    {
        if Tokens[I].Text == "<"
        {
            let HasValidLeft = if I > 0
            {
                match & Tokens[I - 1].Kind
                {
                    Token_Kind::Identifier(_) => true,
                    Token_Kind::Keyword(K) => matches!(K.as_str(),"class" | "interface" | "fun" | "typealias" | "where" | "is" | "as" | "as?"),
                    Token_Kind::Operator(Op) if Op == "::" => true,
                    _ => false,
                }
            }
            else { false };

            if HasValidLeft
            {
                let mut Depth = 1;
                let mut MatchIndex = None;
                let mut J = I + 1;

                while J < Tokens.len() && J - I < 80
                {
                    if Tokens[J].Text == "<" { Depth += 1; }
                    else if Tokens[J].Text == ">" { Depth -= 1; if Depth == 0 { MatchIndex = Some(J); break ; } }
                    else if matches!(Tokens[J].Text.as_str(),";" | "{" | "}") { break ; }
                    J += 1;
                }

                if let Some(M) = MatchIndex
                {
                    let FollowValid = if M + 1 < Tokens.len()
                    {
                        let Next = & Tokens[M + 1];
                        matches!(Next.Text.as_str(),"(" | ")" | "]" | "}" | ":" | ";" | "," | "." | "?" | "==" | "!=" | "[" | "{" | "::" | "->" | ">") || matches!(Next.Kind, Token_Kind::Identifier(_) | Token_Kind::Keyword(_))
                        || Next.Preceding_Newlines > 0
                    }
                    else { true };

                    if FollowValid
                    {
                        Tokens[I].Kind = Token_Kind::Open_Delim(Container_Kind::Angle);
                        Tokens[M].Kind = Token_Kind::Close_Delim(Container_Kind::Angle);
                    }
                }
            }
        }
        I += 1;
    }
}

pub struct Kotlin_Container_Builder { Tokens: Vec<Token_Info>, Index: usize, }

impl Kotlin_Container_Builder
{
    pub fn New(Tokens: Vec<Token_Info>) -> Self { Kotlin_Container_Builder { Tokens, Index: 0 } }

    pub fn BuildTree( & mut self) -> Vec<Kotlin_Syntax_Node>
    {
        let mut RootNodes = Vec:: new ();
        while self.Index < self.Tokens.len() { if let Some(Node) = self.ParseNextNode() { RootNodes.push(Node); } }
        RootNodes
    }

    fn ParseNextNode( & mut self) -> Option<Kotlin_Syntax_Node>
    {
        if self.Index >= self.Tokens.len() { return None; }
        let Token = self.Tokens[self.Index].clone();
        self.Index += 1;

        match & Token.Kind
        {
            Token_Kind::Open_Delim(Kind) =>
            {
                let Role = self.DetermineContainerRole( * Kind);
                let IsPrecededByReturn = self.CheckPrecededByReturnOrThrow();
                let mut Container = Kotlin_Container::New( * Kind, Role, Token, IsPrecededByReturn);
                self.PopulateContainerChildren( & mut Container);
                Some(Kotlin_Syntax_Node::Container(Container))
            }
            _ => Some(Kotlin_Syntax_Node::Token(Token)),
        }
    }

    fn PopulateContainerChildren( & mut self, Container: & mut Kotlin_Container)
    {
        let mut StatementCount = 0;
        let mut IsFirstToken = true;

        while self.Index < self.Tokens.len()
        {
            let CurrentToken = self.Tokens[self.Index].clone();

            if CurrentToken.Preceding_Newlines > 0 { Container.Has_Forced_Break = true; if ! IsFirstToken { StatementCount += 1; } }
            IsFirstToken = false;

            match & CurrentToken.Kind
            {
                Token_Kind::Close_Delim(CloseKind) if * CloseKind == Container.Kind =>
                {
                    self.Index += 1;
                    Container.Close_Token = Some(CurrentToken);
                    // Consume trailing glue (; or ,)
                    while self.Index < self.Tokens.len()
                    {
                        let NextToken = & self.Tokens[self.Index];
                        if NextToken.Preceding_Newlines == 0 && (NextToken.Text == ";" || NextToken.Text == ",")
                        { Container.Tail_Glue.push(NextToken.clone()); self.Index += 1; }
                        else { break ; }
                    }
                    break;
                }
                Token_Kind::Open_Delim(_) => { if let Some(ChildNode) = self.ParseNextNode() { Container.Children.push(ChildNode); } }
                Token_Kind::Comment_Line(_) =>
                { Container.Has_Forced_Break = true; Container.Children.push(Kotlin_Syntax_Node::Token(CurrentToken)); self.Index += 1; }
                _ =>
                {
                    if CurrentToken.Text == ";" { StatementCount += 1; }
                    Container.Children.push(Kotlin_Syntax_Node::Token(CurrentToken));
                    self.Index += 1;
                }
            }
        }

        Container.Statement_Count = StatementCount;
    }

    fn DetermineContainerRole( & self, Kind: Container_Kind) -> Kotlin_Container_Role
    {
        let PrevTokens = self.GetPreviousSignificantTokens(10);
        let Prev1 = PrevTokens.first();

        match Kind
        {
            Container_Kind::Brace =>
            {
                if let Some(T) = Prev1
                {
                    if T.Text == "init" { return Kotlin_Container_Role::Init_Block; }
                    if T.Text == "when" { return Kotlin_Container_Role::When_Body; }
                    if matches!(T.Text.as_str(),"else" | "do" | "try" | "finally") { return Kotlin_Container_Role::Control_Structure_Body; }
                    if matches!(T.Text.as_str(),"return" | "throw") { return Kotlin_Container_Role::Lambda_Literal; }
                    if T.Text == ")"
                    {
                        let PreParen = self.FindKeywordBeforeMatchingParen();
                        if let Some(K) = PreParen
                        {
                            if matches!(K.as_str(),"if" | "while" | "for" | "catch") { return Kotlin_Container_Role::Control_Structure_Body; }
                            if K == "when" { return Kotlin_Container_Role::When_Body; }
                            if K == "get" || K == "set" { return Kotlin_Container_Role::Getter_Setter_Body; }
                        }
                    }
                }

                // Check if inside class or function signature
                if self.IsPrecededByClassOrInterface() { return Kotlin_Container_Role::Class_Body; }
                if self.IsPrecededByFunDeclaration() { return Kotlin_Container_Role::Function_Body; }

                // If preceded by an identifier, call, or property access, it's a trailing lambda
                if let Some(T) = Prev1 { if matches ! (T.Kind, Token_Kind::Identifier(_)) || T.Text == ")" || T.Text == "]" { return Kotlin_Container_Role::Trailing_Lambda; } }

                Kotlin_Container_Role::Lambda_Literal
            }
            Container_Kind::Paren =>
            {
                if let Some(T) = Prev1
                {
                    if matches!(T.Text.as_str(),"if" | "while" | "for" | "when" | "catch") { return Kotlin_Container_Role::Control_Condition; }
                    if matches!(T.Text.as_str(),"val" | "var") { return Kotlin_Container_Role::Destructuring_Declaration; }
                }

                if self.IsPrecededByFunDeclaration() { return Kotlin_Container_Role::Function_Parameters; }
                if self.IsPrecededByClassDeclaration() { return Kotlin_Container_Role::Constructor_Parameters; }

                if let Some(T) = Prev1
                {
                    if matches!(T.Kind, Token_Kind::Identifier(_)) || T.Text == ")" || T.Text == "]" || T.Text == ">"
                    { return Kotlin_Container_Role::Call_Arguments; }
                }

                Kotlin_Container_Role::Parenthesized_Expression
            }
            Container_Kind::Bracket =>
            {
                if let Some(T) = Prev1
                {
                    if T.Text == "@" { return Kotlin_Container_Role::Annotation_List; }
                    if matches!(T.Kind, Token_Kind::Identifier(_)) || T.Text == ")" || T.Text == "]" { return Kotlin_Container_Role::Indexing_Suffix; }
                }
                Kotlin_Container_Role::Collection_Literal
            }
            Container_Kind::Angle =>
            {
                if self.IsPrecededByClassOrInterface() || self.IsPrecededByFunDeclaration() { Kotlin_Container_Role::Type_Parameters }
                else { Kotlin_Container_Role::Type_Arguments }
            }
        }
    }

    fn CheckPrecededByReturnOrThrow( & self) -> bool
    {
        let PrevTokens = self.GetPreviousSignificantTokens(2);
        if let Some(T) = PrevTokens.first() { matches ! (T.Text.as_str(),"return" | "throw") }
        else { false }
    }

    fn GetPreviousSignificantTokens( & self, MaxCount: usize) -> Vec<Token_Info>
    {
        let mut Result = Vec:: new ();
        let Start = if self.Index > 1 { self.Index - 2 } else { return Result; };
        let mut Current = Start;

        loop
        {
            let Tok = & self.Tokens[Current];
            if ! Tok.IsTrivia() && ! Tok.IsComment() { Result.push(Tok.clone()); if Result.len() >= MaxCount { break ; } }
            if Current == 0 { break ; }
            Current -= 1;
        }

        Result
    }

    fn FindKeywordBeforeMatchingParen( & self) -> Option<String>
    {
        if self.Index < 2 { return None; }
        let mut Scan = self.Index - 2;
        while Scan > 0 && (self.Tokens[Scan].IsTrivia() || self.Tokens[Scan].IsComment()) { Scan -= 1; }
        if self.Tokens[Scan].Text != ")" { return None; }

        let mut Depth = 1;
        while Scan > 0
        {
            Scan -= 1;
            if self.Tokens[Scan].Text == ")" { Depth += 1; }
            else if self.Tokens[Scan].Text == "(" { Depth -= 1; if Depth == 0 { break ; } }
        }

        if Depth == 0 && Scan > 0
        {
            let mut Before = Scan - 1;
            while Before > 0 && (self.Tokens[Before].IsTrivia() || self.Tokens[Before].IsComment()) { Before -= 1; }
            return Some(self.Tokens[Before].Text.clone());
        }

        None
    }

    fn IsPrecededByClassOrInterface( & self) -> bool
    {
        let PrevTokens = self.GetPreviousSignificantTokens(15);
        for T in & PrevTokens
        {
            if matches!(T.Text.as_str(),"class" | "interface" | "object" | "enum") { return true; }
            if matches!(T.Text.as_str(),";" | "{" | "}") { break ; }
        }
        false
    }

    fn IsPrecededByClassDeclaration( & self) -> bool
    {
        let PrevTokens = self.GetPreviousSignificantTokens(10);
        for T in & PrevTokens { if matches ! (T.Text.as_str(),"class" | "constructor") { return true; } if matches ! (T.Text.as_str(),";" | "{" | "}") { break ; } }
        false
    }

    fn IsPrecededByFunDeclaration( & self) -> bool
    {
        let PrevTokens = self.GetPreviousSignificantTokens(20);
        for T in & PrevTokens { if T.Text == "fun" { return true; } if matches ! (T.Text.as_str(),";" | "{" | "}") { break ; } }
        false
    }
}

pub struct Kotlin_Layout_Planner<'a> { Config: & 'a Config_Settings, }

impl <'a> Kotlin_Layout_Planner<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self { Kotlin_Layout_Planner { Config } }

    pub fn DecideLayout( & self, Container: & Kotlin_Container, CurrentIndentLevel: usize, CurrentColumn: usize,) -> Layout_State
    {
        // 1. Monolith First (Horizontal Density)
        if ! Container.Has_Forced_Break
        && Container.Statement_Count <= self.Config.Max_Statements_In_Monolith
        && Container.CalculateBlockDepth() <= self.Config.Max_Block_Depth
        && Container.CountTotalContainers() <= self.Config.Max_Containers_In_Monolith
        {
            let SingleSpan = Container.CalculateSingleLineSpan();
            if SingleSpan != usize::MAX
            {
                let BaseCol = if CurrentColumn > 0 { CurrentColumn }
                else { CurrentIndentLevel * self.Config.Indent_Width };
                let ProjectedCol = BaseCol + SingleSpan;
                if ProjectedCol <= self.Config.Canvas_Width { return Layout_State::Monolith; }
            }
        }

        // 2. Container Grouping Architecture (Sole-Child Chain Compound Delimiters)
        if self.Config.Allow_Brace_Grouping && Container.IsSoleChildChain()
        { let Chain = Container.CollectChain(); if Chain.len() >= 2 { return Layout_State::Grouped; } }

        // 3. Expanded Symmetry (Hierarchical Architecture)
        Layout_State::Expanded
    }
}

pub struct Kotlin_Emitter<'a>
{
    Config: & 'a Config_Settings,
    Planner: Kotlin_Layout_Planner<'a>,
    Style: Kotlin_Style,
    Output: String,
    Current_Line_Len: usize,
    Line_Has_Content: bool,
}

impl <'a> Kotlin_Emitter<'a>
{
    pub fn New(Config: & 'a Config_Settings, Style: Kotlin_Style) -> Self
    {
        Kotlin_Emitter { Config, Planner: Kotlin_Layout_Planner::New(Config), Style, Output: String::new (), Current_Line_Len: 0, Line_Has_Content: false, }
    }

    pub fn Emit(mut self, Nodes: & [Kotlin_Syntax_Node]) -> String
    { self.EmitNodeList(Nodes, 0); if ! self.Output.ends_with('\n') { self.Output.push('\n'); } self.Output }

    fn WriteNewline( & mut self)
    {
        while self.Output.ends_with(' ') { self.Output.pop(); }
        self.Output.push('\n');
        self.Current_Line_Len = 0;
        self.Line_Has_Content = false;
    }

    fn EnsureNewline( & mut self) { if self.Line_Has_Content { self.WriteNewline(); } }

    fn EnsureBlankLine( & mut self)
    {
        if self.Config.Max_Blank_Lines == 0 || self.Output.is_empty() { return ; }
        self.EnsureNewline();
        if ! self.Output.ends_with("\n\n")
        && ! self.Output.ends_with("{\n")
        && ! self.Output.ends_with("[\n")
        && ! self.Output.ends_with("(\n") { self.Output.push('\n'); self.Current_Line_Len = 0; self.Line_Has_Content = false; }
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
        if let Some(LastNl) = Text.rfind('\n') { self.Current_Line_Len = Text.len() - LastNl - 1; }
        else { self.Current_Line_Len += Text.len(); }
        self.Line_Has_Content = true;
    }

    fn EmitSpace( & mut self)
    {
        if self.Line_Has_Content
        && self.Current_Line_Len > 0
        && ! self.Output.ends_with(' ')
        && ! self.Output.ends_with('\n') { self.Output.push(' '); self.Current_Line_Len += 1; }
    }

    fn EmitNodeList( & mut self, Nodes: & [Kotlin_Syntax_Node], Level: usize)
    {
        let Total = Nodes.len();
        let mut Index = 0;

        while Index < Total
        {
            let Node = & Nodes[Index];

            match Node
            {
                Kotlin_Syntax_Node::Token(T) =>
                {
                    self.EmitSingleToken(T, Level);
                    if Level == 0 && (T.Text == ";" || T.Preceding_Newlines > 0 || Index + 1 == Total) { if Self::IsImportStatementEnding(Nodes, Index) && Self::HasMoreNonImportStatements(Nodes, Index + 1) { self.EnsureBlankLine(); } }
                }
                Kotlin_Syntax_Node::Container(C) =>
                {
                    let Layout = self.Planner.DecideLayout(C, Level, self.Current_Line_Len);
                    self.EmitContainerNodeWithLayout(C, Level, Layout);

                    // Check if followed by continuation keyword (else, catch, finally)
                    let NextContinuation = if Index + 1 < Total
                    {
                        match & Nodes[Index + 1] { Kotlin_Syntax_Node::Token(T) if matches ! (T.Text.as_str(),"else" | "catch" | "finally") => { Some(T.Text.clone()) } _ => None, }
                    }
                    else { None };

                    let NextIsPunctOrCall = if Index + 1 < Total
                    {
                        match & Nodes[Index + 1]
                        {
                            Kotlin_Syntax_Node::Token(T) => { matches ! (T.Text.as_str(),":" | "." | "?." | "::" | "," | ";") }
                            Kotlin_Syntax_Node::Container(NextC) => { NextC.Role == Kotlin_Container_Role::Trailing_Lambda && self.Style == Kotlin_Style::Official }
                        }
                    }
                    else { false };

                    if NextContinuation.is_some() { self.EnsureNewline(); }
                    else if NextIsPunctOrCall
                    {
                        // Keep on same line for `:`, `.`, `?.`, `::`, `,`, `;` or hugging trailing lambda
                    }
                    else if C.Tail_Glue.iter().any( | G | G.Text == ";") || C.Kind == Container_Kind::Brace || C.Kind == Container_Kind::Paren
                    { self.EnsureNewline(); }
                }
            }

            Index += 1;
        }
    }

    fn EmitSingleToken( & mut self, Token: & Token_Info, Level: usize)
    {
        if let Token_Kind::Comment_Line(CommentText) = & Token.Kind
        {
            if self.Line_Has_Content { self.EmitSpace(); }
            else { self.EmitIndent(Level); }
            self.EmitRaw(CommentText);
            self.WriteNewline();
            return ;
        }

        if let Token_Kind::Comment_Block(CommentText) = & Token.Kind
        { if ! self.Line_Has_Content { self.EmitIndent(Level); } else { self.EmitSpace(); } self.EmitRaw(CommentText); return ; }

        if Token.IsTrivia() { return ; }

        if Token.Preceding_Newlines > 0
        {
            let BlankLines = (Token.Preceding_Newlines - 1).min(self.Config.Max_Blank_Lines);
            self.EnsureNewline();
            for _ in 0..BlankLines { self.WriteNewline(); }
        }

        if ! self.Line_Has_Content { self.EmitIndent(Level); }
        else { self.EmitSpaceBeforeToken(Token); }

        self.EmitRaw( & Token.Text);

        if Token.Text == ";" { self.WriteNewline(); }
        else if let Token_Kind::Keyword(K) = & Token.Kind
        {
            if matches!
            (
                K.as_str(),
                "package" | "import" | "class" | "interface" | "fun" | "val" | "var" | "object"
                | "companion" | "constructor" | "init" | "enum" | "data" | "sealed" | "open"
                | "override" | "private" | "protected" | "public" | "internal" | "inline"
                | "suspend" | "infix" | "operator" | "tailrec" | "external" | "annotation"
                | "crossinline" | "noinline" | "reified" | "actual" | "expect" | "if" | "else"
                | "when" | "while" | "do" | "for" | "in" | "!in" | "is" | "!is" | "as" | "as?"
                | "try" | "catch" | "finally" | "throw" | "return" | "by" | "where" | "typealias"
            )
            { self.EmitSpace(); }
        }
        else if let Token_Kind::Operator(Op) = & Token.Kind
        {
            if matches!(Op.as_str(),"=" | "==" | "===" | "!=" | "!==" | "<=" | ">=" | "&&" | "||" | "+=" | "-=" | "*=" | "/=" | "%=" | "+" | "-" | "*" | "/" | "%" | "->" | "?:")
            { self.EmitSpace(); }
        }
    }

    fn EmitSpaceBeforeToken( & mut self, Token: & Token_Info)
    {
        if Token.Text == ";"
        || Token.Text == ","
        || Token.Text == "."
        || Token.Text == "?."
        || Token.Text == "::"
        || Token.Text == "!!"
        || Token.Text == ".."
        || Token.Text == "..<"
        || Token.Text == "?" { return ; }
        if self.Output.ends_with('.')
        || self.Output.ends_with("?.")
        || self.Output.ends_with("::")
        || self.Output.ends_with("..")
        || self.Output.ends_with("..<")
        || self.Output.ends_with('@')
        || self.Output.ends_with('?') { return ; }
        if self.Output.ends_with('(') || self.Output.ends_with('[') { return ; }
        if Token.Text == ":" { if ! self.IsClassOrInterfaceInheritanceColon() { return ; } }
        self.EmitSpace();
    }

    fn IsClassOrInterfaceInheritanceColon( & self) -> bool
    {
        let CurrentLine = if let Some(Idx) = self.Output.rfind('\n') { & self.Output[Idx + 1..] }
        else { & self.Output };
        let Trimmed = CurrentLine.trim();
        (
            Trimmed.starts_with("class ")
            || Trimmed.starts_with("interface ")
            || Trimmed.starts_with("object ")
            || Trimmed.starts_with("enum class ")
            || Trimmed.starts_with("data class ")
            || Trimmed.starts_with("sealed class ")
            || Trimmed.starts_with("open class ")
            || Trimmed.starts_with("abstract class ")
        )
        && ! Trimmed.starts_with("fun ")
    }

    fn EmitContainerNodeWithLayout( & mut self, Container: & Kotlin_Container, Level: usize, Layout: Layout_State,)
    {
        match Layout
        {
            Layout_State::Monolith => { self.EmitMonolith(Container, Level); }
            Layout_State::Grouped => { self.EmitGrouped(Container, Level); }
            Layout_State::Expanded => { self.EmitExpanded(Container, Level); }
        }
    }

    fn EmitMonolith( & mut self, Container: & Kotlin_Container, Level: usize)
    {
        if ! self.Line_Has_Content { self.EmitIndent(Level); }
        else if Container.Kind == Container_Kind::Brace { self.EmitSpace(); }
        else if Container.Kind == Container_Kind::Paren
        {
            if ! self.Output.ends_with(' ')
            {
                let LastChar = self.Output.chars().last();
                if let Some(Ch) = LastChar { if ! Ch.is_alphanumeric() && Ch != '_' && Ch != '$' && Ch != '`' && Ch != ')' && Ch != ']' && Ch != '.' { self.EmitSpace(); } }
            }
        }

        self.EmitRaw(Container.Kind.OpenGlyph());

        let mut PrevNeedsSpace = matches!(Container.Kind, Container_Kind::Brace);
        for Child in & Container.Children
        {
            match Child
            {
                Kotlin_Syntax_Node::Token(T) =>
                {
                    if T.IsTrivia() { continue ; }
                    let IsPunct = T.Text == ";"
                    || T.Text == ","
                    || T.Text == "."
                    || T.Text == ":"
                    || T.Text == "?."
                    || T.Text == "::"
                    || T.Text == "!!"
                    || T.Text == ".."
                    || T.Text == "..<"
                    || T.Text == "?";

                    if PrevNeedsSpace
                    && ! IsPunct
                    && ! self.Output.ends_with('.')
                    && ! self.Output.ends_with("?.")
                    && ! self.Output.ends_with("::")
                    && ! self.Output.ends_with('(')
                    && ! self.Output.ends_with('[')
                    && ! self.Output.ends_with('@')
                    && ! self.Output.ends_with('?') { self.EmitSpace(); }
                    self.EmitRaw( & T.Text);
                    PrevNeedsSpace = ! matches!(T.Text.as_str(),"." | "?." | "::" | "(" | "[" | "!" | "~" | "@" | ".." | "..<" | "?");
                }
                Kotlin_Syntax_Node::Container(Sub) =>
                {
                    if PrevNeedsSpace && Sub.Kind == Container_Kind::Brace { self.EmitSpace(); }
                    self.EmitMonolith(Sub, Level);
                    PrevNeedsSpace = true;
                }
            }
        }

        if matches!(Container.Kind, Container_Kind::Brace)
        && ! self.Output.ends_with(' ')
        && ! self.Output.ends_with('{') { self.EmitSpace(); }

        self.EmitRaw(Container.Kind.CloseGlyph());
        for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
    }

    fn EmitGrouped( & mut self, Container: & Kotlin_Container, Level: usize)
    {
        let Chain = Container.CollectChain();
        let OpenRun: String = Chain.iter().map( | C | C.Kind.OpenGlyph()).collect();
        let CloseRun: String = Chain.iter().rev().map( | C | C.Kind.CloseGlyph()).collect();

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

    fn EmitExpanded( & mut self, Container: & Kotlin_Container, Level: usize)
    {
        let Hug = Container.ShouldHug(self.Style);

        if Hug
        {
            // Column-Anchored Hybrid / Hugged Opener (Version 2 / Brain KanceR)
            if Container.Kind == Container_Kind::Brace { self.EmitSpace(); }
            self.EmitRaw(Container.Kind.OpenGlyph());
            self.WriteNewline();

            self.EmitNodeList( & Container.Children, Level + 1);

            self.EnsureNewline();
            self.EmitIndent(Level);
            self.EmitRaw(Container.Kind.CloseGlyph());
            for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
            return ;
        }

        // Version 1 (Pure Allman for all containers) OR standard Allman containers in Version 2:
        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();

        self.EmitNodeList( & Container.Children, Level + 1);

        self.EnsureNewline();
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.CloseGlyph());
        for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
        if Container.Kind == Container_Kind::Brace { self.WriteNewline(); }
    }

    fn IsImportStatementEnding(Nodes: & [Kotlin_Syntax_Node], CurrentIdx: usize) -> bool
    {
        if CurrentIdx >= Nodes.len() { return false; }

        let mut StartIdx = CurrentIdx;
        while StartIdx > 0
        {
            let PrevIdx = StartIdx - 1;
            match & Nodes[PrevIdx]
            {
                Kotlin_Syntax_Node::Token(T) => { if T.Preceding_Newlines > 0 { break ; } StartIdx = PrevIdx; }
                _ => break,
            }
        }

        match & Nodes[StartIdx]
        {
            Kotlin_Syntax_Node::Token(T) => { if let Token_Kind::Keyword(ref K) = T.Kind { return matches ! (K.as_str(),"import" | "package"); } false }
            _ => false,
        }
    }

    fn HasMoreNonImportStatements(Nodes: & [Kotlin_Syntax_Node], NextIdx: usize) -> bool
    {
        let mut I = NextIdx;
        while I < Nodes.len()
        {
            match & Nodes[I]
            {
                Kotlin_Syntax_Node::Token(T) =>
                {
                    if let Token_Kind::Keyword(ref K) = T.Kind { if ! matches ! (K.as_str(),"import" | "package") { return true; } }
                    else if ! T.IsTrivia() && ! T.IsComment() { return true; }
                }
                Kotlin_Syntax_Node::Container(_) => { return true; }
            }
            I += 1;
        }
        false
    }
}

pub fn VerifyKotlinTokenEquivalence(OriginalSource: & str, FormattedSource: & str) -> Result<(), String>
{
    let mut OriginalScanner = Kotlin_Lexer::New(OriginalSource);
    let OriginalTokens = OriginalScanner.TokenizeAll();

    let mut FormattedScanner = Kotlin_Lexer::New(FormattedSource);
    let FormattedTokens = FormattedScanner.TokenizeAll();

    let OrigSignificant: Vec <& str > = OriginalTokens
    .iter()
    .filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
    .map( | T | T.Text.as_str())
    .collect();

    let FormSignificant: Vec <& str > = FormattedTokens
    .iter()
    .filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
    .map( | T | T.Text.as_str())
    .collect();

    if OrigSignificant.len() != FormSignificant.len()
    {
        return Err(format ! ("Safety Gate Failed: Significant token count mismatch! (Original: {}, Formatted: {})", OrigSignificant.len(), FormSignificant.len()));
    }

    for (Index, (Orig, Form)) in OrigSignificant.iter().zip(FormSignificant.iter()).enumerate() { if Orig != Form { return Err(format ! ("Safety Gate Failed: Token #{} mismatch! (Expected '{}', Got '{}')", Index, Orig, Form)); } }

    Ok(())
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::types::Config_Settings;

pub struct Batch_Formatter;

impl Batch_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let Parser = Batch_Parser::New(Source);
        let Items = Parser.Parse();

        let Emitter = Batch_Emitter::New(Config);
        let Formatted = Emitter.Emit( & Items);

        if let Err(VerifyErr) = VerifyBatchTokenEquivalence(Source, & Formatted)
        {
            return Err(format ! ("{}. Output aborted to prevent code corruption.", VerifyErr));
        }

        Ok(Formatted)
    }

    pub fn IsFormatted(Source: & str, Config: & Config_Settings) -> bool
    { match Self::Format(Source, Config) { Ok(Formatted) => Formatted == Source, Err(_) => false, } }
}

#[derive(Debug, Clone)]
pub enum Batch_Item
{ Blank, Comment(String), Label(String), Command(Batch_Command), Block(Batch_Block), }

#[derive(Debug, Clone)]
pub struct Batch_Command
{ pub Text: String, pub Continuations: Vec<String>, }

#[derive(Debug, Clone)]
pub struct Batch_Block
{ pub Header: String, pub Body: Vec<Batch_Item>, pub Else_Branch: Option<Vec<Batch_Item>>, }

struct Batch_Parser<'a>
{ Lines: Vec< & 'a str>, Index: usize, }

impl <'a> Batch_Parser<'a>
{
    pub fn New(Source: & 'a str) -> Self
    {
        let Lines: Vec < & 'a str > = Source.lines().map( | L | L.trim_end_matches('\r')).collect();
        Batch_Parser { Lines, Index: 0 }
    }

    pub fn Parse(mut self) -> Vec<Batch_Item>
    {
        let mut Items = Vec:: new ();
        while self.Index < self.Lines.len()
        { if let Some(Item) = self.ParseItem() { Items.push(Item); } }
        Items
    }

    fn ParseItem( & mut self) -> Option<Batch_Item>
    {
        if self.Index >= self.Lines.len()
        { return None; }

        let Line = self.Lines[self.Index];
        let Trimmed = Line.trim();

        if Trimmed.is_empty()
        { self.Index += 1; return Some(Batch_Item::Blank); }

        // Comment check
        let Lower = Trimmed.to_ascii_lowercase();
        if Lower.starts_with("rem ") || Lower.starts_with("rem\t") || Lower == "rem" || Trimmed.starts_with("::")
        {
            self.Index += 1;
            return Some(Batch_Item::Comment(Trimmed.to_string()));
        }

        // Label check
        if Trimmed.starts_with(':') && ! Trimmed.starts_with("::")
        { self.Index += 1; return Some(Batch_Item::Label(Trimmed.to_string())); }

        // Check if line opens a multi-line block
        let (Balance, EndsWithOpen) = CheckParenBalance(Trimmed);
        if EndsWithOpen && Balance > 0
        { return self.ParseBlock(); }

        // Normal command or monolith
        self.Index += 1;
        let CmdText = Trimmed.to_string();
        let mut Continuations = Vec:: new ();

        // Check for caret line continuations
        while CmdText.ends_with('^') && ! CmdText.ends_with("^^") && self.Index < self.Lines.len()
        {
            let NextLine = self.Lines[self.Index].trim();
            self.Index += 1;
            Continuations.push(NextLine.to_string());
            if ! NextLine.ends_with('^') || NextLine.ends_with("^^")
            { break ; }
        }

        Some(Batch_Item::Command(Batch_Command { Text: CmdText, Continuations, }))
    }

    fn ParseBlock( & mut self) -> Option<Batch_Item>
    {
        let Line = self.Lines[self.Index].trim();
        self.Index += 1;

        // Extract header before the trailing unquoted '('
        let Header = ExtractBlockHeader(Line);

        // Parse block body
        let mut Body = Vec:: new ();
        let mut ElseBranch = None;

        while self.Index < self.Lines.len()
        {
            let CurLine = self.Lines[self.Index].trim();

            // Check for closing delimiter: ')' or ') ^' or ') else ('
            if CurLine == ")" || CurLine == ") ^"
            {
                self.Index += 1;
                // Check if followed by else (
                if self.Index < self.Lines.len()
                {
                    let NextTrimmed = self.Lines[self.Index].trim();
                    let LowerNext = NextTrimmed.to_ascii_lowercase();
                    if LowerNext.starts_with("else (") || LowerNext.starts_with("else(")
                    { self.Index += 1; ElseBranch = Some(self.ParseElseBody()); }
                }
                break;
            }

            let LowerCur = CurLine.to_ascii_lowercase();
            if LowerCur.starts_with(") else (") || LowerCur.starts_with(") else(")
            { self.Index += 1; ElseBranch = Some(self.ParseElseBody()); break ; }

            if let Some(Item) = self.ParseItem()
            { Body.push(Item); }
        }

        Some(Batch_Item::Block(Batch_Block { Header, Body, Else_Branch: ElseBranch, }))
    }

    fn ParseElseBody( & mut self) -> Vec<Batch_Item>
    {
        let mut ElseBody = Vec:: new ();
        while self.Index < self.Lines.len()
        {
            let CurLine = self.Lines[self.Index].trim();
            if CurLine == ")" || CurLine.starts_with(")^") || CurLine.starts_with(") ^")
            { self.Index += 1; break ; }

            if let Some(Item) = self.ParseItem()
            { ElseBody.push(Item); }
        }
        ElseBody
    }
}

fn CheckParenBalance(Line: & str) -> (i32, bool)
{
    let mut Balance: i32 = 0;
    let mut InDoubleQuote = false;
    let mut InSingleQuote = false;
    let mut PrevEscape = false;
    let mut LastUnquotedChar = ' ';

    for Ch in Line.chars()
    {
        if PrevEscape
        { PrevEscape = false; continue ; }

        if Ch == '^' && ! InDoubleQuote
        { PrevEscape = true; continue ; }

        if Ch == '"' && ! InSingleQuote
        { InDoubleQuote = ! InDoubleQuote; continue ; }

        if Ch == '\'' && ! InDoubleQuote
        { InSingleQuote = ! InSingleQuote; continue ; }

        if ! InDoubleQuote && ! InSingleQuote
        { if ! Ch.is_whitespace() { LastUnquotedChar = Ch; } if Ch == '(' { Balance += 1; } else if Ch == ')' { Balance -= 1; } }
    }

    (Balance, LastUnquotedChar == '(')
}

fn ExtractBlockHeader(Line: & str) -> String
{
    let mut LastParenIdx = 0;
    let mut InDoubleQuote = false;
    let mut InSingleQuote = false;
    let mut PrevEscape = false;

    for (Idx, Ch) in Line.char_indices()
    {
        if PrevEscape
        { PrevEscape = false; continue ; }
        if Ch == '^' && ! InDoubleQuote
        { PrevEscape = true; continue ; }
        if Ch == '"' && ! InSingleQuote
        { InDoubleQuote = ! InDoubleQuote; continue ; }
        if Ch == '\'' && ! InDoubleQuote
        { InSingleQuote = ! InSingleQuote; continue ; }
        if ! InDoubleQuote && ! InSingleQuote && Ch == '('
        { LastParenIdx = Idx; }
    }

    Line[..LastParenIdx].trim_end().to_string()
}

pub struct Batch_Emitter<'a>
{ Config: & 'a Config_Settings, Output: String, }

impl <'a> Batch_Emitter<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self
    { Batch_Emitter { Config, Output: String::new (), } }

    pub fn Emit(mut self, Items: & [Batch_Item]) -> String
    {
        self.EmitItemList(Items, 0);
        if ! self.Output.ends_with('\n')
        { self.Output.push('\n'); }
        self.Output
    }

    fn EmitItemList( & mut self, Items: & [Batch_Item], BaseIndent: usize)
    {
        let mut ConsecutiveBlanks = 0;

        for Item in Items
        {
            match Item
            {
                Batch_Item::Blank =>
                {
                    if ConsecutiveBlanks < self.Config.Max_Blank_Lines && ! self.Output.is_empty() && ! self.Output.ends_with("\n\n")
                    {
                        self.Output.push('\n');
                        ConsecutiveBlanks += 1;
                    }
                }
                Batch_Item::Comment(Text) =>
                {
                    ConsecutiveBlanks = 0;
                    self.EmitSpaces(BaseIndent);
                    self.Output.push_str(Text);
                    self.Output.push('\n');
                }
                Batch_Item::Label(Text) =>
                {
                    ConsecutiveBlanks = 0;
                    // Labels always sit at column 0 in Batch
                    self.Output.push_str(Text);
                    self.Output.push('\n');
                }
                Batch_Item::Command(Cmd) =>
                {
                    ConsecutiveBlanks = 0;
                    self.EmitSpaces(BaseIndent);
                    self.Output.push_str( & Cmd.Text);
                    self.Output.push('\n');

                    for Cont in & Cmd.Continuations
                    {
                        // Continuations align with extra 5 spaces
                        self.EmitSpaces(BaseIndent + 5);
                        self.Output.push_str(Cont);
                        self.Output.push('\n');
                    }
                }
                Batch_Item::Block(Block) =>
                { ConsecutiveBlanks = 0; self.EmitBlock(Block, BaseIndent); }
            }
        }
    }

    fn EmitBlock( & mut self, Block: & Batch_Block, BaseIndent: usize)
    {
        // 1. Check if the block qualifies as a State 1 Monolith
        if let Some(MonolithStr) = self.TryBuildMonolith(Block, BaseIndent)
        {
            self.EmitSpaces(BaseIndent);
            self.Output.push_str( & MonolithStr);
            self.Output.push('\n');
            return ;
        }

        // 2. State 2: Delimiter-Anchored Column Symmetry
        let HeaderText = & Block.Header;
        let HeaderCol = BaseIndent;

        // Opening delimiter column X
        let ColParen = HeaderCol + HeaderText.len() + 1;

        self.EmitSpaces(HeaderCol);
        self.Output.push_str(HeaderText);
        self.Output.push_str(" (\n");

        // Payload inside is indented at ColParen + Indent_Width
        let PayloadIndent = ColParen + self.Config.Indent_Width;
        self.EmitItemList( & Block.Body, PayloadIndent);

        if let Some(ref ElseBranch) = Block.Else_Branch
        {
            // Close if-block with caret bridge ') ^'
            self.EmitSpaces(ColParen);
            self.Output.push_str(") ^\n");

            // Start 'else (' at base indent
            let ElseColParen = BaseIndent + 5;
            // "else " is 5 chars
            self.EmitSpaces(BaseIndent);
            self.Output.push_str("else (\n");

            let ElsePayloadIndent = ElseColParen + self.Config.Indent_Width;
            self.EmitItemList(ElseBranch, ElsePayloadIndent);

            self.EmitSpaces(ElseColParen);
            self.Output.push_str(")\n");
        }
        else
        { self.EmitSpaces(ColParen); self.Output.push_str(")\n"); }
    }

    fn TryBuildMonolith( & self, Block: & Batch_Block, BaseIndent: usize) -> Option<String>
    {
        // 1. Loop blocks (for ...) must NEVER collapse into a monolith if parsed as a multi-line block
        let LowerHeader = Block.Header.trim_start().to_ascii_lowercase();
        if LowerHeader.starts_with("for ") || LowerHeader.starts_with("for\t")
        { return None; }

        // 2. Only blocks with exactly 1 command body can be considered for monolith
        if Block.Body.len() != 1
        { return None; }

        let BodyCmd = match & Block.Body[0]
        { Batch_Item::Command(Cmd) if Cmd.Continuations.is_empty() => & Cmd.Text, _ => return None, };

        // If the body command contains parentheses, it is already a compound block/subexpression
        // (such as an inner for loop or inner if statement). Exceeds complexity budget.
        if BodyCmd.contains('(') || BodyCmd.contains(')')
        { return None; }

        if let Some(ref ElseBranch) = Block.Else_Branch
        {
            if ElseBranch.len() != 1
            { return None; }
            let ElseCmd = match & ElseBranch[0]
            { Batch_Item::Command(Cmd) if Cmd.Continuations.is_empty() => & Cmd.Text, _ => return None, };

            // If else branch contains parentheses, it cannot be inside a monolith
            if ElseCmd.contains('(') || ElseCmd.contains(')')
            { return None; }

            let Monolith = format!("{} ( {} ) else ( {} )", Block.Header, BodyCmd, ElseCmd);
            if BaseIndent + Monolith.len() <= self.Config.Canvas_Width
            { return Some(Monolith); }
            return None;
        }

        // Monolith for simple if without else
        let Monolith = if BodyCmd.starts_with("set ") || BodyCmd.starts_with("goto ") || BodyCmd.starts_with("exit ") || BodyCmd.starts_with("echo ") || BodyCmd.starts_with("call ")
        { format ! ("{} {}", Block.Header, BodyCmd) }
        else
        { format ! ("{} ( {} )", Block.Header, BodyCmd) };

        if BaseIndent + Monolith.len() <= self.Config.Canvas_Width
        { Some(Monolith) }
        else
        { None }
    }

    fn EmitSpaces( & mut self, Count: usize)
    { for _ in 0..Count { self.Output.push(' '); } }
}

pub fn VerifyBatchTokenEquivalence(OriginalSource: & str, FormattedSource: & str) -> Result<(), String>
{
    let OrigTokens = ExtractBatchSignificantTokens(OriginalSource);
    let FormTokens = ExtractBatchSignificantTokens(FormattedSource);

    if OrigTokens.len() != FormTokens.len()
    {
        return Err(format ! ("Safety Gate Failed: Significant token count mismatch! (Original: {}, Formatted: {})", OrigTokens.len(), FormTokens.len()));
    }

    for (Index, (Orig, Form)) in OrigTokens.iter().zip(FormTokens.iter()).enumerate()
    { if Orig != Form { return Err(format ! ("Safety Gate Failed: Token #{} mismatch! (Expected '{}', Got '{}')", Index, Orig, Form)); } }

    Ok(())
}

fn ExtractBatchSignificantTokens(Source: & str) -> Vec<String>
{
    let mut Tokens = Vec:: new ();
    let Chars: Vec < char > = Source.chars().collect();
    let Len = Chars.len();
    let mut I = 0;
    let mut AtLineStart = true;

    while I < Len
    {
        let Ch = Chars[I];

        // Newlines reset AtLineStart
        if Ch == '\n' || Ch == '\r'
        { AtLineStart = true; I += 1; continue ; }

        // Whitespace (spaces and tabs)
        if Ch.is_whitespace()
        { I += 1; continue ; }

        // Line continuation '^' immediately followed by newline is trivia
        if Ch == '^'
        {
            let mut J = I + 1;
            while J < Len && (Chars[J] == ' ' || Chars[J] == '\t' || Chars[J] == '\r')
            { J += 1; }
            if J < Len && Chars[J] == '\n'
            {
                I = J + 1;
                // Next line continues the statement
                AtLineStart = false;
                continue;
            }
        }

        // Check for comment at line start
        if AtLineStart
        {
            // Check for "rem" followed by space/tab/eol, or "::"
            let Remaining: String = Chars[I..std::cmp::min(I + 4, Len)].iter().collect();
            let Lower4 = Remaining.to_ascii_lowercase();
            let IsRem = Lower4.starts_with("rem ") || Lower4.starts_with("rem\t") || Lower4.starts_with("rem\r") || Lower4.starts_with("rem\n") || Lower4 == "rem";
            let IsColonColon = Remaining.starts_with("::");

            if IsRem || IsColonColon
            {
                let mut CommentStr = String:: new ();
                while I < Len && Chars[I] != '\n' && Chars[I] != '\r'
                { CommentStr.push(Chars[I]); I += 1; }
                Tokens.push(CommentStr.trim().to_string());
                AtLineStart = true;
                continue;
            }
        }

        AtLineStart = false;

        // Double-quoted strings in Batch (do not cross newlines unless escaped)
        if Ch == '"'
        {
            let mut S = String:: new ();
            S.push('"');
            I += 1;
            while I < Len && Chars[I] != '\n' && Chars[I] != '\r'
            { let C = Chars[I]; S.push(C); I += 1; if C == '"' { break ; } }
            Tokens.push(S);
            continue;
        }

        // Single-quoted strings in Batch (e.g. in 'command' inside for /f; never cross newlines)
        if Ch == '\''
        {
            let mut S = String:: new ();
            S.push('\'');
            I += 1;
            while I < Len && Chars[I] != '\n' && Chars[I] != '\r'
            { let C = Chars[I]; S.push(C); I += 1; if C == '\'' { break ; } }
            Tokens.push(S);
            continue;
        }

        // Delimiters
        if Ch == '(' || Ch == ')'
        { Tokens.push(Ch.to_string()); I += 1; continue ; }

        // Non-whitespace token chunk
        let mut TokenStr = String:: new ();
        while I < Len
        {
            let C = Chars[I];
            if C.is_whitespace() || C == '\r' || C == '\n' || C == '(' || C == ')' || C == '"' || C == '\''
            { break ; }
            // Check if '^' is line continuation
            if C == '^'
            {
                let mut J = I + 1;
                while J < Len && (Chars[J] == ' ' || Chars[J] == '\t' || Chars[J] == '\r')
                { J += 1; }
                if J < Len && Chars[J] == '\n'
                { break ; }
            }
            TokenStr.push(C);
            I += 1;
        }

        if ! TokenStr.is_empty()
        { Tokens.push(TokenStr); }
    }

    Tokens
}

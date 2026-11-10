#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::types:: { Config_Settings, Source_Language };

pub struct CSS_Formatter;

impl CSS_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let Formatted = Self::FormatInternal(Source, Config, 0)?;
        Ok(Formatted)
    }

    pub fn FormatWithBaseLevel(Source: & str, Config: & Config_Settings, BaseLevel: usize) -> Result<String, String>
    { Self::FormatInternal(Source, Config, BaseLevel) }

    fn FormatInternal(Source: & str, Config: & Config_Settings, BaseLevel: usize) -> Result<String, String>
    {
        let Tokens = Self::Tokenize(Source);
        let Nodes = Self::Parse( & Tokens);
        let Output = Self::Emit( & Nodes, Config, BaseLevel);
        Ok(Output)
    }

    fn Tokenize(Source: & str) -> Vec<CSS_Token>
    {
        let Chars: Vec < char > = Source.chars().collect();
        let Len = Chars.len();
        let mut I = 0;
        let mut Tokens = Vec:: new ();

        while I < Len
        {
            let Ch = Chars[I];

            // 1. Whitespace
            if Ch == ' ' || Ch == '\t' || Ch == '\r' || Ch == '\n'
            {
                let mut HasNewline = false;
                while I < Len && (Chars[I] == ' ' || Chars[I] == '\t' || Chars[I] == '\r' || Chars[I] == '\n')
                {
                    if Chars[I] == '\n' { HasNewline = true; }
                    I += 1;
                }
                Tokens.push(CSS_Token::Whitespace(HasNewline));
                continue;
            }

            // 2. Block Comments /* ... */
            if Ch == '/' && I + 1 < Len && Chars[I + 1] == '*'
            {
                let Start = I;
                I += 2;
                while I + 1 < Len && !(Chars[I] == '*' && Chars[I + 1] == '/')
                { I += 1; }
                if I + 1 < Len
                {
                    I += 2;
                    // skip */
                }
                else
                { I = Len; }
                let Text: String = Chars[Start..I].iter().collect();
                Tokens.push(CSS_Token::Comment(Text));
                continue;
            }

            // 3. Strings
            if Ch == '"' || Ch == '\''
            {
                let Quote = Ch;
                let Start = I;
                I += 1;
                while I < Len
                {
                    if Chars[I] == '\\'
                    { I += 1; if I < Len { I += 1; } continue ; }
                    if Chars[I] == Quote
                    { I += 1; break ; }
                    I += 1;
                }
                let Text: String = Chars[Start..I].iter().collect();
                Tokens.push(CSS_Token::StringLit(Text));
                continue;
            }

            // 4. Delimiters & Punctuation
            if Ch == '{'
            { Tokens.push(CSS_Token::OpenBrace); I += 1; continue ; }
            if Ch == '}'
            { Tokens.push(CSS_Token::CloseBrace); I += 1; continue ; }
            if Ch == '('
            { Tokens.push(CSS_Token::OpenParen); I += 1; continue ; }
            if Ch == ')'
            { Tokens.push(CSS_Token::CloseParen); I += 1; continue ; }
            if Ch == ';'
            { Tokens.push(CSS_Token::Semicolon); I += 1; continue ; }
            if Ch == ':'
            { Tokens.push(CSS_Token::Colon); I += 1; continue ; }
            if Ch == ','
            { Tokens.push(CSS_Token::Comma); I += 1; continue ; }

            // 5. General Word / Ident / Value
            let Start = I;
            while I < Len
            {
                let C = Chars[I];
                if C == ' ' || C == '\t' || C == '\r' || C == '\n'
                || C == '{' || C == '}' || C == '(' || C == ')' || C == ';' || C == ':' || C == ','
                || (C == '/' && I + 1 < Len && Chars[I + 1] == '*')
                || C == '"' || C == '\''
                { break ; }
                I += 1;
            }
            let mut Word: String = Chars[Start..I].iter().collect();

            // Uppercase hex color codes: #7100ff -> #7100FF
            if Word.starts_with('#') && (Word.len() == 4 || Word.len() == 5 || Word.len() == 7 || Word.len() == 9)
            { if Word[1..].chars().all( | c | c.is_ascii_hexdigit()) { Word = format ! ("#{}", Word[1..].to_uppercase()); } }

            Tokens.push(CSS_Token::Word(Word));
        }

        Tokens
    }

    fn Parse(Tokens: & [CSS_Token]) -> Vec<CSS_Node>
    {
        let mut Nodes = Vec:: new ();
        let mut I = 0;
        let Len = Tokens.len();

        while I < Len
        {
            match & Tokens[I]
            {
                CSS_Token::Whitespace(_) => { I += 1; }
                CSS_Token::Comment(c) =>
                { Nodes.push(CSS_Node::Comment(c.clone())); I += 1; }
                _ =>
                { if let Some((Node, NewI)) = Self::ParseRuleOrAtRule(Tokens, I) { Nodes.push(Node); I = NewI; } else { I += 1; } }
            }
        }

        Nodes
    }

    fn ParseRuleOrAtRule(Tokens: & [CSS_Token], StartIdx: usize) -> Option < (CSS_Node, usize) >
    {
        let mut HeaderWords = Vec:: new ();
        let mut I = StartIdx;
        let Len = Tokens.len();

        while I < Len
        {
            match & Tokens[I]
            {
                CSS_Token::OpenBrace =>
                {
                    // Block found! Parse declarations or nested rules inside
                    I += 1;
                    // skip {
                    let mut BlockNodes = Vec:: new ();
                    let mut CurrentDeclWords = Vec:: new ();
                    let mut DeclStartTokenIdx = I;

                    while I < Len
                    {
                        match & Tokens[I]
                        {
                            CSS_Token::CloseBrace =>
                            {
                                if ! CurrentDeclWords.is_empty()
                                {
                                    BlockNodes.push(CSS_Node::Declaration(CurrentDeclWords.join(" ")));
                                    CurrentDeclWords.clear();
                                }
                                I += 1;
                                // skip }
                                break;
                            }
                            CSS_Token::Semicolon =>
                            {
                                CurrentDeclWords.push(";".to_string());
                                BlockNodes.push(CSS_Node::Declaration(CurrentDeclWords.join(" ")));
                                CurrentDeclWords.clear();
                                I += 1;
                                DeclStartTokenIdx = I;
                            }
                            CSS_Token::Comment(c) =>
                            {
                                if ! CurrentDeclWords.is_empty()
                                {
                                    BlockNodes.push(CSS_Node::Declaration(CurrentDeclWords.join(" ")));
                                    CurrentDeclWords.clear();
                                }
                                BlockNodes.push(CSS_Node::Comment(c.clone()));
                                I += 1;
                                DeclStartTokenIdx = I;
                            }
                            CSS_Token::OpenBrace =>
                            {
                                // Nested rule!
                                if let Some((NestedNode, NewI)) = Self::ParseRuleOrAtRule(Tokens, DeclStartTokenIdx)
                                {
                                    CurrentDeclWords.clear();
                                    BlockNodes.push(NestedNode);
                                    I = NewI;
                                    DeclStartTokenIdx = I;
                                }
                                else
                                { I += 1; }
                            }
                            CSS_Token::OpenParen =>
                            {
                                if CurrentDeclWords.is_empty() { DeclStartTokenIdx = I; }
                                CurrentDeclWords.push("(".to_string());
                                I += 1;
                            }
                            CSS_Token::CloseParen =>
                            {
                                if CurrentDeclWords.is_empty() { DeclStartTokenIdx = I; }
                                CurrentDeclWords.push(")".to_string());
                                I += 1;
                            }
                            CSS_Token::Colon =>
                            {
                                if CurrentDeclWords.is_empty() { DeclStartTokenIdx = I; }
                                CurrentDeclWords.push(":".to_string());
                                I += 1;
                            }
                            CSS_Token::Comma =>
                            {
                                if CurrentDeclWords.is_empty() { DeclStartTokenIdx = I; }
                                CurrentDeclWords.push(",".to_string());
                                I += 1;
                            }
                            CSS_Token::Word(w) | CSS_Token::StringLit(w) =>
                            {
                                if CurrentDeclWords.is_empty() { DeclStartTokenIdx = I; }
                                CurrentDeclWords.push(w.clone());
                                I += 1;
                            }
                            CSS_Token::Whitespace(_) => { I += 1; }
                        }
                    }

                    let Header = Self::FormatHeader( & HeaderWords);
                    return Some((CSS_Node::RuleSet { Header, Children: BlockNodes }, I));
                }
                CSS_Token::Semicolon =>
                {
                    HeaderWords.push(";".to_string());
                    I += 1;
                    return Some((CSS_Node::Declaration(HeaderWords.join(" ")), I));
                }
                CSS_Token::Comment(c) =>
                { HeaderWords.push(c.clone()); I += 1; }
                CSS_Token::OpenParen =>
                { HeaderWords.push("(".to_string()); I += 1; }
                CSS_Token::CloseParen =>
                { HeaderWords.push(")".to_string()); I += 1; }
                CSS_Token::Colon =>
                { HeaderWords.push(":".to_string()); I += 1; }
                CSS_Token::Comma =>
                { HeaderWords.push(",".to_string()); I += 1; }
                CSS_Token::Word(w) | CSS_Token::StringLit(w) =>
                { HeaderWords.push(w.clone()); I += 1; }
                CSS_Token::Whitespace(_) => { I += 1; }
                CSS_Token::CloseBrace =>
                { break ; }
            }
        }

        if ! HeaderWords.is_empty()
        { Some((CSS_Node::Declaration(HeaderWords.join(" ")), I)) }
        else
        { None }
    }

    fn Emit(Nodes: & [CSS_Node], Config: & Config_Settings, BaseLevel: usize) -> String
    {
        let mut Out = String:: new ();
        for (Idx, Node) in Nodes.iter().enumerate()
        {
            if Idx > 0 && ! Out.ends_with("\n\n")
            { if ! Out.ends_with('\n') { Out.push('\n'); } }
            Self::EmitNode( & mut Out, Node, Config, BaseLevel);
        }
        if ! Out.is_empty() && ! Out.ends_with('\n')
        { Out.push('\n'); }
        Out
    }

    fn EmitNode(Out: & mut String, Node: & CSS_Node, Config: & Config_Settings, Level: usize)
    {
        let Indent = " ".repeat(Level * Config.Indent_Width);

        match Node
        {
            CSS_Node::Comment(c) =>
            { Out.push_str( & Indent); Out.push_str(c); Out.push('\n'); }
            CSS_Node::Declaration(d) =>
            { Out.push_str( & Indent); Out.push_str( & Self::FormatDeclaration(d)); Out.push('\n'); }
            CSS_Node::RuleSet { Header, Children }
            =>
            {
                let HasNestedRules = Children.iter().any( | c | matches ! (c, CSS_Node::RuleSet {.. }));
                let IsSingleDecl = Level == 0 && ! HasNestedRules && Children.len() == 1 && matches!(Children[0], CSS_Node::Declaration(_));
                if IsSingleDecl
                {
                    if let CSS_Node::Declaration(d) = & Children[0]
                    {
                        let FormattedD = Self::FormatDeclaration(d);
                        let MonolithLine = format!("{}{} {{ {} }}", Indent, Header, FormattedD);
                        if MonolithLine.len() <= Config.Canvas_Width
                        { Out.push_str( & MonolithLine); Out.push('\n'); return ; }
                    }
                }

                // Expanded Symmetry
                Out.push_str( & Indent);
                Out.push_str(Header);
                Out.push('\n');
                Out.push_str( & Indent);
                Out.push_str("{\n");

                for Child in Children
                { Self::EmitNode(Out, Child, Config, Level + 1); }

                Out.push_str( & Indent);
                Out.push_str("}\n");
            }
        }
    }

    fn FormatDeclaration(Decl: & str) -> String
    {
        let mut Result = String:: new ();
        let Parts: Vec < & str > = Decl.split_whitespace().collect();
        let mut I = 0;
        while I < Parts.len()
        {
            let Part = Parts[I];
            if Part == ":"
            { Result.push_str(": "); }
            else if Part.ends_with(':')
            { Result.push_str(Part); Result.push(' '); }
            else if Part == ";"
            { while Result.ends_with(' ') { Result.pop(); } Result.push(';'); }
            else if Part == ","
            { while Result.ends_with(' ') { Result.pop(); } Result.push_str(", "); }
            else
            {
                if ! Result.is_empty() && ! Result.ends_with(' ') && ! Result.ends_with('(')
                { Result.push(' '); }
                Result.push_str(Part);
            }
            I += 1;
        }

        if ! Result.ends_with(';') && ! Result.is_empty()
        { Result.push(';'); }
        Result
    }

    fn FormatHeader(Words: & [String]) -> String
    {
        let mut Res = String:: new ();
        let mut ParenDepth = 0;
        for (Idx, W) in Words.iter().enumerate()
        {
            if W == ":"
            { while Res.ends_with(' ') { Res.pop(); } if ParenDepth > 0 { Res.push_str(": "); } else { Res.push(':'); } }
            else if W == ","
            { while Res.ends_with(' ') { Res.pop(); } Res.push_str(", "); }
            else if W == "("
            {
                if ! Res.is_empty() && ! Res.ends_with(' ') && ! Res.ends_with(':')
                {
                    let PrevIsPseudo = Words. get(Idx.saturating_sub(1)).map_or(false, | p | { p.starts_with(':') || (Idx >= 2 && Words[Idx - 2] == ":") });
                    if ! PrevIsPseudo
                    { Res.push(' '); }
                }
                Res.push('(');
                ParenDepth += 1;
            }
            else if W == ")"
            { Res.push(')'); if ParenDepth > 0 { ParenDepth -= 1; } }
            else
            {
                if ! Res.is_empty()
                && ! Res.ends_with(' ')
                && ! Res.ends_with('(')
                && (ParenDepth > 0 || ! Res.ends_with(':'))
                { Res.push(' '); }
                Res.push_str(W);
            }
        }
        Res
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CSS_Token
{ Word(String), StringLit(String), Comment(String), OpenBrace, CloseBrace, OpenParen, CloseParen, Colon, Semicolon, Comma, Whitespace(bool), }

#[derive(Debug, Clone)]
enum CSS_Node
{ RuleSet { Header: String, Children: Vec<CSS_Node> }, Declaration(String), Comment(String), }

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::types::{Config_Settings, Source_Language};

pub struct CSS_Formatter;

#[derive(Debug, Clone, PartialEq, Eq)]
enum CSS_Token_Kind
{
    Word,
    StringLit,
    Comment,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Semicolon,
    Colon,
    Comma,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CSS_Token
{
    Kind: CSS_Token_Kind,
    Text: String,
    Preceding_Newlines: usize,
}

#[derive(Debug, Clone)]
enum CSS_Node
{
    RuleSet
    {
        Header_Lines: Vec<String>,
        Children: Vec<CSS_Node>,
        Preceding_Newlines: usize,
        Was_Multiline_Selector: bool,
    },
    Declaration
    {
        Text: String,
        Preceding_Newlines: usize,
    },
    Comment
    {
        Text: String,
        Preceding_Newlines: usize,
    },
}

impl CSS_Node
{
    fn Preceding_Newlines(&self) -> usize
    {
        match self
        {
            CSS_Node::RuleSet { Preceding_Newlines, .. } => *Preceding_Newlines,
            CSS_Node::Declaration { Preceding_Newlines, .. } => *Preceding_Newlines,
            CSS_Node::Comment { Preceding_Newlines, .. } => *Preceding_Newlines,
        }
    }
}

impl CSS_Formatter
{
    pub fn Format(Source: &str, Config: &Config_Settings) -> Result<String, String>
    {
        let Formatted = Self::FormatInternal(Source, Config, 0)?;
        Ok(Formatted)
    }

    pub fn FormatWithBaseLevel(Source: &str, Config: &Config_Settings, BaseLevel: usize) -> Result<String, String>
    {
        Self::FormatInternal(Source, Config, BaseLevel)
    }

    fn FormatInternal(Source: &str, Config: &Config_Settings, BaseLevel: usize) -> Result<String, String>
    {
        let Tokens = Self::Tokenize(Source);
        let Nodes = Self::Parse(&Tokens);
        let Output = Self::Emit(&Nodes, Config, BaseLevel);
        Ok(Output)
    }

    fn Tokenize(Source: &str) -> Vec<CSS_Token>
    {
        let Chars: Vec<char> = Source.chars().collect();
        let Len = Chars.len();
        let mut I = 0;
        let mut Tokens = Vec::new();
        let mut Preceding_Newlines: usize = 0;

        while I < Len
        {
            let Ch = Chars[I];

            // 1. Whitespace
            if Ch == ' ' || Ch == '\t' || Ch == '\r' || Ch == '\n'
            {
                while I < Len && (Chars[I] == ' ' || Chars[I] == '\t' || Chars[I] == '\r' || Chars[I] == '\n')
                {
                    if Chars[I] == '\n' { Preceding_Newlines += 1; }
                    I += 1;
                }
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
                { I += 2; }
                else
                { I = Len; }
                let Text: String = Chars[Start..I].iter().collect();
                Tokens.push(CSS_Token { Kind: CSS_Token_Kind::Comment, Text, Preceding_Newlines });
                Preceding_Newlines = 0;
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
                    { I += 1; if I < Len { I += 1; } continue; }
                    if Chars[I] == Quote
                    { I += 1; break; }
                    I += 1;
                }
                let Text: String = Chars[Start..I].iter().collect();
                Tokens.push(CSS_Token { Kind: CSS_Token_Kind::StringLit, Text, Preceding_Newlines });
                Preceding_Newlines = 0;
                continue;
            }

            // 4. Delimiters & Punctuation
            if Ch == '{'
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::OpenBrace, Text: "{".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }
            if Ch == '}'
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::CloseBrace, Text: "}".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }
            if Ch == '('
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::OpenParen, Text: "(".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }
            if Ch == ')'
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::CloseParen, Text: ")".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }
            if Ch == ';'
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::Semicolon, Text: ";".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }
            if Ch == ':'
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::Colon, Text: ":".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }
            if Ch == ','
            { Tokens.push(CSS_Token { Kind: CSS_Token_Kind::Comma, Text: ",".to_string(), Preceding_Newlines }); Preceding_Newlines = 0; I += 1; continue; }

            // 5. General Word / Ident / Value
            let Start = I;
            while I < Len
            {
                let C = Chars[I];
                if C == ' ' || C == '\t' || C == '\r' || C == '\n'
                || C == '{' || C == '}' || C == '(' || C == ')' || C == ';' || C == ':' || C == ','
                || (C == '/' && I + 1 < Len && Chars[I + 1] == '*')
                || C == '"' || C == '\''
                { break; }
                I += 1;
            }
            let mut Word: String = Chars[Start..I].iter().collect();

            // Uppercase hex color codes: #7100ff -> #7100FF
            if Word.starts_with('#') && (Word.len() == 4 || Word.len() == 5 || Word.len() == 7 || Word.len() == 9)
            {
                if Word[1..].chars().all(|c| c.is_ascii_hexdigit())
                { Word = format!("#{}", Word[1..].to_uppercase()); }
            }

            Tokens.push(CSS_Token { Kind: CSS_Token_Kind::Word, Text: Word, Preceding_Newlines });
            Preceding_Newlines = 0;
        }

        Tokens
    }

    fn Parse(Tokens: &[CSS_Token]) -> Vec<CSS_Node>
    {
        let mut Nodes = Vec::new();
        let mut I = 0;
        let Len = Tokens.len();

        while I < Len
        {
            match Tokens[I].Kind
            {
                CSS_Token_Kind::Comment =>
                {
                    Nodes.push(CSS_Node::Comment
                    {
                        Text: Tokens[I].Text.clone(),
                        Preceding_Newlines: Tokens[I].Preceding_Newlines,
                    });
                    I += 1;
                }
                _ =>
                {
                    if let Some((Node, NewI)) = Self::ParseRuleOrAtRule(Tokens, I)
                    {
                        Nodes.push(Node);
                        I = NewI;
                    }
                    else
                    {
                        I += 1;
                    }
                }
            }
        }

        Nodes
    }

    fn ParseRuleOrAtRule(Tokens: &[CSS_Token], StartIdx: usize) -> Option<(CSS_Node, usize)>
    {
        let mut HeaderTokens = Vec::new();
        let mut I = StartIdx;
        let Len = Tokens.len();
        let RulePrecedingNewlines = Tokens[StartIdx].Preceding_Newlines;

        while I < Len
        {
            match Tokens[I].Kind
            {
                CSS_Token_Kind::OpenBrace =>
                {
                    // Block found! Parse declarations or nested rules inside
                    I += 1; // skip {
                    let mut BlockNodes = Vec::new();
                    let mut CurrentDeclWords = Vec::new();
                    let mut DeclPrecedingNewlines = 0;
                    let mut DeclStartTokenIdx = I;

                    while I < Len
                    {
                        match Tokens[I].Kind
                        {
                            CSS_Token_Kind::CloseBrace =>
                            {
                                if !CurrentDeclWords.is_empty()
                                {
                                    BlockNodes.push(CSS_Node::Declaration
                                    {
                                        Text: CurrentDeclWords.join(" "),
                                        Preceding_Newlines: DeclPrecedingNewlines,
                                    });
                                    CurrentDeclWords.clear();
                                }
                                I += 1; // skip }
                                break;
                            }
                            CSS_Token_Kind::Semicolon =>
                            {
                                CurrentDeclWords.push(";".to_string());
                                BlockNodes.push(CSS_Node::Declaration
                                {
                                    Text: CurrentDeclWords.join(" "),
                                    Preceding_Newlines: DeclPrecedingNewlines,
                                });
                                CurrentDeclWords.clear();
                                I += 1;
                                DeclStartTokenIdx = I;
                            }
                            CSS_Token_Kind::Comment =>
                            {
                                if !CurrentDeclWords.is_empty()
                                {
                                    BlockNodes.push(CSS_Node::Declaration
                                    {
                                        Text: CurrentDeclWords.join(" "),
                                        Preceding_Newlines: DeclPrecedingNewlines,
                                    });
                                    CurrentDeclWords.clear();
                                }
                                BlockNodes.push(CSS_Node::Comment
                                {
                                    Text: Tokens[I].Text.clone(),
                                    Preceding_Newlines: Tokens[I].Preceding_Newlines,
                                });
                                I += 1;
                                DeclStartTokenIdx = I;
                            }
                            CSS_Token_Kind::OpenBrace =>
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
                                {
                                    I += 1;
                                }
                            }
                            _ =>
                            {
                                if CurrentDeclWords.is_empty()
                                {
                                    DeclStartTokenIdx = I;
                                    DeclPrecedingNewlines = Tokens[I].Preceding_Newlines;
                                }
                                CurrentDeclWords.push(Tokens[I].Text.clone());
                                I += 1;
                            }
                        }
                    }

                    let (Header_Lines, Was_Multiline_Selector) = Self::BuildHeaderLines(&HeaderTokens);

                    return Some((CSS_Node::RuleSet
                    {
                        Header_Lines,
                        Children: BlockNodes,
                        Preceding_Newlines: RulePrecedingNewlines,
                        Was_Multiline_Selector,
                    }, I));
                }
                CSS_Token_Kind::Semicolon =>
                {
                    HeaderTokens.push(Tokens[I].clone());
                    I += 1;
                    let (Header_Lines, _) = Self::BuildHeaderLines(&HeaderTokens);
                    return Some((CSS_Node::Declaration
                    {
                        Text: Header_Lines.join(" "),
                        Preceding_Newlines: RulePrecedingNewlines,
                    }, I));
                }
                _ =>
                {
                    HeaderTokens.push(Tokens[I].clone());
                    I += 1;
                }
            }
        }

        if !HeaderTokens.is_empty()
        {
            let (Header_Lines, _) = Self::BuildHeaderLines(&HeaderTokens);
            Some((CSS_Node::Declaration
            {
                Text: Header_Lines.join(" "),
                Preceding_Newlines: RulePrecedingNewlines,
            }, I))
        }
        else
        {
            None
        }
    }

    fn BuildHeaderLines(Tokens: &[CSS_Token]) -> (Vec<String>, bool)
    {
        if Tokens.is_empty() { return (Vec::new(), false); }

        let IsAtRule = Tokens.first().map_or(false, |t| t.Text.starts_with('@'));

        // Check if there was any newline in the source header (after the first token)
        let HasSourceNewlines = Tokens.iter().skip(1).any(|t| t.Preceding_Newlines > 0);

        if HasSourceNewlines && !IsAtRule
        {
            let mut Lines: Vec<Vec<&CSS_Token>> = Vec::new();
            let mut CurrentLine: Vec<&CSS_Token> = Vec::new();

            for (Idx, T) in Tokens.iter().enumerate()
            {
                if Idx > 0 && T.Preceding_Newlines > 0 && !CurrentLine.is_empty()
                {
                    Lines.push(CurrentLine);
                    CurrentLine = Vec::new();
                }
                CurrentLine.push(T);
            }
            if !CurrentLine.is_empty()
            {
                Lines.push(CurrentLine);
            }

            let FormattedLines: Vec<String> = Lines.iter()
                .map(|line| Self::FormatHeaderTokens(line, IsAtRule))
                .filter(|s| !s.is_empty())
                .collect();

            if FormattedLines.len() > 1
            {
                return (FormattedLines, true);
            }
        }

        let SingleLine = Self::FormatHeaderTokens(&Tokens.iter().collect::<Vec<_>>(), IsAtRule);
        (vec![SingleLine], false)
    }

    fn FormatHeaderTokens(Tokens: &[&CSS_Token], IsAtRule: bool) -> String
    {
        let Words: Vec<String> = Tokens.iter().map(|t| t.Text.clone()).collect();
        Self::FormatHeaderWords(&Words, IsAtRule)
    }

    fn FormatHeaderWords(Words: &[String], IsAtRule: bool) -> String
    {
        let mut Res = String::new();
        let mut ParenDepth = 0;

        for (Idx, W) in Words.iter().enumerate()
        {
            if W == ":"
            {
                if !Res.ends_with(", ")
                && !Res.ends_with("> ")
                && !Res.ends_with("+ ")
                && !Res.ends_with("~ ")
                {
                    while Res.ends_with(' ') { Res.pop(); }
                }

                let IsPropertyColon = IsAtRule
                    && ParenDepth > 0
                    && Idx > 0
                    && Words[Idx - 1] != "("
                    && Words[Idx - 1] != ":"
                    && Words[Idx - 1] != ",";

                if IsPropertyColon
                { Res.push_str(": "); }
                else
                { Res.push(':'); }
            }
            else if W == ","
            {
                while Res.ends_with(' ') { Res.pop(); }
                if Idx + 1 == Words.len()
                { Res.push(','); }
                else
                { Res.push_str(", "); }
            }
            else if W == "("
            {
                let PrevIsPseudo = Words.get(Idx.saturating_sub(1)).map_or(false, |p| {
                    p.starts_with(':') || (Idx >= 2 && Words[Idx - 2] == ":")
                });
                if !Res.is_empty() && !Res.ends_with(' ') && !Res.ends_with(':') && !PrevIsPseudo
                { Res.push(' '); }
                while Res.ends_with(' ') && PrevIsPseudo
                { Res.pop(); }
                Res.push('(');
                ParenDepth += 1;
            }
            else if W == ")"
            {
                while Res.ends_with(' ') { Res.pop(); }
                Res.push(')');
                if ParenDepth > 0 { ParenDepth -= 1; }
            }
            else if W == "]"
            {
                while Res.ends_with(' ') { Res.pop(); }
                Res.push(']');
            }
            else
            {
                let PrecededByNoSpace = Res.ends_with(':') || Res.ends_with('=') || Res.ends_with('[');
                if !Res.is_empty()
                && !Res.ends_with(' ')
                && !Res.ends_with('(')
                && !PrecededByNoSpace
                { Res.push(' '); }
                Res.push_str(W);
            }
        }
        Res
    }

    fn FormatDeclaration(Decl: &str) -> String
    {
        let mut Result = String::new();
        let Parts: Vec<&str> = Decl.split_whitespace().collect();
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
            else if Part == "("
            { while Result.ends_with(' ') { Result.pop(); } Result.push('('); }
            else if Part == ")"
            { while Result.ends_with(' ') { Result.pop(); } Result.push(')'); }
            else
            {
                if !Result.is_empty() && !Result.ends_with(' ') && !Result.ends_with('(')
                { Result.push(' '); }
                Result.push_str(Part);
            }
            I += 1;
        }

        if !Result.ends_with(';') && !Result.is_empty()
        { Result.push(';'); }
        Result
    }

    fn Emit(Nodes: &[CSS_Node], Config: &Config_Settings, BaseLevel: usize) -> String
    {
        let mut Out = String::new();
        for (Idx, Node) in Nodes.iter().enumerate()
        {
            if Idx > 0
            {
                let PrecedingNewlines = Node.Preceding_Newlines();
                if Config.Preserve_Empty_Lines && PrecedingNewlines >= 2
                {
                    if !Out.ends_with("\n\n")
                    {
                        if !Out.ends_with('\n') { Out.push('\n'); }
                        Out.push('\n');
                    }
                }
                else
                {
                    if !Out.ends_with('\n')
                    { Out.push('\n'); }
                }
            }
            else
            {
                if Config.Preserve_Empty_Lines && Node.Preceding_Newlines() >= 2
                {
                    Out.push('\n');
                }
            }
            Self::EmitNode(&mut Out, Node, Config, BaseLevel);
        }
        if !Out.is_empty() && !Out.ends_with('\n')
        { Out.push('\n'); }
        Out
    }

    fn EmitNode(Out: &mut String, Node: &CSS_Node, Config: &Config_Settings, Level: usize)
    {
        let Indent = " ".repeat(Level * Config.Indent_Width);

        match Node
        {
            CSS_Node::Comment { Text, .. } =>
            {
                Out.push_str(&Indent);
                Out.push_str(Text);
                Out.push('\n');
            }
            CSS_Node::Declaration { Text, .. } =>
            {
                Out.push_str(&Indent);
                Out.push_str(&Self::FormatDeclaration(Text));
                Out.push('\n');
            }
            CSS_Node::RuleSet { Header_Lines, Children, Was_Multiline_Selector, .. } =>
            {
                let HasNestedRules = Children.iter().any(|c| matches!(c, CSS_Node::RuleSet { .. }));
                let IsSingleDecl = Level == 0
                    && !HasNestedRules
                    && !Was_Multiline_Selector
                    && Header_Lines.len() == 1
                    && Children.len() == 1
                    && matches!(Children[0], CSS_Node::Declaration { .. });

                if IsSingleDecl
                {
                    if let CSS_Node::Declaration { Text, .. } = &Children[0]
                    {
                        let FormattedD = Self::FormatDeclaration(Text);
                        let MonolithLine = format!("{}{} {{ {} }}", Indent, Header_Lines[0], FormattedD);
                        if MonolithLine.len() <= Config.Canvas_Width
                        {
                            Out.push_str(&MonolithLine);
                            Out.push('\n');
                            return;
                        }
                    }
                }

                // Expanded Symmetry
                for HeaderLine in Header_Lines
                {
                    Out.push_str(&Indent);
                    Out.push_str(HeaderLine);
                    Out.push('\n');
                }

                Out.push_str(&Indent);
                Out.push_str("{\n");

                for (ChildIdx, Child) in Children.iter().enumerate()
                {
                    if ChildIdx > 0 && Config.Preserve_Empty_Lines && Child.Preceding_Newlines() >= 2
                    {
                        if !Out.ends_with("\n\n")
                        { Out.push('\n'); }
                    }
                    Self::EmitNode(Out, Child, Config, Level + 1);
                }

                Out.push_str(&Indent);
                Out.push_str("}\n");
            }
        }
    }
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::formatter::Source_Formatter;
use crate::languages::css::CSS_Formatter;
use crate::languages::js_ts::JS_TS_Formatter;
use crate::types:: { Config_Settings, Source_Language };

pub struct Polyglot_Formatter;

impl Polyglot_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let Tokens = Self::Tokenize(Source);
        let Nodes = Self::BuildTree( & Tokens);
        let Formatted = Self::Emit( & Nodes, Config, 0);
        Ok(Formatted)
    }

    fn Tokenize(Source: & str) -> Vec<HTML_Token>
    {
        let Chars: Vec < char > = Source.chars().collect();
        let Len = Chars.len();
        let mut I = 0;
        let mut Tokens = Vec:: new ();

        while I < Len
        {
            // 1. Comments <!-- ... -->
            if I + 3 < Len && Chars[I] == '<' && Chars[I + 1] == '!' && Chars[I + 2] == '-' && Chars[I + 3] == '-'
            {
                let Start = I;
                I += 4;
                while I + 2 < Len && !(Chars[I] == '-' && Chars[I + 1] == '-' && Chars[I + 2] == '>')
                { I += 1; }
                if I + 2 < Len
                { I += 3; }
                else
                { I = Len; }
                let Text: String = Chars[Start..I].iter().collect();
                Tokens.push(HTML_Token::Comment(Text));
                continue;
            }

            // 2. Doctype <!DOCTYPE ...>
            if I + 8 < Len && Chars[I] == '<' && Chars[I + 1] == '!'
            {
                let Sub: String = Chars[I..I + 9].iter().collect();
                if Sub.eq_ignore_ascii_case("<!DOCTYPE")
                {
                    let Start = I;
                    while I < Len && Chars[I] != '>'
                    { I += 1; }
                    if I < Len { I += 1; }
                    let Text: String = Chars[Start..I].iter().collect();
                    Tokens.push(HTML_Token::Doctype(Text));
                    continue;
                }
            }

            // 3. PHP Block <?php ... ?> or <?= ... ?>
            if I + 1 < Len && Chars[I] == '<' && Chars[I + 1] == '?'
            {
                let Start = I;
                I += 2;
                while I + 1 < Len && !(Chars[I] == '?' && Chars[I + 1] == '>')
                { I += 1; }
                if I + 1 < Len { I += 2; } else { I = Len; }
                let Text: String = Chars[Start..I].iter().collect();
                Tokens.push(HTML_Token::PhpBlock(Text));
                continue;
            }

            // 4. Tags <tag ...> or </tag>
            if Chars[I] == '<' && I + 1 < Len && (Chars[I + 1].is_alphabetic() || Chars[I + 1] == '/' || Chars[I + 1] == '!')
            {
                let Start = I;
                let IsClose = Chars[I + 1] == '/';
                I += 1;
                if IsClose { I += 1; }

                // Tag name
                let NameStart = I;
                while I < Len && (Chars[I].is_alphanumeric() || Chars[I] == '-' || Chars[I] == ':')
                { I += 1; }
                let TagName: String = Chars[NameStart..I].iter().collect();
                let TagNameLower = TagName.to_lowercase();

                // Consume until '>'
                let mut InQuote: Option < char > = None;
                while I < Len
                {
                    let C = Chars[I];
                    if let Some(Q) = InQuote
                    { if C == Q { InQuote = None; } }
                    else if C == '"' || C == '\''
                    { InQuote = Some(C); }
                    else if C == '>'
                    { I += 1; break ; }
                    I += 1;
                }
                let RawTag: String = Chars[Start..I].iter().collect();

                // Check for embedded <script> or <style>
                if ! IsClose && TagNameLower == "script"
                {
                    let ContentStart = I;
                    let mut FoundEnd = false;
                    while I + 8 < Len
                    {
                        if Chars[I] == '<' && Chars[I + 1] == '/'
                        {
                            let Check: String = Chars[I + 2..I + 8].iter().collect();
                            if Check.eq_ignore_ascii_case("script")
                            {
                                let Content: String = Chars[ContentStart..I].iter().collect();
                                while I < Len && Chars[I] != '>' { I += 1; }
                                if I < Len { I += 1; }
                                Tokens.push(HTML_Token::ScriptBlock { OpenTag: RawTag.clone(), Content, CloseTag: "</script>".to_string(), });
                                FoundEnd = true;
                                break;
                            }
                        }
                        I += 1;
                    }
                    if FoundEnd { continue ; }
                }

                if ! IsClose && TagNameLower == "style"
                {
                    let ContentStart = I;
                    let mut FoundEnd = false;
                    while I + 7 < Len
                    {
                        if Chars[I] == '<' && Chars[I + 1] == '/'
                        {
                            let Check: String = Chars[I + 2..I + 7].iter().collect();
                            if Check.eq_ignore_ascii_case("style")
                            {
                                let Content: String = Chars[ContentStart..I].iter().collect();
                                while I < Len && Chars[I] != '>' { I += 1; }
                                if I < Len { I += 1; }
                                Tokens.push(HTML_Token::StyleBlock { OpenTag: RawTag.clone(), Content, CloseTag: "</style>".to_string(), });
                                FoundEnd = true;
                                break;
                            }
                        }
                        I += 1;
                    }
                    if FoundEnd { continue ; }
                }

                let IsSelfClosing = RawTag.ends_with("/>");
                let IsVoid = Self::IsVoidElement( & TagNameLower);

                if IsClose
                { Tokens.push(HTML_Token::CloseTag(TagNameLower)); }
                else
                { Tokens.push(HTML_Token::OpenTag { Name: TagNameLower, Raw: RawTag, IsVoid: IsVoid || IsSelfClosing, }); }
                continue;
            }

            // 5. Text content between tags
            let Start = I;
            while I < Len && Chars[I] != '<'
            { I += 1; }
            let Text: String = Chars[Start..I].iter().collect();
            if ! Text.trim().is_empty()
            { Tokens.push(HTML_Token::Text(Text)); }
        }

        Tokens
    }

    fn IsVoidElement(Name: & str) -> bool
    { matches ! (Name, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr") }

    fn BuildTree(Tokens: & [HTML_Token]) -> Vec<HTML_Node>
    {
        let mut RootNodes = Vec:: new ();
        let mut I = 0;
        let Len = Tokens.len();

        while I < Len
        { if let Some((Node, NewI)) = Self::ParseNode(Tokens, I) { RootNodes.push(Node); I = NewI; } else { I += 1; } }

        RootNodes
    }

    fn ParseNode(Tokens: & [HTML_Token], Index: usize) -> Option < (HTML_Node, usize) >
    {
        if Index >= Tokens.len() { return None; }

        match & Tokens[Index]
        {
            HTML_Token::Doctype(d) => Some((HTML_Node::Doctype(d.clone()), Index + 1)),
            HTML_Token::Comment(c) => Some((HTML_Node::Comment(c.clone()), Index + 1)),
            HTML_Token::PhpBlock(p) => Some((HTML_Node::PhpBlock(p.clone()), Index + 1)),
            HTML_Token::Text(t) => Some((HTML_Node::Text(t.clone()), Index + 1)),
            HTML_Token::ScriptBlock { OpenTag, Content, CloseTag }
            =>
            { Some((HTML_Node::ScriptBlock { OpenTag: OpenTag.clone(), Content: Content.clone(), CloseTag: CloseTag.clone(), }, Index + 1)) }
            HTML_Token::StyleBlock { OpenTag, Content, CloseTag }
            =>
            { Some((HTML_Node::StyleBlock { OpenTag: OpenTag.clone(), Content: Content.clone(), CloseTag: CloseTag.clone(), }, Index + 1)) }
            HTML_Token::OpenTag { Name, Raw, IsVoid }
            =>
            {
                if * IsVoid
                { return Some((HTML_Node::VoidElement { Raw: Raw.clone() }, Index + 1)); }

                let mut Children = Vec:: new ();
                let mut I = Index + 1;
                let TargetName = Name.clone();

                while I < Tokens.len()
                {
                    match & Tokens[I]
                    {
                        HTML_Token::CloseTag(CloseName) if CloseName == & TargetName =>
                        { I += 1; break ; }
                        _ =>
                        { if let Some((ChildNode, NewI)) = Self::ParseNode(Tokens, I) { Children.push(ChildNode); I = NewI; } else { I += 1; } }
                    }
                }

                Some((HTML_Node::Element { Name: TargetName, OpenTag: Raw.clone(), Children, }, I))
            }
            HTML_Token::CloseTag(_) => None,
        }
    }

    fn Emit(Nodes: & [HTML_Node], Config: & Config_Settings, Level: usize) -> String
    {
        let mut Out = String:: new ();
        for Node in Nodes
        { Self::EmitNode( & mut Out, Node, Config, Level); }
        Out
    }

    fn EmitNode(Out: & mut String, Node: & HTML_Node, Config: & Config_Settings, Level: usize)
    {
        let Indent = " ".repeat(Level * Config.Indent_Width);

        match Node
        {
            HTML_Node::Doctype(d) =>
            { Out.push_str(d); Out.push('\n'); }
            HTML_Node::Comment(c) =>
            { Out.push_str( & Indent); Out.push_str(c.trim()); Out.push('\n'); }
            HTML_Node::PhpBlock(p) =>
            { Out.push_str( & Indent); Out.push_str(p.trim()); Out.push('\n'); }
            HTML_Node::Text(t) =>
            {
                let Trimmed = t.trim();
                if ! Trimmed.is_empty()
                { Out.push_str( & Indent); Out.push_str(Trimmed); Out.push('\n'); }
            }
            HTML_Node::VoidElement { Raw }
            =>
            { Out.push_str( & Indent); Out.push_str(Raw.trim()); Out.push('\n'); }
            HTML_Node::ScriptBlock { OpenTag, Content, CloseTag }
            =>
            {
                Out.push_str( & Indent);
                Out.push_str(OpenTag.trim());
                Out.push('\n');

                let TrimmedContent = Content.trim();
                if ! TrimmedContent.is_empty()
                {
                    let mut SubConfig = Config.clone();
                    SubConfig.Language = Source_Language::JavaScript;
                    let FormattedJS = JS_TS_Formatter::Format(TrimmedContent, & SubConfig)
                   .unwrap_or_else( | _ | TrimmedContent.to_string());

                    let ChildIndent = " ".repeat((Level + 1) * Config.Indent_Width);
                    for Line in FormattedJS.lines()
                    { if Line.trim().is_empty() { Out.push('\n'); } else { Out.push_str( & ChildIndent); Out.push_str(Line); Out.push('\n'); } }
                }

                Out.push_str( & Indent);
                Out.push_str(CloseTag.trim());
                Out.push('\n');
            }
            HTML_Node::StyleBlock { OpenTag, Content, CloseTag }
            =>
            {
                Out.push_str( & Indent);
                Out.push_str(OpenTag.trim());
                Out.push('\n');

                let TrimmedContent = Content.trim();
                if ! TrimmedContent.is_empty()
                {
                    let mut SubConfig = Config.clone();
                    SubConfig.Language = Source_Language::Css;
                    let FormattedCSS = CSS_Formatter::FormatWithBaseLevel(TrimmedContent, & SubConfig, Level + 1)
                   .unwrap_or_else( | _ | TrimmedContent.to_string());

                    Out.push_str( & FormattedCSS);
                }

                Out.push_str( & Indent);
                Out.push_str(CloseTag.trim());
                Out.push('\n');
            }
            HTML_Node::Element { Name, OpenTag, Children }
            =>
            {
                // Check if can be a 1-line Monolith
                let CanBeMonolith = Children.len() == 1
                && match & Children[0]
                {
                    HTML_Node::Text(t) => ! t.contains('\n') && (Indent.len() + OpenTag.len() + t.trim().len() + Name.len() + 3 <= Config.Canvas_Width),
                    _ => false,
                };

                if CanBeMonolith
                {
                    if let HTML_Node::Text(t) = & Children[0]
                    {
                        Out.push_str( & Indent);
                        Out.push_str(OpenTag.trim());
                        Out.push_str(t.trim());
                        Out.push_str( & format ! ("</{}>\n", Name));
                        return ;
                    }
                }

                // Expanded Symmetry
                Out.push_str( & Indent);
                Out.push_str(OpenTag.trim());
                Out.push('\n');

                for Child in Children
                { Self::EmitNode(Out, Child, Config, Level + 1); }

                Out.push_str( & Indent);
                Out.push_str( & format ! ("</{}>\n", Name));
            }
        }
    }
}

#[derive(Debug, Clone)]
enum HTML_Token
{
    Doctype(String),
    Comment(String),
    PhpBlock(String),
    OpenTag { Name: String, Raw: String, IsVoid: bool },

    CloseTag(String),
    ScriptBlock { OpenTag: String, Content: String, CloseTag: String },

    StyleBlock { OpenTag: String, Content: String, CloseTag: String },
    Text(String),
}

#[derive(Debug, Clone)]
enum HTML_Node
{
    Doctype(String),
    Comment(String),
    PhpBlock(String),
    VoidElement { Raw: String },

    Element { Name: String, OpenTag: String, Children: Vec<HTML_Node> },
    ScriptBlock { OpenTag: String, Content: String, CloseTag: String },
    StyleBlock { OpenTag: String, Content: String, CloseTag: String },
    Text(String),
}

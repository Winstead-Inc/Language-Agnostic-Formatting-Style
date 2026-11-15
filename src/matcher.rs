#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use std::fmt;
use crate::container_tree:: { Container_Node, Syntax_Node };
use crate::profile::Language_Profile;
use crate::types:: { Container_Kind, Token_Info, Token_Kind };

pub const DEFAULT_MAX_DEPTH: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match_Error_Code
{ E_ORPHAN, E_MISMATCH, E_UNCLOSED, E_UNTERMINATED, E_BRANCH, E_DEPTH, E_AMBIGUOUS, E_ENCODING, E_VERIFY, }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Match_Error
{
    Orphan
    { Closer: Token_Info, },
    Mismatch
    { Opener: Token_Info, Closer: Token_Info, },
    Unclosed
    { Opener: Token_Info, },
    Unterminated
    { Payload_Start: Token_Info, },
    Branch
    { Directive: Token_Info, },
    Depth
    { Opener: Token_Info, Max_Depth: usize, },
    Ambiguous
    { Token: Token_Info, },
    Encoding
    { Byte_Offset: usize, },
}

impl Match_Error
{
    pub fn Code( & self) -> Match_Error_Code
    {
        match self
        {
            Match_Error::Orphan {.. }
            => Match_Error_Code::E_ORPHAN,
            Match_Error::Mismatch {.. }
            => Match_Error_Code::E_MISMATCH,
            Match_Error::Unclosed {.. }
            => Match_Error_Code::E_UNCLOSED,
            Match_Error::Unterminated {.. }
            => Match_Error_Code::E_UNTERMINATED,
            Match_Error::Branch {.. }
            => Match_Error_Code::E_BRANCH,
            Match_Error::Depth {.. }
            => Match_Error_Code::E_DEPTH,
            Match_Error::Ambiguous {.. }
            => Match_Error_Code::E_AMBIGUOUS,
            Match_Error::Encoding {.. }
            => Match_Error_Code::E_ENCODING,
        }
    }
}

impl fmt::Display for Match_Error
{
    fn fmt( & self, F: & mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Match_Error::Orphan { Closer }
            =>
            { write ! (F, "E_ORPHAN: Stray closing delimiter '{}' at line {}, column {} has no opening partner", Closer.Text, Closer.Line, Closer.Column) }
            Match_Error::Mismatch { Opener, Closer }
            =>
            {
                write!
                (
                    F,
                    "E_MISMATCH: Mismatched delimiter: opener '{}' at line {}, column {} closed by incompatible '{}' at line {}, column {}",
                    Opener.Text, Opener.Line, Opener.Column, Closer.Text, Closer.Line, Closer.Column
                )
            }
            Match_Error::Unclosed { Opener }
            =>
            { write ! (F, "E_UNCLOSED: Unclosed delimiter '{}' at line {}, column {} reaches end of file", Opener.Text, Opener.Line, Opener.Column) }
            Match_Error::Unterminated { Payload_Start }
            =>
            { write ! (F, "E_UNTERMINATED: Unterminated literal starting at line {}, column {}", Payload_Start.Line, Payload_Start.Column) }
            Match_Error::Branch { Directive }
            =>
            { write ! (F, "E_BRANCH: Conditional branches leave different container stacks at directive '{}' at line {}, column {}", Directive.Text, Directive.Line, Directive.Column) }
            Match_Error::Depth { Opener, Max_Depth }
            =>
            { write ! (F, "E_DEPTH: Maximum container nesting depth ({}) exceeded at line {}, column {}", Max_Depth, Opener.Line, Opener.Column) }
            Match_Error::Ambiguous { Token }
            =>
            { write ! (F, "E_AMBIGUOUS: Lexical ambiguity could not be resolved at '{}' at line {}, column {}", Token.Text, Token.Line, Token.Column) }
            Match_Error::Encoding { Byte_Offset }
            =>
            { write ! (F, "E_ENCODING: Invalid UTF-8 sequence at byte offset {}", Byte_Offset) }
        }
    }
}

impl std::error::Error for Match_Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair_Record
{ pub Opener: Token_Info, pub Closer: Token_Info, pub Kind: Container_Kind, }

#[derive(Debug, Clone)]
struct Matcher_Frame
{ Container: Container_Node, Is_Tentative: bool , Paren_Depth: usize, Bracket_Depth: usize, Brace_Depth: usize, }

pub struct Container_Matcher
{ pub Max_Depth: usize, }

impl Container_Matcher
{
    pub fn New() -> Self
    { Container_Matcher { Max_Depth: DEFAULT_MAX_DEPTH, } }

    pub fn WithMaxDepth(Max_Depth: usize) -> Self
    { Container_Matcher { Max_Depth } }

    pub fn Match( & self, Tokens: & [Token_Info], Profile: & 'static Language_Profile,) -> Result < (Vec<Syntax_Node>, Vec<Pair_Record>), Match_Error >
    {
        let mut Root_Nodes: Vec<Syntax_Node> = Vec:: new ();
        let mut Stack: Vec<Matcher_Frame> = Vec:: new ();
        let mut Pairs: Vec<Pair_Record> = Vec:: new ();
        let mut Directive_Snapshots: Vec<Vec<Matcher_Frame>> = Vec:: new ();

        let mut Idx = 0;
        while Idx < Tokens.len()
        {
            let Token = & Tokens[Idx];

            // Preprocessor conditional branch handling (§8 E_BRANCH)
            if Profile.Directives.Enabled
            {
                if let Token_Kind::Preprocessor(ref Dir) = Token.Kind
                {
                    let Trimmed = Dir.trim_start();
                    let Cmd = if let Some(Rest) = Trimmed.strip_prefix('#')
                    { Rest.trim_start().split_whitespace().next().unwrap_or("") }
                    else
                    { "" };

                    match Cmd
                    {
                        "if" | "ifdef" | "ifndef" =>
                        { Directive_Snapshots.push(Stack.clone()); }
                        "else" | "elif" =>
                        {
                            if let Some(Snapshot) = Directive_Snapshots.last()
                            {
                                while Stack.len() > Snapshot.len()
                                {
                                    let Unclosed = Stack.pop().unwrap();
                                    let Node = Syntax_Node::Container(Unclosed.Container);
                                    if let Some(Top) = Stack.last_mut()
                                    { Top.Container.Children.push(Node); }
                                    else
                                    { Root_Nodes.push(Node); }
                                }
                            }
                        }
                        "endif" =>
                        { Directive_Snapshots.pop(); }
                        _ => {}
                    }
                }
            }

            // Shape automaton demotion check for tentative containers on stack (§6 line 94, §7 T2)
            while let Some(Top) = Stack.last()
            {
                if ! Top.Is_Tentative
                { break ; }

                if Profile.IsTentativeAngleTokenRejected( & Token.Text, Top.Paren_Depth, Top.Brace_Depth, Top.Bracket_Depth)
                {
                    let Demoted = Stack.pop().unwrap();
                    let mut Demoted_Opener = Demoted.Container.Open_Token;
                    Demoted_Opener.Kind = Token_Kind::Operator(Demoted_Opener.Text.clone());

                    if let Some(Parent) = Stack.last_mut()
                    {
                        Parent.Container.Children.push(Syntax_Node::Token(Demoted_Opener));
                        Parent.Container.Children.extend(Demoted.Container.Children);
                    }
                    else
                    { Root_Nodes.push(Syntax_Node::Token(Demoted_Opener)); Root_Nodes.extend(Demoted.Container.Children); }
                }
                else
                { break ; }
            }

            // Update parent container forced break and statement count
            if let Some(Top) = Stack.last_mut()
            {
                if Token.Text.contains('\n')
                || matches!(Token.Kind, Token_Kind::Comment_Line(_) | Token_Kind::Preprocessor(_))
                { Top.Container.Has_Forced_Break = true; }

                if matches!(Token.Kind, Token_Kind::Keyword(ref K) if K == "public" || K == "private" || K == "protected")
                { if Idx + 1 < Tokens.len() && Tokens[Idx + 1].Text == ":" { Top.Container.Has_Forced_Break = true; } }

                if Token.Text == ";"
                { Top.Container.Statement_Count += 1; }

                if Top.Is_Tentative
                {
                    match Token.Text.as_str()
                    {
                        "(" => Top.Paren_Depth += 1,
                        ")" if Top.Paren_Depth > 0 => Top.Paren_Depth -= 1,
                        "[" => Top.Bracket_Depth += 1,
                        "]" if Top.Bracket_Depth > 0 => Top.Bracket_Depth -= 1,
                        "{" | "${" => Top.Brace_Depth += 1,
                        "}" if Top.Brace_Depth > 0 => Top.Brace_Depth -= 1,
                        _ => {}
                    }
                }
            }

            match & Token.Kind
            {
                Token_Kind::Open_Delim(Kind) =>
                {
                    if Stack.len() >= self.Max_Depth
                    { return Err(Match_Error::Depth { Opener: Token.clone(), Max_Depth: self.Max_Depth, }); }

                    let Frame = Matcher_Frame
                    { Container: Container_Node::New( * Kind, Token.clone()), Is_Tentative: false, Paren_Depth: 0, Bracket_Depth: 0, Brace_Depth: 0, };

                    Stack.push(Frame);
                    Idx += 1;
                }

                Token_Kind::Close_Delim(CloseKind) =>
                {
                    if Stack.is_empty()
                    { return Err(Match_Error::Orphan { Closer: Token.clone(), }); }

                    let Top_Kind = Stack.last().unwrap().Container.Kind;
                    if Top_Kind != * CloseKind
                    {
                        return Err(Match_Error::Mismatch { Opener: Stack.last().unwrap().Container.Open_Token.clone(), Closer: Token.clone(), });
                    }

                    let mut Completed_Frame = Stack.pop().unwrap();
                    Completed_Frame.Container.Close_Token = Some(Token.clone());

                    Pairs.push(Pair_Record { Opener: Completed_Frame.Container.Open_Token.clone(), Closer: Token.clone(), Kind: * CloseKind, });

                    // Tail glue consumption per M7
                    Idx += 1;
                    while Idx < Tokens.len()
                    {
                        let Next = & Tokens[Idx];
                        if Next.Preceding_Newlines > 0
                        { break ; }

                        if Next.Text == ";" || Next.Text == ","
                        {
                            Completed_Frame.Container.Tail_Glue.push(Next.clone());
                            if Next.Text == ";"
                            { if let Some(Parent) = Stack.last_mut() { Parent.Container.Statement_Count += 1; } }
                            Idx += 1;
                        }
                        else if Completed_Frame.Container.Kind == Container_Kind::Angle
                        && Next.Text == "["
                        && Idx + 1 < Tokens.len()
                        && Tokens[Idx + 1].Text == "]"
                        && Tokens[Idx + 1].Preceding_Newlines == 0
                        {
                            Completed_Frame.Container.Tail_Glue.push(Next.clone());
                            Completed_Frame.Container.Tail_Glue.push(Tokens[Idx + 1].clone());
                            Idx += 2;
                        }
                        else if Completed_Frame.Container.Kind == Container_Kind::Bracket && Next.Text == ":"
                        {
                            Completed_Frame.Container.Tail_Glue.push(Next.clone());
                            Idx += 1;
                        }
                        else
                        { break ; }
                    }

                    // Bubble forced break to parent
                    if Completed_Frame.Container.Has_Forced_Break
                    { if let Some(Parent) = Stack.last_mut() { Parent.Container.Has_Forced_Break = true; } }

                    let Completed_Node = Syntax_Node::Container(Completed_Frame.Container);
                    if let Some(Parent) = Stack.last_mut()
                    { Parent.Container.Children.push(Completed_Node); }
                    else
                    { Root_Nodes.push(Completed_Node); }
                }

                _ =>
                {
                    let Child_Node = Syntax_Node::Token(Token.clone());
                    if let Some(Parent) = Stack.last_mut()
                    { Parent.Container.Children.push(Child_Node); }
                    else
                    { Root_Nodes.push(Child_Node); }
                    Idx += 1;
                }
            }
        }

        // EOF tentative demotion (§6 line 124, §7 T3)
        while let Some(Top) = Stack.last()
        {
            if Top.Is_Tentative
            {
                let Demoted = Stack.pop().unwrap();
                let mut Demoted_Opener = Demoted.Container.Open_Token;
                Demoted_Opener.Kind = Token_Kind::Operator(Demoted_Opener.Text.clone());

                if let Some(Parent) = Stack.last_mut()
                {
                    Parent.Container.Children.push(Syntax_Node::Token(Demoted_Opener));
                    Parent.Container.Children.extend(Demoted.Container.Children);
                }
                else
                { Root_Nodes.push(Syntax_Node::Token(Demoted_Opener)); Root_Nodes.extend(Demoted.Container.Children); }
            }
            else
            { break ; }
        }

        // Check for unclosed containers at EOF (§6 line 126, §8 E_UNCLOSED)
        if let Some(Unclosed) = Stack.pop()
        { return Err(Match_Error::Unclosed { Opener: Unclosed.Container.Open_Token, }); }

        Ok((Root_Nodes, Pairs))
    }

    pub fn BuildTree( & self, Tokens: & [Token_Info], Profile: & 'static Language_Profile,) -> Result<Vec<Syntax_Node>, Match_Error>
    { self.Match(Tokens, Profile).map( | (Nodes, _) | Nodes) }
}

#[cfg(test)]
mod Tests
{
    use super:: * ;
    use crate::types::Source_Language;

   #[test]
    fn Test_Matcher_Valid_Pairs()
    {
        let Tokens = vec!
        [
            Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Paren), "(".to_string(), 1, 1, 0),
            Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Bracket), "[".to_string(), 1, 2, 0),
            Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "{".to_string(), 1, 3, 0),
            Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), 1, 4, 0),
            Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Bracket), "]".to_string(), 1, 5, 0),
            Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Paren), ")".to_string(), 1, 6, 0),
        ];

        let Matcher = Container_Matcher::New();
        let Profile = Source_Language::Generic.Profile();
        let (Nodes, Pairs) = Matcher.Match( & Tokens, Profile).expect("Matching must succeed");

        assert_eq!(Nodes.len(), 1);
        assert_eq!(Pairs.len(), 3);
        assert_eq!(Pairs[0].Kind, Container_Kind::Brace);
        assert_eq!(Pairs[1].Kind, Container_Kind::Bracket);
        assert_eq!(Pairs[2].Kind, Container_Kind::Paren);
    }

   #[test]
    fn Test_Matcher_Orphan_Closer()
    {
        let Tokens = vec![Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Brace), "}".to_string(), 1, 1, 0),];

        let Matcher = Container_Matcher::New();
        let Profile = Source_Language::Generic.Profile();
        let Error = Matcher.Match( & Tokens, Profile).unwrap_err();

        assert_eq!(Error.Code(), Match_Error_Code::E_ORPHAN);
        assert!(Error.to_string().contains("E_ORPHAN"));
    }

   #[test]
    fn Test_Matcher_Mismatch()
    {
        let Tokens = vec!
        [
            Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Paren), "(".to_string(), 1, 1, 0),
            Token_Info::New(Token_Kind::Close_Delim(Container_Kind::Bracket), "]".to_string(), 1, 2, 0),
        ];

        let Matcher = Container_Matcher::New();
        let Profile = Source_Language::Generic.Profile();
        let Error = Matcher.Match( & Tokens, Profile).unwrap_err();

        assert_eq!(Error.Code(), Match_Error_Code::E_MISMATCH);
        assert!(Error.to_string().contains("E_MISMATCH"));
        assert!(Error.to_string().contains("opener '('"));
        assert!(Error.to_string().contains("incompatible ']'"));
    }

   #[test]
    fn Test_Matcher_Unclosed_Opener()
    {
        let Tokens = vec![Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Brace), "{".to_string(), 1, 1, 0),];

        let Matcher = Container_Matcher::New();
        let Profile = Source_Language::Generic.Profile();
        let Error = Matcher.Match( & Tokens, Profile).unwrap_err();

        assert_eq!(Error.Code(), Match_Error_Code::E_UNCLOSED);
        assert!(Error.to_string().contains("E_UNCLOSED"));
    }

   #[test]
    fn Test_Matcher_Max_Depth()
    {
        let mut Tokens = Vec:: new ();
        for _ in 0..10
        { Tokens.push(Token_Info::New(Token_Kind::Open_Delim(Container_Kind::Paren), "(".to_string(), 1, 1, 0)); }

        let Matcher = Container_Matcher::WithMaxDepth(5);
        let Profile = Source_Language::Generic.Profile();
        let Error = Matcher.Match( & Tokens, Profile).unwrap_err();

        assert_eq!(Error.Code(), Match_Error_Code::E_DEPTH);
        assert!(Error.to_string().contains("E_DEPTH"));
    }
}

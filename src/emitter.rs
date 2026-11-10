#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::container_tree:: { Container_Node, Syntax_Node };
use crate::planner::Layout_Planner;
use crate::types:: { Config_Settings, Container_Kind, Layout_State, Source_Language, Token_Info, Token_Kind };

pub struct Code_Emitter<'a>
{
    Config: & 'a Config_Settings, Planner: Layout_Planner<'a>, Output: String, Current_Line_Len: usize, Prev_Line_Len: usize, Line_Has_Content: bool,
}

impl <'a> Code_Emitter<'a>
{
    pub fn New(Config: & 'a Config_Settings) -> Self
    {
        Code_Emitter { Config, Planner: Layout_Planner::New(Config), Output: String::new (), Current_Line_Len: 0, Prev_Line_Len: 0, Line_Has_Content: false, }
    }

    pub fn Emit(mut self, Nodes: & [Syntax_Node]) -> String
    {
        self.EmitNodeList(Nodes, 0);
        if ! self.Output.ends_with('\n') { self.Output.push('\n'); }
        self.Output
    }

    fn WriteNewline( & mut self)
    {
        while self.Output.ends_with(' ') { self.Output.pop(); }
        self.Prev_Line_Len = self.Current_Line_Len;
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

    fn EmitNodeList( & mut self, Nodes: & [Syntax_Node], Level: usize)
    {
        let Total = Nodes.len();
        let mut Index = 0;

        while Index < Total
        {
            let Node = & Nodes[Index];

            match Node
            {
                Syntax_Node::Token(T) =>
                {
                    self.EmitSingleToken(T, Level);
                    if Level == 0 && (T.Text == ";" || matches ! (T.Kind, Token_Kind::Preprocessor(_))) { if Self::IsImportStatementEnding(Nodes, Index) && Self::HasMoreNonImportStatements(Nodes, Index + 1) { self.EnsureBlankLine(); } }
                }
                Syntax_Node::Container(C) =>
                {
                    let IsTemplate = Index > 0 && matches!( & Nodes[Index - 1], Syntax_Node::Token(T) if T.Text == "template");

                    let ShouldAttachInline = if Index > 0
                    {
                        match & Nodes[Index - 1]
                        {
                            Syntax_Node::Token(PrevT) =>
                            {
                                if matches!(PrevT.Kind, Token_Kind::String_Lit(_) | Token_Kind::Number(_) | Token_Kind::Char_Lit(_))
                                { false }
                                else if PrevT.Text == ";" || PrevT.Text == "," || PrevT.Text == "=" || PrevT.Text == ":"
                                { false }
                                else if Self::IsPrecededByAssignment(Nodes, Index - 1)
                                { false }
                                else if matches!(C.Kind, Container_Kind::Paren | Container_Kind::Bracket) && matches!(PrevT.Kind, Token_Kind::Identifier(_))
                                { ! Self::IsControlKeyword( & PrevT.Text) }
                                else
                                { false }
                            }
                            Syntax_Node::Container(_) => false,
                        }
                    }
                    else
                    { false };

                    let IsPrecededByEquals = Index > 0 && matches!( & Nodes[Index - 1], Syntax_Node::Token(T) if T.Text == "=");

                    if C.Open_Token.Preceding_Newlines > 0 && ! ShouldAttachInline && ! IsPrecededByEquals
                    {
                        self.EnsureNewline();
                        if C.Open_Token.Preceding_Newlines >= 2 { self.EnsureBlankLine(); }
                    }

                    let StartedAtLineStart = ! self.Line_Has_Content;
                    let EffectiveCol = if self.Line_Has_Content
                    { self.Current_Line_Len }
                    else if C.Open_Token.Preceding_Newlines > 0 && C.Statement_Count > 0 && self.Prev_Line_Len > Level * self.Config.Indent_Width
                    { self.Prev_Line_Len }
                    else
                    { Level * self.Config.Indent_Width };
                    let Layout = self.Planner.DecideLayout(C, Level, EffectiveCol);
                    self.EmitContainerNodeWithLayout(C, Level, Layout, IsPrecededByEquals);
                    // Check if followed by continuation keyword (else, catch, finally, while)
                    let NextContinuation = if Index + 1 < Total
                    {
                        match & Nodes[Index + 1] { Syntax_Node::Token(T) if matches ! (T.Text.as_str(), "else" | "catch" | "finally" | "while") => { Some(T.Text.clone()) } _ => None, }
                    }
                    else { None };

                    if let Some(ContKw) = NextContinuation
                    {
                        if Layout == Layout_State::Monolith
                        {
                            // Check if the rest of the clause fits on the same line
                            let CanInline = if Index + 2 < Total
                            {
                                match & Nodes[Index + 2]
                                {
                                    Syntax_Node::Container(NextC) =>
                                    {
                                        if ! NextC.Has_Forced_Break
                                        {
                                            let Span = NextC.CalculateSingleLineSpan();
                                            Span != usize::MAX && self.Current_Line_Len + 1 + ContKw.len() + 1 + Span <= self.Config.Canvas_Width
                                        }
                                        else { false }
                                    }
                                    _ => false,
                                }
                            }
                            else { false };

                            if CanInline { self.EmitSpace(); } else { self.EnsureNewline(); }
                        }
                        else { self.EnsureNewline(); }
                    }
                    else if C.Tail_Glue.iter().any( | G | G.Text == ";") || C.Kind == Container_Kind::Brace || (C.Kind == Container_Kind::Angle && IsTemplate)
                    || self.Config.Language.IsToml()
                    {
                        let IsTrailingPunct = Index + 1 < Total && matches!( & Nodes[Index + 1], Syntax_Node::Token(T) if T.Text == ";" || T.Text == ",");
                        if C.Kind == Container_Kind::Brace && ! IsTrailingPunct && Index + 1 < Total && ! self.Config.Language.IsToml()
                        {
                            if Self::IsDeclaration(Nodes, Index) && (Level == 0 || Level == 1) { self.EnsureBlankLine(); }
                            else if Level == 0 && Self::IsImportStatementEnding(Nodes, Index) && Self::HasMoreNonImportStatements(Nodes, Index + 1)
                            {
                                self.EnsureBlankLine();
                            }
                            else { self.EnsureNewline(); }
                        }
                        else if ! IsTrailingPunct
                        { self.EnsureNewline(); }
                    }
                    else if StartedAtLineStart
                    {
                        let IsTrailingPunct = Index + 1 < Total && matches!( & Nodes[Index + 1], Syntax_Node::Token(T) if T.Text == ";" || T.Text == ",");
                        if ! IsTrailingPunct
                        { self.EnsureNewline(); }
                    }
                }
            }

            Index += 1;
        }
    }

    fn IsControlKeyword(Text: & str) -> bool
    { matches ! (Text, "return" | "throw" | "break" | "continue" | "yield" | "await" | "assert" | "case" | "default" | "in" | "is" | "as") }

    fn IsPrecededByAssignment(Nodes: & [Syntax_Node], FromIdx: usize) -> bool
    {
        let mut ScanIdx = FromIdx;
        loop
        {
            match & Nodes[ScanIdx]
            {
                Syntax_Node::Token(T) =>
                { if T.Text == "=" { return true; } if T.Text == ";" || T.Text == "{" || T.Text == "}" || T.Preceding_Newlines > 0 { break ; } }
                Syntax_Node::Container(C) =>
                { if C.Kind == Container_Kind::Brace || C.Open_Token.Preceding_Newlines > 0 { break ; } }
            }
            if ScanIdx == 0 { break ; }
            ScanIdx -= 1;
        }
        false
    }

    fn IsImportStatement(Nodes: & [Syntax_Node], mut StartIdx: usize) -> bool
    {
        if StartIdx >= Nodes.len() { return false; }
        if let Syntax_Node::Token(T) = & Nodes[StartIdx]
        {
            if T.Text == "pub"
            {
                StartIdx += 1;
                if StartIdx < Nodes.len()
                {
                    if let Syntax_Node::Container(C) = & Nodes[StartIdx] { if C.Kind == Container_Kind::Paren { StartIdx += 1; } }
                }
                if StartIdx >= Nodes.len() { return false; }
            }
        }

        match & Nodes[StartIdx]
        {
            Syntax_Node::Token(T) =>
            {
                if let Token_Kind::Preprocessor(ref Text) = T.Kind { return Text.starts_with("#include"); }
                if let Token_Kind::Keyword(ref K) = T.Kind
                {
                    if matches!(K.as_str(), "use" | "using" | "import") { return true; }
                    if K == "extern" && StartIdx + 1 < Nodes.len() { if let Syntax_Node::Token(NextT) = & Nodes[StartIdx + 1] { return NextT.Text == "crate"; } }
                }
                false
            }
            _ => false,
        }
    }

    fn IsImportStatementEnding(Nodes: & [Syntax_Node], CurrentIdx: usize) -> bool
    {
        if CurrentIdx >= Nodes.len() { return false; }
        match & Nodes[CurrentIdx]
        {
            Syntax_Node::Token(T) => { if let Token_Kind::Preprocessor(ref Text) = T.Kind { return Text.starts_with("#include"); } if T.Text != ";" { return false; } }
            Syntax_Node::Container(C) => { if ! C.Tail_Glue.iter().any( | G | G.Text == ";") { return false; } }
        }

        let mut StartIdx = CurrentIdx;
        while StartIdx > 0
        {
            let PrevIdx = StartIdx - 1;
            match & Nodes[PrevIdx]
            {
                Syntax_Node::Token(T) =>
                {
                    if T.Text == ";" || matches!(T.Kind, Token_Kind::Preprocessor(_) | Token_Kind::Comment_Line(_) | Token_Kind::Comment_Block(_)) { break ; }
                    StartIdx = PrevIdx;
                }
                Syntax_Node::Container(C) => { if C.Kind == Container_Kind::Brace { break ; } StartIdx = PrevIdx; }
            }
        }

        Self::IsImportStatement(Nodes, StartIdx)
    }

    fn HasMoreNonImportStatements(Nodes: & [Syntax_Node], FromIdx: usize) -> bool
    {
        let mut NextIdx = FromIdx;
        while NextIdx < Nodes.len()
        {
            match & Nodes[NextIdx]
            {
                Syntax_Node::Token(T) =>
                {
                    if matches!(T.Kind, Token_Kind::Comment_Line(_) | Token_Kind::Comment_Block(_)) { NextIdx += 1; continue ; }
                    return ! Self::IsImportStatement(Nodes, NextIdx);
                }
                Syntax_Node::Container(_) => { return true; }
            }
        }
        false
    }

    fn IsDeclaration(Nodes: & [Syntax_Node], ContainerIdx: usize) -> bool
    {
        if ContainerIdx == 0 { return false; }
        let mut ScanIdx = ContainerIdx;
        let mut HasParen = false;
        let mut HasEquals = false;

        while ScanIdx > 0
        {
            ScanIdx -= 1;
            match & Nodes[ScanIdx]
            {
                Syntax_Node::Token(T) =>
                {
                    if T.Text == ";" || T.Text == "{" || T.Text == "}" { break ; }
                    if T.Text == "=" { HasEquals = true; }
                    if let Token_Kind::Keyword(ref K) = T.Kind
                    {
                        if matches!(K.as_str(), "if" | "else" | "while" | "for" | "loop" | "match" | "switch" | "catch" | "finally" | "do" | "try" | "with" | "synchronized" | "lock")
                        {
                            return false;
                        }

                        if matches!(K.as_str(), "use" | "using" | "import") { return false; }

                        if matches!(K.as_str(), "fn" | "func" | "function" | "def" | "struct" | "class" | "enum" | "trait" | "impl" | "interface" | "mod" | "namespace")
                        {
                            return true;
                        }
                    }
                }
                Syntax_Node::Container(C) => { if C.Kind == Container_Kind::Brace { break ; } if C.Kind == Container_Kind::Paren { HasParen = true; } }
            }
        }

        HasParen && ! HasEquals
    }

    fn EmitSingleToken( & mut self, Token: & Token_Info, Level: usize)
    {
        // 1. Preserve intentional blank line or line break if preceded by newlines in source
        if Token.Preceding_Newlines > 0 { self.EnsureNewline(); if Token.Preceding_Newlines >= 2 { self.EnsureBlankLine(); } }

        // 2. Preprocessor directives must always sit on their own line
        if let Token_Kind::Preprocessor(DirectiveText) = & Token.Kind
        {
            self.EnsureNewline();
            let FormattedDirective = Self::FormatPreprocessorDirective(DirectiveText, self.Config.Language);
            self.EmitRaw( & FormattedDirective);
            self.WriteNewline();
            return ;
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

        // 5. If line is empty, apply indentation
        if ! self.Line_Has_Content { self.EmitIndent(Level); }

        // 6. Formatting token text and spacing
        match & Token.Kind
        {
            Token_Kind::Punctuation(';') =>
            {
                self.EmitRaw(";");
                // In C-family, statements generally end with newline unless inside a for-header
                self.WriteNewline();
            }
            Token_Kind::Punctuation(',') => { self.EmitRaw(","); self.EmitSpace(); }
            Token_Kind::Punctuation(':') =>
            {
                if self.Output.ends_with("public ") || self.Output.ends_with("private ") || self.Output.ends_with("protected ") || self.Output.ends_with("default ")
                {
                    self.Output.pop();
                    self.Current_Line_Len -= 1;
                    self.EmitRaw(":");
                    self.WriteNewline();
                }
                else { self.EmitRaw(":"); self.EmitSpace(); }
            }
            Token_Kind::Punctuation('.') =>
            {
                if self.Output.ends_with(' ')
                {
                    let Trimmed = self.Output.trim_end();
                    if ! Trimmed.ends_with( | c: char | c.is_ascii_digit()) && ! Trimmed.ends_with('.')
                    {
                        self.Output.pop();
                        self.Current_Line_Len -= 1;
                    }
                }
                else if self.Output.ends_with('.')
                { self.EmitSpace(); }
                else if self.Config.Language == Source_Language::Rust && self.Output.ends_with( | c: char | c.is_ascii_digit())
                { self.EmitSpace(); }
                self.EmitRaw(".");
            }
            Token_Kind::Operator(Op) =>
            {
                let IsArrowReturnType = Op == "->" &&
                (
                    self.Config.Language == Source_Language::Rust
                    || self.Output.ends_with(')')
                    || self.Output.ends_with(']')
                    || self.Output.ends_with('|')
                    || self.Output.ends_with('>')
                );

                if IsArrowReturnType
                {
                    if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                    { self.EmitSpace(); }
                    self.EmitRaw("->");
                    self.EmitSpace();
                }
                else if Op == "::" || Op == "->" || Op == "?."
                {
                    if self.Output.ends_with(' ') && ! self.Output.ends_with("-> ") && ! self.Output.ends_with("> ") && ! self.Output.ends_with(": ")
                    && ! self.Output.ends_with("return ") && ! self.Output.ends_with("using ") && ! self.Output.ends_with("case ")
                    && ! self.Output.ends_with("new ") && ! self.Output.ends_with("delete ")
                    { self.Output.pop(); self.Current_Line_Len -= 1; }
                    self.EmitRaw(Op);
                }
                else if Op == "++" || Op == "--" || Op == "!" || Op == "~" || Op == ".." || Op == "..=" || Op == "..." || Op == "#"
                {
                    if (Op == ".." || Op == "..=" || Op == "..." || Op == "#") && self.Output.ends_with(' ')
                    {
                        let Trimmed = self.Output.trim_end();
                        if ! Trimmed.ends_with('.')
                        { self.Output.pop(); self.Current_Line_Len -= 1; }
                    }
                    if (Op == ".." || Op == "..=" || Op == "...") && self.Output.ends_with('.')
                    { self.EmitSpace(); }
                    self.EmitRaw(Op);
                }
                else { self.EmitSpace(); self.EmitRaw(Op); self.EmitSpace(); }
            }
            Token_Kind::Keyword(K) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n') { self.EmitSpace(); }
                self.EmitRaw(K);
                if matches!
                (
                    K.as_str(), "if" | "while" | "for" | "switch" | "catch" | "return" | "class" | "struct" | "enum" | "namespace" | "using" | "new" | "case" | "throw" | "public" | "private" | "protected" | "static" | "virtual" | "override" | "const" | "impl" | "fn" | "trait" | "match" | "loop" | "mod" | "let" | "pub" | "mut" | "use" | "extern" | "in"
                )
                { self.EmitSpace(); }
            }
            Token_Kind::Identifier(Ident) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n') && ! self.Output.ends_with('.')
                && ! self.Output.ends_with("->") && ! self.Output.ends_with("::") { self.EmitSpace(); }
                self.EmitRaw(Ident);
            }
            Token_Kind::Number(Num) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n') && ! self.Output.ends_with('.')
                && ! self.Output.ends_with("->") && ! self.Output.ends_with("::") && ! self.Output.ends_with("-")
                && ! self.Output.ends_with("..=") && ! self.Output.ends_with("..") { self.EmitSpace(); }
                self.EmitRaw(Num);
            }
            Token_Kind::String_Lit(Text) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n') && ! self.Output.ends_with('.')
                && ! self.Output.ends_with("->") && ! self.Output.ends_with("::") && ! self.Output.ends_with('(') && ! self.Output.ends_with('[')
                {
                    self.EmitSpace();
                }
                let Realigned = Self::RealignMultilineString(Text, self.Current_Line_Len);
                self.EmitRaw( & Realigned);
            }
            Token_Kind::Char_Lit(Text) =>
            {
                if self.Line_Has_Content && self.Current_Line_Len > 0 && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n') && ! self.Output.ends_with('.')
                && ! self.Output.ends_with("->") && ! self.Output.ends_with("::") && ! self.Output.ends_with('(') && ! self.Output.ends_with('[')
                {
                    self.EmitSpace();
                }
                self.EmitRaw(Text);
            }
            _ => { self.EmitRaw( & Token.Text); }
        }
    }

    fn EmitContainerNode( & mut self, Container: & Container_Node, Level: usize)
    {
        let Layout = self.Planner.DecideLayout(Container, Level, self.Current_Line_Len);
        self.EmitContainerNodeWithLayout(Container, Level, Layout, false);
    }

    fn EmitContainerNodeWithLayout( & mut self, Container: & Container_Node, Level: usize, Layout: Layout_State, IsPrecededByEquals: bool ,)
    {
        if self.Config.Language.IsCargoLock() && (Container.Kind == Container_Kind::Bracket || Container.Kind == Container_Kind::Brace) && IsPrecededByEquals
        {
            let HasNonTrivia = Container.Children.iter().any( | C | match C { Syntax_Node::Token(T) => ! T.IsTrivia(), Syntax_Node::Container(_) => true, });
            if HasNonTrivia
            { self.EmitCargoLockColumnAnchoredHybrid(Container, Level); }
            else
            { self.EmitMonolith(Container, Level); }
            return ;
        }

        match Layout
        {
            Layout_State::Monolith => { self.EmitMonolith(Container, Level); }
            Layout_State::Grouped => { self.EmitGrouped(Container, Level); }
            Layout_State::Expanded => { self.EmitExpanded(Container, Level); }
        }
    }

    fn EmitCargoLockColumnAnchoredHybrid( & mut self, Container: & Container_Node, _Level: usize)
    {
        if ! self.Output.ends_with(' ') { self.EmitSpace(); }
        let OpenCol = self.Current_Line_Len;
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();

        let ChildIndent = " ".repeat(OpenCol + self.Config.Indent_Width);
        for Child in & Container.Children
        {
            match Child
            {
                Syntax_Node::Token(T) =>
                {
                    if T.Text == ","
                    { if self.Line_Has_Content && ! self.Output.ends_with(',') { self.EmitRaw(","); } self.WriteNewline(); }
                    else if matches!(T.Kind, Token_Kind::Comment_Line(_))
                    {
                        self.EnsureNewline();
                        self.EmitRaw( & ChildIndent);
                        self.EmitRaw( & T.Text);
                        self.WriteNewline();
                    }
                    else if ! T.IsTrivia()
                    { self.EnsureNewline(); self.EmitRaw( & ChildIndent); self.EmitRaw( & T.Text); }
                }
                Syntax_Node::Container(SubC) =>
                { self.EnsureNewline(); self.EmitRaw( & ChildIndent); self.EmitMonolith(SubC, 0); }
            }
        }

        if self.Line_Has_Content
        { if ! self.Output.ends_with(',') { self.EmitRaw(","); } self.WriteNewline(); }

        let CloseIndent = " ".repeat(OpenCol);
        self.EmitRaw( & CloseIndent);
        self.EmitRaw(Container.Kind.CloseGlyph());
        for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
        self.WriteNewline();
    }

    fn EmitMonolith( & mut self, Container: & Container_Node, Level: usize)
    {
        if ! self.Line_Has_Content { self.EmitIndent(Level); }
        // For Monolith: openers and closers remain horizontal on the same line
        let IsAtLineStart = self.Output.is_empty() || self.Output.trim_end_matches([' ', '\t']).ends_with('\n');
        let NeedSpace = ! IsAtLineStart && ! self.Output.ends_with(' ') && ! self.Output.ends_with('[')
        && (Container.Kind == Container_Kind::Brace || (Container.Kind == Container_Kind::Bracket && self.Output.ends_with('=')));
        if NeedSpace { self.EmitSpace(); }

        self.EmitRaw(Container.Kind.OpenGlyph());

        if Container.Kind == Container_Kind::Brace && ! Container.Children.is_empty() { self.EmitRaw(" "); }

        for Child in & Container.Children
        {
            match Child
            {
                Syntax_Node::Token(T) =>
                {
                    if T.Text == ";" { self.EmitRaw(";"); self.EmitSpace(); }
                    else if T.Text == "," { self.EmitRaw(","); self.EmitSpace(); }
                    else if T.Text == "."
                    {
                        if self.Output.ends_with(' ')
                        {
                            let Trimmed = self.Output.trim_end();
                            if ! Trimmed.ends_with( | c: char | c.is_ascii_digit()) && ! Trimmed.ends_with('.')
                            {
                                self.Output.pop();
                                self.Current_Line_Len -= 1;
                            }
                        }
                        else if self.Output.ends_with('.')
                        { self.EmitSpace(); }
                        else if self.Config.Language == Source_Language::Rust && self.Output.ends_with( | c: char | c.is_ascii_digit())
                        { self.EmitSpace(); }
                        self.EmitRaw(".");
                    }
                    else if T.Text == ":"
                    {
                        if self.Output.ends_with("public ") || self.Output.ends_with("private ") || self.Output.ends_with("protected ") || self.Output.ends_with("default ")
                        {
                            self.Output.pop();
                            self.Current_Line_Len -= 1;
                            self.EmitRaw(":");
                            self.EmitSpace();
                        }
                        else { self.EmitRaw(":"); self.EmitSpace(); }
                    }
                    else if T.Text == "::" || T.Text == "->" || T.Text == "?."
                    {
                        let IsArrowReturnType = T.Text == "->" &&
                        (
                            self.Config.Language == Source_Language::Rust
                            || self.Output.ends_with(')')
                            || self.Output.ends_with(']')
                            || self.Output.ends_with('|')
                            || self.Output.ends_with('>')
                        );

                        if IsArrowReturnType
                        {
                            if self.Line_Has_Content && ! self.Output.ends_with(' ') && ! self.Output.ends_with('\n')
                            { self.EmitSpace(); }
                            self.EmitRaw("->");
                            self.EmitSpace();
                        }
                        else
                        {
                            if self.Output.ends_with(' ') && ! self.Output.ends_with("-> ") && ! self.Output.ends_with("> ") && ! self.Output.ends_with(": ")
                            && ! self.Output.ends_with("return ") && ! self.Output.ends_with("using ") && ! self.Output.ends_with("case ")
                            && ! self.Output.ends_with("new ") && ! self.Output.ends_with("delete ")
                            { self.Output.pop(); self.Current_Line_Len -= 1; }
                            self.EmitRaw( & T.Text);
                        }
                    }
                    else if let Token_Kind::Operator(ref Op) = T.Kind
                    {
                        if Op == ".." || Op == "..=" || Op == "..." || Op == "#"
                        {
                            if self.Output.ends_with(' ')
                            {
                                let Trimmed = self.Output.trim_end();
                                if ! Trimmed.ends_with('.')
                                { self.Output.pop(); self.Current_Line_Len -= 1; }
                            }
                            if (Op == ".." || Op == "..=" || Op == "...") && self.Output.ends_with('.')
                            { self.EmitSpace(); }
                            self.EmitRaw( & T.Text);
                        }
                        else { self.EmitSpace(); self.EmitRaw( & T.Text); self.EmitSpace(); }
                    }
                    else
                    {
                        let NeedSpace = if self.Output.ends_with('(') || self.Output.ends_with('[') || self.Output.ends_with('<') || self.Output.ends_with('{') || self.Output.ends_with(' ')
                        || self.Output.ends_with("::") || self.Output.ends_with("->") || self.Output.ends_with('.')
                        || self.Output.ends_with("..=") || self.Output.ends_with("..") { false }
                        else
                        {
                            matches!
                            (
                                T.Kind, Token_Kind::Keyword(_) | Token_Kind::Identifier(_) | Token_Kind::Number(_) | Token_Kind::String_Lit(_) | Token_Kind::Char_Lit(_)
                            )
                        };

                        if NeedSpace { self.EmitSpace(); }
                        self.EmitRaw( & T.Text);
                        if matches!(T.Kind, Token_Kind::Keyword(_)) { self.EmitSpace(); }
                    }
                }
                Syntax_Node::Container(SubC) => { self.EmitMonolith(SubC, Level); }
            }
        }

        if Container.Kind == Container_Kind::Brace && ! Container.Children.is_empty() && ! self.Output.ends_with(' ') { self.EmitRaw(" "); }

        if Container.Close_Token.is_some()
        {
            if (Container.Kind == Container_Kind::Paren || Container.Kind == Container_Kind::Bracket || Container.Kind == Container_Kind::Angle) && self.Output.ends_with(' ')
            {
                self.Output.pop();
                self.Current_Line_Len -= 1;
            }

            self.EmitRaw(Container.Kind.CloseGlyph());

            for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
        }
    }

    fn EmitGrouped( & mut self, Container: & Container_Node, Level: usize)
    {
        let Chain = Container.CollectChain();
        let OpenRun: String = Chain.iter().map( | C | C.Kind.OpenGlyph()).collect();
        let CloseRun: String = Chain.iter().rev().filter( | C | C.Close_Token.is_some()).map( | C | C.Kind.CloseGlyph()).collect();
        let Innermost = Chain.last().unwrap();
        // 1. Head stays on current line, then break line
        self.EnsureNewline();
        // 2. Compound opening run sits on its own line at Level indent (Load-bearing pillar)
        self.EmitIndent(Level);
        self.EmitRaw( & OpenRun);
        self.WriteNewline();
        // 3. Innermost children indented at Level + 1
        self.EmitNodeList( & Innermost.Children, Level + 1);
        if ! CloseRun.is_empty()
        {
            self.EnsureNewline();
            // Remove trailing empty line right before compound closer
            if self.Output.ends_with("\n\n") { self.Output.pop(); }
            // 4. Compound closing run sits on its own line at Level indent (Load-bearing pillar)
            self.EmitIndent(Level);
            self.EmitRaw( & CloseRun);
            // 5. Tail glue from outermost container sits right after close run
            for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
            self.WriteNewline();
        }
    }

    fn EmitExpanded( & mut self, Container: & Container_Node, Level: usize)
    {
        // 1. Head stays on current line, then break line (Anti-Hugging Law)
        self.EnsureNewline();
        // 2. Opening delimiter sits on its own line at Level indent (Load-bearing pillar)
        self.EmitIndent(Level);
        self.EmitRaw(Container.Kind.OpenGlyph());
        self.WriteNewline();
        // 3. Inner children indented at Level + 1
        self.EmitNodeList( & Container.Children, Level + 1);
        if Container.Close_Token.is_some()
        {
            self.EnsureNewline();
            // Remove trailing empty line right before closer
            if self.Output.ends_with("\n\n") { self.Output.pop(); }
            // 4. Closing delimiter sits on its own line at Level indent (Translational symmetry)
            self.EmitIndent(Level);
            self.EmitRaw(Container.Kind.CloseGlyph());
            // 5. Tail glue (';', ',') sits right after closing delimiter
            for Glue in & Container.Tail_Glue { self.EmitRaw( & Glue.Text); }
            // In C/C++, a closing brace of a function, struct, or block is followed by a newline
            self.WriteNewline();
        }
    }

    fn RealignMultilineString(Text: & str, TargetIndent: usize) -> String
    {
        if ! Text.contains('\n') { return Text.to_string(); }

        let Newline = if Text.contains("\r\n") { "\r\n" } else { "\n" };
        let RawLines: Vec < & str > = Text.split('\n').collect();
        if RawLines.len() < 2 { return Text.to_string(); }

        let Line0 = RawLines[0].trim_end_matches('\r');
        let Line0Trimmed = Line0 .trim();
        let LastLine = RawLines[RawLines.len() - 1].trim_end_matches('\r');
        let LastTrimmed = LastLine.trim();

        let (IsOpenDelimOnly, ExpectedClose) = if Line0Trimmed.starts_with("r#") || Line0Trimmed.starts_with("br#") || Line0Trimmed.starts_with("cr#")
        {
            let HashCount = Line0Trimmed.chars().filter( | c | * c == '#').count();
            let Close = format!("\"{}", "#".repeat(HashCount));
            (Line0Trimmed.ends_with('"'), Close)
        }
        else if Line0Trimmed == "r\"" || Line0Trimmed == "br\"" || Line0Trimmed == "cr\""
        { (true, "\"".to_string()) }
        else if Line0Trimmed.starts_with("\"\"\"")
        {
            let QuoteCount = Line0Trimmed.chars().take_while( | c | * c == '"').count();
            (Line0Trimmed.len() == QuoteCount, "\"".repeat(QuoteCount))
        }
        else if Line0Trimmed == "\""
        { (true, "\"".to_string()) }
        else if Line0Trimmed == "`"
        { (true, "`".to_string()) }
        else
        { (false, String::new ()) };

        if ! IsOpenDelimOnly || ExpectedClose.is_empty() || ! LastTrimmed.ends_with( & ExpectedClose)
        { return Text.to_string(); }

        let OrigClosingIndent = LastLine.len() - LastLine.trim_start().len();
        if OrigClosingIndent == 0 { return Text.to_string(); }

        let Shift: isize = TargetIndent as isize - OrigClosingIndent as isize;
        if Shift == 0 { return Text.to_string(); }

        let mut Result = String::with_capacity(Text.len() + 64);
        Result.push_str(Line0);

        for (Idx, RawLine) in RawLines[1..].iter().enumerate()
        {
            let LineIdx = Idx + 1;
            let Clean = RawLine.trim_end_matches('\r');
            Result.push_str(Newline);

            if Clean.trim().is_empty() { continue ; }

            if LineIdx == RawLines.len() - 1
            { Result.push_str( & " ".repeat(TargetIndent)); Result.push_str(Clean.trim_start()); }
            else
            {
                let Leading = Clean.len() - Clean.trim_start().len();
                let NewLeading = (Leading as isize + Shift).max(0) as usize;
                Result.push_str( & " ".repeat(NewLeading));
                Result.push_str(Clean.trim_start());
            }
        }

        Result
    }

    pub fn FormatPreprocessorDirective(Directive: & str, Language: Source_Language) -> String
    {
        if ! Language.SupportsPreprocessor()
        || ! Directive.contains('\n')
        || ! Directive.contains('\\')
        || ( ! Directive.contains('{') && ! Directive.contains('}'))
        { return Directive.to_string(); }

        let Newline = if Directive.contains("\r\n") { "\r\n" } else { "\n" };
        let RawLines: Vec < & str > = Directive.split('\n').collect();

        // 1. Strip trailing backslashes and whitespace, record original indent
        struct MacroRawEntry
        { Text: String, Original_Indent: usize, }

        let mut Entries: Vec < MacroRawEntry > = Vec:: new ();

        for RawLine in RawLines
        {
            let Trimmed = RawLine.trim_end_matches('\r').trim_end();
            let HasContinuation = Trimmed.ends_with('\\');
            let Content = if HasContinuation
            { Trimmed[..Trimmed.len() - 1].trim_end() }
            else
            { Trimmed };

            if Content.trim().is_empty()
            { continue ; }

            let OrigIndent = Content.chars().take_while( | c | * c == ' ').count()
            + Content.chars().take_while( | c | * c == '\t').count() * 4;

            let SplitParts = Self::SplitMacroLine(Content);
            for (Idx, Part) in SplitParts.into_iter().enumerate()
            {
                let PartIndent = if Idx == 0 { OrigIndent } else { OrigIndent };
                Entries.push(MacroRawEntry { Text: Part, Original_Indent: PartIndent, });
            }
        }

        if Entries.is_empty()
        { return Directive.to_string(); }

        // 2. Indent according to Allman block nesting
        let mut ResultLines: Vec < String > = Vec:: new ();
        let mut Indent_Stack: Vec < usize > = Vec:: new ();
        let mut Last_Statement_Indent: usize = 0;
        let mut Unbraced_Body = false;

        for Entry in & Entries
        {
            let Trimmed = Entry.Text.trim();
            if Trimmed.is_empty()
            { ResultLines.push(String::new ()); continue ; }

            if Trimmed.starts_with('}')
            {
                Unbraced_Body = false;
                let Close_Indent = if ! Indent_Stack.is_empty()
                { Indent_Stack.pop().unwrap() }
                else
                { 0 };
                ResultLines.push(format ! ("{}{}", " ".repeat(Close_Indent), Trimmed));
                Last_Statement_Indent = Close_Indent;
                continue;
            }

            if Trimmed == "{" || Trimmed.starts_with("{ ") || Trimmed.starts_with("{\t")
            {
                Unbraced_Body = false;
                let Brace_Indent = if Indent_Stack.is_empty()
                { 0 }
                else
                { Last_Statement_Indent };
                Indent_Stack.push(Brace_Indent);
                ResultLines.push(format ! ("{}{}", " ".repeat(Brace_Indent), Trimmed));
                continue;
            }

            if Indent_Stack.is_empty()
            { ResultLines.push(Trimmed.to_string()); Last_Statement_Indent = 0; Unbraced_Body = false; }
            else
            {
                let Base_Indent = Indent_Stack.last().unwrap() + 4;
                let Actual_Indent = if Unbraced_Body
                { Base_Indent + 4 }
                else
                { Base_Indent };
                Last_Statement_Indent = Actual_Indent;
                ResultLines.push(format ! ("{}{}", " ".repeat(Actual_Indent), Trimmed));

                // Check if this statement is an unbraced control header (if, else, while, for)
                let IsControlHeader = Trimmed.starts_with("if ") || Trimmed.starts_with("if(")
                || Trimmed.starts_with("else ") || Trimmed == "else"
                || Trimmed.starts_with("while ") || Trimmed.starts_with("while(")
                || Trimmed.starts_with("for ") || Trimmed.starts_with("for(");

                if IsControlHeader && ! Trimmed.ends_with(';') && ! Trimmed.ends_with('{')
                { Unbraced_Body = true; }
                else if Trimmed.ends_with(';')
                { Unbraced_Body = false; }
            }
        }

        while ResultLines.len() > 1 && ResultLines.last().map_or(false, | L | L.trim().is_empty())
        { ResultLines.pop(); }

        // 3. Append continuation backslashes
        let Total = ResultLines.len();
        let mut Formatted = String:: new ();
        for (Idx, Line) in ResultLines.iter().enumerate()
        {
            if Idx > 0
            { Formatted.push_str(Newline); }
            let LineTrimmed = Line.trim_end();
            if Idx + 1 < Total
            { Formatted.push_str(LineTrimmed); Formatted.push_str(" \\"); }
            else
            { Formatted.push_str(LineTrimmed); }
        }

        Formatted
    }

    fn SplitMacroLine(Line: & str) -> Vec<String>
    {
        let Trimmed = Line.trim();
        if Trimmed == "{" || Trimmed == "}" || Trimmed.starts_with("} while")
        { return vec ! [Line.to_string()]; }

        if let Some(BraceIdx) = Self::FindUnquotedBrace(Line)
        {
            let Before = Line[..BraceIdx].trim_end();
            let After = Line[BraceIdx + 1..].trim();

            if Before.is_empty()
            { return vec ! [Line.to_string()]; }

            let BeforeTrim = Before.trim_start();
            if BeforeTrim.starts_with("} else")
            {
                let ElsePart = BeforeTrim[1..].trim_start();
                let mut Parts = vec!["}".to_string(), ElsePart.to_string()];
                if After.is_empty()
                { Parts.push("{".to_string()); }
                else
                { Parts.push(format ! ("{{ {}", After)); }
                return Parts;
            }

            let mut Parts = vec![Before.to_string()];
            if After.is_empty()
            { Parts.push("{".to_string()); }
            else
            { Parts.push(format ! ("{{ {}", After)); }
            return Parts;
        }

        vec![Line.to_string()]
    }

    fn FindUnquotedBrace(Line: & str) -> Option<usize>
    {
        let Chars: Vec < char > = Line.chars().collect();
        let mut InString = false;
        let mut InChar = false;
        let mut InComment = false;
        let mut I = 0;
        let Len = Chars.len();

        while I < Len
        {
            let Ch = Chars[I];

            if InString
            { if Ch == '\\' && I + 1 < Len { I += 2; continue ; } if Ch == '"' { InString = false; } I += 1; continue ; }

            if InChar
            { if Ch == '\\' && I + 1 < Len { I += 2; continue ; } if Ch == '\'' { InChar = false; } I += 1; continue ; }

            if InComment
            { if Ch == '*' && I + 1 < Len && Chars[I + 1] == '/' { InComment = false; I += 2; continue ; } I += 1; continue ; }

            if Ch == '"' { InString = true; I += 1; continue ; }
            if Ch == '\'' { InChar = true; I += 1; continue ; }
            if Ch == '/' && I + 1 < Len && Chars[I + 1] == '*' { InComment = true; I += 2; continue ; }
            if Ch == '/' && I + 1 < Len && Chars[I + 1] == '/' { break ; }

            if Ch == '{'
            {
                let mut AfterIsWhitespaceOrComment = true;
                let mut J = I + 1;
                while J < Len
                {
                    if Chars[J] == '/' && J + 1 < Len && (Chars[J + 1] == '/' || Chars[J + 1] == '*')
                    { break ; }
                    if ! Chars[J].is_whitespace()
                    { AfterIsWhitespaceOrComment = false; break ; }
                    J += 1;
                }
                if AfterIsWhitespaceOrComment
                { return Some(I); }
            }

            I += 1;
        }

        None
    }
}

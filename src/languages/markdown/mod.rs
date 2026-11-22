#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

use crate::formatter::Source_Formatter;
use crate::types:: { Config_Settings, Kotlin_Style, Source_Language };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Markdown_Line_Type
{ Blank, Structural, ListItemStart, Blockquote, Text, }

pub struct Markdown_Formatter;

impl Markdown_Formatter
{
    pub fn Format(Source: & str, Config: & Config_Settings) -> Result<String, String>
    {
        let HasCRLF = Source.contains("\r\n");
        let LineEnding = if HasCRLF { "\r\n" } else { "\n" };
        let EndsWithNewline = Source.ends_with('\n');

        let RawLines: Vec< & str> = Source
       .split('\n')
       .map( | L | L.strip_suffix('\r').unwrap_or(L))
       .collect();

        let mut OutLines: Vec<String> = Vec::with_capacity(RawLines.len());
        let mut NonFenceBuffer: Vec< & str> = Vec:: new ();
        let mut I = 0;
        let TotalLines = RawLines.len();

        let FlushNonFence = | Buffer: & mut Vec< & str>, Out: & mut Vec<String>, Cfg: & Config_Settings |
        {
            if Buffer.is_empty()
            { return ; }
            if Cfg.Unwrap_Markdown_Paragraphs
            { let Unwrapped = Self::UnwrapParagraphs(Buffer); for Line in Unwrapped { Out.push(Line); } }
            else
            { for & Line in Buffer.iter() { Out.push(Line.to_string()); } }
            Buffer.clear();
        };

        while I < TotalLines
        {
            if I == TotalLines - 1 && EndsWithNewline && RawLines[I].is_empty()
            {
                FlushNonFence( & mut NonFenceBuffer, & mut OutLines, Config);
                OutLines.push(String::new ());
                break;
            }

            let Line = RawLines[I];

            if let Some((OpenPrefix, FenceChar, FenceLen, InfoString)) = Self::ParseOpeningFence(Line)
            {
                FlushNonFence( & mut NonFenceBuffer, & mut OutLines, Config);

                let OpeningLine = Line.to_string();
                let LangOpt = Self::LanguageFromInfoString(InfoString, Config);
                let ContentPrefix = Self::ComputeContentPrefix(OpenPrefix);
                let mut SnippetLines: Vec<String> = Vec:: new ();
                let mut FoundClose = false;
                let mut ClosingLine = String:: new ();

                I += 1;
                while I < TotalLines
                {
                    if I == TotalLines - 1 && EndsWithNewline && RawLines[I].is_empty()
                    { break ; }
                    let InnerLine = RawLines[I];
                    if Self::IsClosingFence(InnerLine, FenceChar, FenceLen, OpenPrefix)
                    {
                        FoundClose = true;
                        ClosingLine = InnerLine.to_string();
                        I += 1;
                        break;
                    }
                    else
                    { SnippetLines.push(InnerLine.to_string()); I += 1; }
                }

                if FoundClose
                {
                    OutLines.push(OpeningLine);
                    if let Some(Lang) = LangOpt
                    {
                        if let Some(FormattedSnippet) = Self::FormatSnippet( & SnippetLines, Lang, OpenPrefix, & ContentPrefix, Config)
                        { for SLine in FormattedSnippet { OutLines.push(SLine); } }
                        else
                        { for SLine in SnippetLines { OutLines.push(SLine); } }
                    }
                    else
                    { for SLine in SnippetLines { OutLines.push(SLine); } }
                    OutLines.push(ClosingLine);
                }
                else
                { OutLines.push(OpeningLine); for SLine in SnippetLines { OutLines.push(SLine); } }
            }
            else
            { NonFenceBuffer.push(Line); I += 1; }
        }

        FlushNonFence( & mut NonFenceBuffer, & mut OutLines, Config);

        let mut ResultText = OutLines.join(LineEnding);
        if EndsWithNewline && ! ResultText.ends_with(LineEnding)
        { ResultText.push_str(LineEnding); }

        Ok(ResultText)
    }

    pub fn IsFormatted(Source: & str, Config: & Config_Settings) -> bool
    { match Self::Format(Source, Config) { Ok(Formatted) => Formatted == Source, Err(_) => false, } }

    fn ParseOpeningFence<'a>(Line: & 'a str) -> Option < ( & 'a str, char , usize, & 'a str) >
    {
        let Bytes = Line.as_bytes();
        let Len = Bytes.len();
        let mut I = 0;

        while I < Len
        {
            let B = Bytes[I];
            if B == b' ' || B == b'\t' || B == b'>'
            { I += 1; }
            else if (B == b'-' || B == b'*' || B == b'+') && I + 1 < Len && (Bytes[I + 1] == b' ' || Bytes[I + 1] == b'\t')
            { I += 2; }
            else if B.is_ascii_digit()
            {
                let DigitStart = I;
                while I < Len && Bytes[I].is_ascii_digit()
                { I += 1; }
                if I < Len && (Bytes[I] == b'.' || Bytes[I] == b')') && I + 1 < Len && (Bytes[I + 1] == b' ' || Bytes[I + 1] == b'\t')
                { I += 2; }
                else
                { I = DigitStart; break ; }
            }
            else
            { break ; }
        }

        if I >= Len
        { return None; }
        let FenceChar = Bytes[I] as char;
        if FenceChar != '`' && FenceChar != '~'
        { return None; }

        let FenceStart = I;
        while I < Len && Bytes[I] as char == FenceChar
        { I += 1; }
        let FenceLen = I - FenceStart;
        if FenceLen < 3
        { return None; }

        let Prefix = & Line[..FenceStart];
        let InfoString = & Line[I..];

        if FenceChar == '`' && InfoString.contains('`')
        { return None; }

        Some((Prefix, FenceChar, FenceLen, InfoString))
    }

    fn IsClosingFence(Line: & str, OpenFenceChar: char , OpenFenceLen: usize, OpenPrefix: & str) -> bool
    {
        let Bytes = Line.as_bytes();
        let Len = Bytes.len();
        let mut I = 0;

        let OpenGtCount = OpenPrefix.chars().filter( | & C | C == '>').count();
        let mut LineGtCount = 0;

        while I < Len
        { let B = Bytes[I]; if B == b' ' || B == b'\t' { I += 1; } else if B == b'>' { LineGtCount += 1; I += 1; } else { break ; } }

        if LineGtCount != OpenGtCount
        { return false; }

        if I >= Len
        { return false; }
        if Bytes[I] as char != OpenFenceChar
        { return false; }

        let FenceStart = I;
        while I < Len && Bytes[I] as char == OpenFenceChar
        { I += 1; }
        let FenceLen = I - FenceStart;
        if FenceLen < OpenFenceLen
        { return false; }

        while I < Len
        { if Bytes[I] != b' ' && Bytes[I] != b'\t' { return false; } I += 1; }

        true
    }

    pub fn LanguageFromInfoString(Info: & str, Config: & Config_Settings) -> Option<Source_Language>
    {
        let FirstWord = Info.trim().split_whitespace().next()?;
        let CleanTag = FirstWord
       .split( & [',', '{', ':', ';', '(', '[', '<', '>', ' ', '\t'][..])
       .next()?
       .trim_matches( | C: char | ! C.is_alphanumeric() && C != '+' && C != '#' && C != '-')
       .to_lowercase();

        match CleanTag.as_str()
        {
            "rust" | "rs" => Some(Source_Language::Rust),
            "javascript" | "js" | "mjs" | "cjs" | "node" => Some(Source_Language::JavaScript),
            "typescript" | "ts" | "mts" | "cts" => Some(Source_Language::TypeScript),
            "jsx" | "react" | "reactjs" => Some(Source_Language::Jsx),
            "tsx" | "react-ts" => Some(Source_Language::Tsx),
            "c" | "h" => Some(Source_Language::C),
            "cpp" | "c++" | "cc" | "cxx" | "hpp" | "hxx" | "hh" | "inl" | "cu" => Some(Source_Language::Cpp),
            "csharp" | "cs" | "c#" | "dotnet" => Some(Source_Language::CSharp),
            "go" | "golang" => Some(Source_Language::Go),
            "kotlin" | "kt" | "kts" =>
            { if Config.Kotlin_Style == Kotlin_Style::Official { Some(Source_Language::KotlinOfficial) } else { Some(Source_Language::Kotlin) } },
            "dart" | "flutter" => Some(Source_Language::Dart),
            "php" | "phtml" => Some(Source_Language::Php),
            "html" | "htm" | "xhtml" | "vue" | "svelte" => Some(Source_Language::Html),
            "css" | "scss" | "less" => Some(Source_Language::Css),
            "shell" | "bash" | "sh" | "zsh" | "ksh" => Some(Source_Language::Shell),
            "batch" | "bat" | "cmd" => Some(Source_Language::Batch),
            "toml" => Some(Source_Language::Toml),
            "cargo-lock" | "cargolock" | "lock" => Some(Source_Language::CargoLock),
            "json" | "jsonc" | "json5" => Some(Source_Language::Generic),
            "generic" | "c-like" => Some(Source_Language::Generic),
            _ => None,
        }
    }

    fn ComputeContentPrefix(OpenPrefix: & str) -> String
    {
        if OpenPrefix.contains('>')
        { OpenPrefix.to_string() }
        else
        { OpenPrefix.chars().map( | C | if C == '\t' { '\t' } else { ' ' }).collect() }
    }

    fn StripPrefix<'a>(Line: & 'a str, OpenPrefix: & str, ContentPrefix: & str) -> & 'a str
    {
        if ! ContentPrefix.is_empty() && Line.starts_with(ContentPrefix)
        { return & Line[ContentPrefix.len()..]; }
        if ! OpenPrefix.is_empty() && Line.starts_with(OpenPrefix)
        { return & Line[OpenPrefix.len()..]; }
        if OpenPrefix.contains('>')
        {
            let Trimmed = Line.trim();
            if Trimmed == ">"
            { return ""; }
            if Line.starts_with('>')
            { let After = & Line[1..]; if After.starts_with(' ') { return & After[1..]; } return After; }
        }
        let mut StripCount = 0;
        let TargetLen = ContentPrefix.len();
        for (Idx, Ch) in Line.char_indices()
        { if (Ch == ' ' || Ch == '\t') && StripCount < TargetLen { StripCount += 1; } else { return & Line[Idx..]; } }
        ""
    }

    fn PrependPrefix(Line: & str, ContentPrefix: & str) -> String
    {
        if Line.is_empty()
        { if ContentPrefix.contains('>') { ContentPrefix.trim_end().to_string() } else { String::new () } }
        else
        { format ! ("{}{}", ContentPrefix, Line) }
    }

    fn FormatSnippet(SnippetLines: & [String], Lang: Source_Language, OpenPrefix: & str, ContentPrefix: & str, Config: & Config_Settings,) -> Option<Vec<String>>
    {
        if SnippetLines.is_empty()
        { return Some(Vec::new ()); }

        let StrippedLines: Vec< & str> = SnippetLines
       .iter()
       .map( | L | Self::StripPrefix(L, OpenPrefix, ContentPrefix))
       .collect();

        let SnippetSource = StrippedLines.join("\n");
        if SnippetSource.trim().is_empty()
        {
            let Reconstructed: Vec<String> = StrippedLines
           .iter()
           .map( | L | Self::PrependPrefix(L, ContentPrefix))
           .collect();
            return Some(Reconstructed);
        }

        let mut SubConfig = Config.clone();
        SubConfig.Language = Lang;
        SubConfig.Canvas_Width = Config.Canvas_Width.saturating_sub(ContentPrefix.len());

        let Formatted = Source_Formatter::Format( & SnippetSource, & SubConfig).ok()?;

        let FormattedLines: Vec<String> = Formatted
       .lines()
       .map( | L | Self::PrependPrefix(L, ContentPrefix))
       .collect();

        Some(FormattedLines)
    }

    pub fn UnwrapParagraphs(Lines: & [ & str]) -> Vec<String>
    {
        let mut Result: Vec<String> = Vec::with_capacity(Lines.len());
        let mut CurrentPara: Option<String> = None;
        let mut CurrentType: Option<Markdown_Line_Type> = None;

        for & Line in Lines
        {
            let LineType = Self::ClassifyLine(Line);

            match LineType
            {
                Markdown_Line_Type::Blank =>
                {
                    if let Some(Para) = CurrentPara.take()
                    { Result.push(Para); }
                    CurrentType = None;
                    Result.push(Line.to_string());
                }
                Markdown_Line_Type::Structural =>
                {
                    if let Some(Para) = CurrentPara.take()
                    { Result.push(Para); }
                    CurrentType = None;
                    Result.push(Line.to_string());
                }
                Markdown_Line_Type::ListItemStart =>
                {
                    if let Some(Para) = CurrentPara.take()
                    { Result.push(Para); }
                    if Self::HasHardBreak(Line)
                    { Result.push(Line.to_string()); CurrentType = None; }
                    else
                    { CurrentPara = Some(Line.to_string()); CurrentType = Some(Markdown_Line_Type::ListItemStart); }
                }
                Markdown_Line_Type::Blockquote =>
                {
                    let CleanContent = Self::ExtractBlockquoteContent(Line);
                    if let (Some(ref mut Para), Some(Markdown_Line_Type::Blockquote)) = ( & mut CurrentPara, CurrentType)
                    {
                        if Self::HasHardBreak(Para)
                        { Result.push(CurrentPara.take().unwrap()); CurrentPara = Some(Line.to_string()); }
                        else
                        { Para.push(' '); Para.push_str(CleanContent.trim()); }
                    }
                    else
                    {
                        if let Some(Para) = CurrentPara.take()
                        { Result.push(Para); }
                        CurrentPara = Some(Line.to_string());
                        CurrentType = Some(Markdown_Line_Type::Blockquote);
                    }
                }
                Markdown_Line_Type::Text =>
                {
                    if let Some(ref mut Para) = CurrentPara
                    {
                        if Self::HasHardBreak(Para)
                        {
                            Result.push(CurrentPara.take().unwrap());
                            CurrentPara = Some(Line.to_string());
                            CurrentType = Some(Markdown_Line_Type::Text);
                        }
                        else
                        { let Trimmed = Line.trim(); if ! Trimmed.is_empty() { Para.push(' '); Para.push_str(Trimmed); } }
                    }
                    else
                    { CurrentPara = Some(Line.to_string()); CurrentType = Some(Markdown_Line_Type::Text); }
                }
            }
        }

        if let Some(Para) = CurrentPara
        { Result.push(Para); }

        Result
    }

    fn HasHardBreak(Line: & str) -> bool
    { Line.ends_with("  ") || Line.ends_with('\\') }

    fn ExtractBlockquoteContent(Line: & str) -> & str
    {
        let Trimmed = Line.trim_start();
        if let Some(Stripped) = Trimmed.strip_prefix('>')
        { if let Some(After) = Stripped.strip_prefix(' ') { After } else { Stripped } }
        else
        { Trimmed }
    }

    fn ClassifyLine(Line: & str) -> Markdown_Line_Type
    {
        let Trimmed = Line.trim();
        if Trimmed.is_empty()
        { return Markdown_Line_Type::Blank; }

        if Trimmed.starts_with("> [!")
        { return Markdown_Line_Type::Structural; }

        if Trimmed.starts_with('#')
        {
            let HashCount = Trimmed.chars().take_while( | & C | C == '#').count();
            if HashCount <= 6
            {
                let Rem = & Trimmed[HashCount..];
                if Rem.is_empty() || Rem.starts_with(' ') || Rem.starts_with('\t')
                { return Markdown_Line_Type::Structural; }
            }
        }

        if Trimmed.len() >= 3
        && ((Trimmed.chars().all( | C | C == '-')) || (Trimmed.chars().all( | C | C == '*')) || (Trimmed.chars().all( | C | C == '_')))
        {
            return Markdown_Line_Type::Structural;
        }

        if Trimmed.starts_with('|') || (Trimmed.contains('|') && Trimmed.ends_with('|'))
        { return Markdown_Line_Type::Structural; }

        if Trimmed.starts_with("<!--")
        || Trimmed.starts_with("<div")
        || Trimmed.starts_with("</div>")
        || Trimmed.starts_with("<section")
        || Trimmed.starts_with("</section>")
        || Trimmed.starts_with("<table")
        || Trimmed.starts_with("</table>")
        || Trimmed.starts_with("<p>")
        || Trimmed.starts_with("</p>")
        { return Markdown_Line_Type::Structural; }

        if Trimmed.starts_with("- ")
        || Trimmed.starts_with("* ")
        || Trimmed.starts_with("+ ")
        { return Markdown_Line_Type::ListItemStart; }
        let Bytes = Trimmed.as_bytes();
        let mut I = 0;
        while I < Bytes.len() && Bytes[I].is_ascii_digit()
        { I += 1; }
        if I > 0
        && I < Bytes.len()
        && (Bytes[I] == b'.' || Bytes[I] == b')')
        && I + 1 < Bytes.len()
        && (Bytes[I + 1] == b' ' || Bytes[I + 1] == b'\t')
        { return Markdown_Line_Type::ListItemStart; }

        if Trimmed.starts_with('>')
        { return Markdown_Line_Type::Blockquote; }

        Markdown_Line_Type::Text
    }
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)] use crate::lexer::Lexer_Scanner;
use crate::types:: { Source_Language, Token_Kind };

pub struct Safety_Verifier;

impl Safety_Verifier
{
    pub fn VerifyTokenEquivalence(OriginalSource: & str, FormattedSource: & str, Language: Source_Language,) -> Result < (), String >
    {
        let mut OriginalScanner = Lexer_Scanner::New(OriginalSource, Language);
        let OriginalTokens = OriginalScanner.TokenizeAll();

        let mut FormattedScanner = Lexer_Scanner::New(FormattedSource, Language);
        let FormattedTokens = FormattedScanner.TokenizeAll();

        let OrigSignificant: Vec < & str > = OriginalTokens.iter().filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline)).map( | T | T.Text.as_str()).collect();

        let FormSignificant: Vec < & str > = FormattedTokens.iter().filter( | T | ! matches ! (T.Kind, Token_Kind::Whitespace(_) | Token_Kind::Newline))
       .map( | T | T.Text.as_str()).collect();

        if OrigSignificant.len() != FormSignificant.len()
        {
            let diff_len = std::cmp::min(OrigSignificant.len(), FormSignificant.len());
            let mut FoundMismatch = false;
            for i in 0..diff_len
            {
                if OrigSignificant[i] != FormSignificant[i] && ! Self::AreTokensEquivalent(OrigSignificant[i], FormSignificant[i])
                {
                    let Start = i.saturating_sub(5);
                    let EndOrig = (i + 5).min(OrigSignificant.len());
                    let EndForm = (i + 5).min(FormSignificant.len());
                    eprintln!("Mismatch at index {}:\nOriginal: {:?}\nFormatted: {:?}", i, OrigSignificant[i], FormSignificant[i]);
                    eprintln!("Context Original: {:?}", & OrigSignificant[Start..EndOrig]);
                    eprintln!("Context Formatted: {:?}", & FormSignificant[Start..EndForm]);
                    FoundMismatch = true;
                    break;
                }
            }
            if ! FoundMismatch
            {
                eprintln!("Tail Original: {:?}", & OrigSignificant[OrigSignificant.len().saturating_sub(10)..]);
                eprintln!("Tail Formatted: {:?}", & FormSignificant[FormSignificant.len().saturating_sub(10)..]);
            }
            return Err(format ! ("Safety Gate Failed: Significant token count mismatch! (Original: {}, Formatted: {})", OrigSignificant.len(), FormSignificant.len()));
        }

        for (Index, (Orig, Form)) in OrigSignificant.iter().zip(FormSignificant.iter()).enumerate()
        {
            if Orig != Form && ! Self::AreTokensEquivalent(Orig, Form)
            {
                let Start = Index.saturating_sub(5);
                let EndOrig = (Index + 5).min(OrigSignificant.len());
                let EndForm = (Index + 5).min(FormSignificant.len());
                eprintln!("Context Original: {:?}", & OrigSignificant[Start..EndOrig]);
                eprintln!("Context Formatted: {:?}", & FormSignificant[Start..EndForm]);
                return Err(format ! ("Safety Gate Failed: Token #{} mismatch! (Expected '{}', Got '{}')", Index, Orig, Form));
            }
        }

        Ok(())
    }

    fn AreTokensEquivalent(Orig: & str, Form: & str) -> bool
    {
        if Orig == Form { return true; }

        if (Orig.starts_with("//") && Form.starts_with("//")) || (Orig.starts_with("/*") && Form.starts_with("/*")) || (Orig.starts_with('#') && Form.starts_with('#'))
        { if Orig.trim_end() == Form.trim_end() { return true; } }

        if Orig.starts_with('#') && Form.starts_with('#')
        { if Self::ArePreprocessorDirectivesEquivalent(Orig, Form) { return true; } }

        if
        (
            Orig.starts_with('"') || Orig.starts_with("r\"") || Orig.starts_with("r#") || Orig.starts_with("br") || Orig.starts_with("cr") || Orig.starts_with("b\"") || Orig.starts_with('`') || Orig.starts_with('\'')
        )
        &&
        (
            Form.starts_with('"') || Form.starts_with("r\"") || Form.starts_with("r#") || Form.starts_with("br") || Form.starts_with("cr") || Form.starts_with("b\"") || Form.starts_with('`') || Form.starts_with('\'')
        )
        {
            let OrigLines: Vec < & str > = Orig.lines().collect();
            let FormLines: Vec < & str > = Form.lines().collect();
            if OrigLines.len() == FormLines.len()
            {
                let AllLinesMatch = OrigLines.iter().zip(FormLines.iter()).all( | (OL, FL) | OL.trim() == FL.trim());
                if AllLinesMatch { return true; }
            }
        }

        false
    }

    fn ArePreprocessorDirectivesEquivalent(Orig: & str, Form: & str) -> bool
    {
        let OrigTokens = Self::ExtractDirectiveTokens(Orig);
        let FormTokens = Self::ExtractDirectiveTokens(Form);
        OrigTokens == FormTokens
    }

    fn ExtractDirectiveTokens(Text: & str) -> Vec<String>
    {
        let mut Tokens = Vec:: new ();
        let Chars: Vec < char > = Text.chars().collect();
        let mut I = 0;
        let Len = Chars.len();

        while I < Len
        {
            let Ch = Chars[I];

            // Skip whitespace and line continuation backslashes
            if Ch.is_whitespace() || Ch == '\\'
            { I += 1; continue ; }

            // Line comment
            if Ch == '/' && I + 1 < Len && Chars[I + 1] == '/'
            { while I < Len && Chars[I] != '\n' { I += 1; } continue ; }

            // Block comment
            if Ch == '/' && I + 1 < Len && Chars[I + 1] == '*'
            {
                I += 2;
                while I + 1 < Len && !(Chars[I] == '*' && Chars[I + 1] == '/')
                { I += 1; }
                I = (I + 2).min(Len);
                continue;
            }

            // String literal
            if Ch == '"'
            {
                let mut S = String:: new ();
                S.push(Ch);
                I += 1;
                while I < Len
                {
                    let C = Chars[I];
                    S.push(C);
                    I += 1;
                    if C == '\\' && I < Len
                    { S.push(Chars[I]); I += 1; }
                    else if C == '"'
                    { break ; }
                }
                Tokens.push(S);
                continue;
            }

            // Char literal
            if Ch == '\''
            {
                let mut S = String:: new ();
                S.push(Ch);
                I += 1;
                while I < Len
                {
                    let C = Chars[I];
                    S.push(C);
                    I += 1;
                    if C == '\\' && I < Len
                    { S.push(Chars[I]); I += 1; }
                    else if C == '\''
                    { break ; }
                }
                Tokens.push(S);
                continue;
            }

            // Identifiers / numbers / keywords
            if Ch.is_ascii_alphanumeric() || Ch == '_'
            {
                let mut S = String:: new ();
                while I < Len && (Chars[I].is_ascii_alphanumeric() || Chars[I] == '_')
                { S.push(Chars[I]); I += 1; }
                Tokens.push(S);
                continue;
            }

            // Multi-char operators (==, !=, <=, >=, ->, &&, ||, <<, >>)
            if I + 1 < Len
            {
                let TwoChars = format!("{}{}", Ch, Chars[I + 1]);
                if matches!(TwoChars.as_str(), "==" | "!=" | "<=" | ">=" | "->" | "&&" | "||" | "<<" | ">>" | "++" | "--" | "+=" | "-=" | "*=" | "/=")
                {
                    Tokens.push(TwoChars);
                    I += 2;
                    continue;
                }
            }

            // Single punctuation / operator
            Tokens.push(Ch.to_string());
            I += 1;
        }

        Tokens
    }

    pub fn VerifyIdempotence<F>(FormattedSource: & str, FormatFn: F) -> Result < (), String > where F: Fn( & str) -> String,
    {
        let SecondPass = FormatFn(FormattedSource);
        if SecondPass != FormattedSource { return Err("Safety Gate Warning: Formatting is not idempotent on second pass!".to_string(),); }
        Ok(())
    }
}

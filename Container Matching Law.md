# Container Matching Law

*Final specification · Version 1.0 · 15 November 2026 · for the LAFS container engine*

## 1\. The Law in One Paragraph

**The Strict Matching Law.** Container matching is a single left-to-right pass over a typed token stream, never a per-opener forward search. A profile-driven lexer first separates payload (strings, char literals, raw and tagged verbatim blocks, comments, regex literals, preprocessor lines, markup text) from structure, tracking context on a mode stack so that interpolation re-enters code and returns to the exact enclosing string. Glyphs inside payload never reach the matcher. Structural openers are pushed onto a stack, and a closer is legal only if it is the exact partner of the entry on top: it never closes a different kind, never closes anything deeper, and the scan never advances past a foreign closer in search of a better one, so depth lives in the stack and same-kind nesting is automatic. Identical-glyph containers are resolved by state rather than direction, and tagged ones (raw-string hashes, `$tag$`, heredoc names, triple-quote kind) must match their tag exactly. Overloaded glyphs (`<`, `|`) enter only as tentative openers where the language profile admits them, are demoted to plain operators the instant a token outside their shape or a hard closer proves them wrong, and are promoted only when their partner arrives and the profile's follow-set validates the context, with demotion as the safe default. Any orphan closer, mismatch, unclosed opener, unterminated payload, depth overflow, or unresolved lexical ambiguity makes the matcher fail closed: the file (or, in opt-in quarantine mode, the affected region) is returned byte-for-byte unchanged with the exact position reported. It never guesses, never repairs, never searches ahead. Finally, nothing is trusted until it round-trips: the formatted output must preserve the token stream, re-match to the identical pairing tree, and reformat to itself, otherwise the original is returned.

## 2\. From the Original Rule to the Hardened Rule

The original rule has three clauses plus a warning. Each survives, but each moves to the layer that can actually enforce it.

|Original clause|Hardened form|Where|
|-|-|-|
|Never assume a different closing container|Typed partners, and a closer must partner the entry on top of the stack|§6, M2 and M3|
|Keep searching forward until the matching closer is found|One forward pass in which depth is the stack: no per-opener search, and the scan never skips a foreign closer|§6, M1 and M4|
|Keep in mind ALL the containers the language recognizes|A declarative per-language profile lists every container kind, including identical-glyph, tagged, keyword and tentative ones|§5.1 and §11|
|Beware masquerading containers inside quotes and similar glyphs|A lexer layer with one mode stack resolves payload, escapes, interpolation and context-sensitive lexemes before matching|§5|

This specification adds four things the original rule lacked: tentative containers for overloaded glyphs (§7), a fail-closed failure doctrine (§8), a verification gate (§9), and resource limits (§10).

## 3\. Guarantees and Non-Goals

* **G1, Soundness:** the matcher produces the pairing the language's own grammar would produce, or no pairing at all.
* **G2, Non-corruption:** output differs from input only in whitespace, plus an explicit whitelist of intentional token edits (for example a trailing comma).
* **G3, Local failure:** an error is reported at the first token that makes a valid pairing impossible, not wherever a later search happened to land.
* **G4, Totality:** it terminates on every input in O(n) time and O(depth) memory, never panics, and never recurses.
* **G5, Determinism:** the result depends on tokens and the language profile only, never on prior whitespace or line breaks (consistent with Monolith First).
* **Non-goals:** it is not a grammar validator (it checks container structure only), it never repairs input, it does not choose layouts (the layout engine consumes its pairing record), and it does not claim to resolve glyphs that are genuinely ambiguous without semantic information (see §14).

## 4\. Vocabulary

* **Payload:** text whose glyphs have no structural meaning (strings, comments, regex, verbatim blocks, markup text).
* **Opener, closer, partner:** every opener has exactly one partner kind, and a partner is never inferred from a neighbor.
* **Hard closer:** a closer that is unambiguous in every context (`)`, `]`, `}`, and the language's keyword or tag closers).
* **Soft closer:** a glyph that closes only when a tentative opener is on top of the stack (`>`, and the closing `|` of a closure).
* **Tentative opener:** an overloaded glyph (`<`, `|`) pushed provisionally and later promoted or demoted.
* **Tagged verbatim container:** a payload container whose closer must repeat its tag (`r##"…"##`, `$body$…$body$`).
* **Keyword container:** `begin/end`, `do/end`, `<Tag>…</Tag>`, admitted only in the positions the profile lists (so Ruby's `x.end` is not a closer).
* **Pairing record:** the list of (opener, closer) pairs the matcher emits, which is the only thing the layout engine reads.
* **Checkpoint:** a position where the stack is empty and the lexer is in Code mode.

## 5\. Layer 1: The Lexer

### 5.1 The Language Profile

A declarative table, one per language, with these fields:

* Comment markers, and whether block comments nest.
* String kinds: delimiters, escape character, whether they may span lines, prefixes, raw and tagged forms, and whether they interpolate.
* The char-literal versus lifetime or label rule.
* Interpolation openers and the lexer mode each enters.
* The regex-literal rule.
* Markup and JSX text modes, including raw-text elements.
* Directive lines and their continuation rule.
* Keyword containers and the positions where they are admitted.
* Tentative glyphs, each with an admission predicate, a shape automaton and a follow-set.
* Atomic multi-character tokens.

### 5.2 Rules

* **L1, Maximal munch:** `->`, `=>`, `>=`, `<=`, `<<`, `::`, `\&\&`, `||`, `//`, `/\*`, `${` and `$$` are lexed first and never split into container candidates. The one exception is `>`: runs are lexed as single `>` tokens with a Joint flag and fused into `>>`, `>=` or `>>=` only if the matcher does not consume them as closers.
* **L2, One unified stack:** every container entry records the lexer mode to resume on pop. Payload containers that interpolate (templates, interpolated strings) are stack entries too, and so is each interpolation: a `${` inside a template pushes an entry whose resume mode is Template, so its matching `}` returns to the exact enclosing string at any depth. Plain strings without interpolation are single payload tokens and never touch the stack. Lexer and matcher run in lockstep, and the lexer reads its mode from the top of the matcher's stack.
* **L3, Escapes:** honored per profile and resolved before delimiter checks, so `"\\""` and `'\\\\'` cannot leak.
* **L4, Bounded payload:** a single-line string kind ends at the newline. An unterminated one is `E\_UNTERMINATED` at its opening quote, never "up to the next quote anywhere in the file".
* **L5, Tagged verbatim closers:** the closer repeats the tag or hash count exactly. This covers Rust `r##"…"##`, C++ `R"d(…)d"`, Postgres `$tag$…$tag$` (where `$1` is a parameter, not a tag), heredoc names, Lua `\[==\[…]==]` and the Python triple-quote kind.
* **L6a, Rust apostrophes:** a backslash after `'` means a char literal, and one scalar followed by `'` means a char literal. Anything else is a lifetime or label, so `<'a>(X: \&'a str)` can never open a string.
* **L6b, Regex versus division:** decided by the previous significant token and confirmed by same-line lookahead for the terminating `/`, with `\[...]` classes honored. `)` and `}` are the ambiguous predecessors.
* **L6c, Markup text:** in JSX and XML text mode `'` and `"` are payload (`don't`), while `{` re-enters code through a container.
* **L6d, Comment nesting:** Rust and Postgres block comments nest, while C and JS block comments do not.
* **L6e, Digit separators:** in C++ `1'000'000` the apostrophe is not a char-literal opener, so the profile decides by the preceding digit.
* **L7, Conditional compilation:** each `#if/#elif/#else` branch is matched against the same entry stack. All branches must end with identical kind-and-depth stacks, otherwise `E\_BRANCH`. A surviving opener becomes a Branch Group and is formatted opaque (verbatim).
* **L8, Local ambiguity only:** lexical ambiguities are resolved by bounded same-line lookahead. If that cannot decide, the result is `E\_AMBIGUOUS`. Global backtracking is forbidden.
* **L9, Input hygiene:** input must be valid UTF-8 (`E\_ENCODING` otherwise). BOM and line endings are preserved exactly. Positions are reported as line and column computed from byte offsets, with multi-byte characters handled.

## 6\. Layer 2: The Matcher

* **M1:** one pass, no per-opener search, and no backtracking except tentative demotion.
* **M2, Typed partners:** a closer's kind must equal its opener's kind. Cross-kind pairing is impossible by construction.
* **M3, Top-of-stack:** a hard closer is valid only if it partners the entry on top. Otherwise the matcher stops at that token and never scans past a foreign closer.
* **M4:** depth is the stack, so same-kind nesting needs no extra counting.
* **M5, Identical glyphs by state:** if the same glyph (and, for tagged forms, the same tag) is on top, it closes, otherwise it opens. Inside a verbatim container only its own closer, and its interpolation openers if any, are recognized.
* **M6:** unpaired structural characters (`,`, `;`, `:`, `.`) never touch the stack.
* **M7, Pairing record:** every pop records an (opener, closer) pair. The layout engine (Monolith, Grouping, Hierarchy) consumes that record and MUST NOT re-scan the text.

```text
Stack = StackOf(Entry);

for (Token : Lexer.Stream(Profile))
{
    if (Token.IsTrivia) { continue; }

    // A token the shape automaton rejects proves a tentative glyph was an operator all along.
    while (Stack.NotEmpty \&\& Stack.Top.IsTentative \&\& !Stack.Top.Shape.Accepts(Token)) { Stack.Pop().DemoteToOperator(); }

    if (Token.IsPayload || Token.IsUnpaired) { continue; }

    if (Token.IsOpener)
    {
        if (Stack.Depth == Limits.MAX\_DEPTH) { return Fail(E\_DEPTH, Token); }
        Stack.Push(Entry(Token, Lexer.Mode));
        Lexer.Enter(Profile.BodyMode(Token));
        continue;
    }

    if (Token.IsSoftCloser)
    {
        if (Stack.NotEmpty \&\& Stack.Top.IsTentative \&\& Stack.Top.Partners(Token))
        {
            if (Profile.Follow.Accepts(Lexer.PeekSignificant())) { Pairs.Add(Stack.Pop(), Token); }
            else { Stack.Pop().DemoteToOperator(); }
        }
        continue; // any other soft closer is a plain operator
    }

    if (Stack.IsEmpty) { return Fail(E\_ORPHAN, Token); }
    if (!Stack.Top.Partners(Token)) { return Fail(E\_MISMATCH, Token, Stack.Top); } // never search past a foreign closer

    Entry Opened = Stack.Pop();
    Lexer.Resume(Opened.ResumeMode);
    Pairs.Add(Opened, Token);
}

while (Stack.NotEmpty \&\& Stack.Top.IsTentative) { Stack.Pop().DemoteToOperator(); }

if (Stack.NotEmpty) { return Fail(E\_UNCLOSED, Stack.Top); }
if (Lexer.Mode != Code) { return Fail(E\_UNTERMINATED, Lexer.PayloadStart); }

return Verify(Pairs);
```

## 7\. Tentative Containers

* **T1, Admission:** a glyph becomes a tentative opener only if the profile's admission predicate holds for the previous significant token. Otherwise it is an operator immediately.
* **T2, Shape:** each tentative entry carries a small per-profile shape automaton for what may appear directly inside it (identifiers, `,`, `::`, `.`, `\&`, lifetimes, the profile's keywords, and nested containers). Any token it rejects, such as a statement terminator, an operator outside the alphabet, or two adjacent bare identifiers, demotes the entry.
* **T3, Hard closers and EOF:** a hard closer arriving demotes every tentative entry above its partner (the shape check does this because no shape accepts a hard closer). Tentative entries still open at EOF are demoted. A non-tentative entry in the way is a real mismatch.
* **T4, Promotion:** a soft closer (`>`, splitting `>>` when two tentatives are stacked) closes the top tentative opener only if the profile's follow-set accepts the next significant token. The follow-set lists what may legally come after the closer, for example `(`, `)`, `]`, `}`, `,`, `;`, `:`, `.`, `::`, `=`, `>`, `{`, end of line or file, and a few keywords. Otherwise the opener is demoted.
* **T5, Demotion is the safe default:** a demoted glyph becomes an opaque token with its source whitespace untouched, so a wrong demotion can cost a pretty layout but can never corrupt structure.

|Language family|`<` `>` admitted as containers|Vertical bar admitted as container|
|-|-|-|
|JavaScript|JSX tags in JSX mode only, never generics|never|
|TypeScript / TSX|after a generic-capable identifier or keyword, and at arrow-generic start in TSX|never|
|Rust|after `::` (turbofish) anywhere, bare `<` only in type position, comparison in expressions|closure parameters at expression start (`|
|C, C++, Java, C#, Dart, Kotlin, Swift|after an identifier when the follow-set validates|never|
|Ruby|n/a|block parameters directly after `{` or `do`|

## 8\. Failure Doctrine

|Code|Meaning|Reported at|
|-|-|-|
|`E\_ORPHAN`|closer with an empty stack|the closer|
|`E\_MISMATCH`|closer is not the partner of the top|the closer, plus the open entry it should have closed|
|`E\_UNCLOSED`|opener still open at EOF|the opener|
|`E\_UNTERMINATED`|payload still open at line end or EOF|the payload start|
|`E\_BRANCH`|conditional branches leave different stacks|the directive|
|`E\_DEPTH`|nesting beyond the limit|the opener that crossed it|
|`E\_AMBIGUOUS`|lexical ambiguity local lookahead cannot settle|the lexeme|
|`E\_ENCODING`|invalid UTF-8|the byte offset|
|`E\_VERIFY`|verification gate failed|the whole file|

* **F1, Fail closed:** on any error the input is returned byte-for-byte unchanged.
* **F2, Never repair:** no inserting a missing closer and no dropping a stray one.
* **F3, Report both ends:** for `E\_MISMATCH` the message names the opener and the closer with positions.
* **F4, Strict by default:** the whole file stays verbatim on error. An opt-in quarantine mode may resume at profile-defined checkpoint candidates with a fresh empty stack. Failed spans stay verbatim, accepted spans must pass the gate independently, and quarantine is disabled entirely after `E\_UNTERMINATED` because the lexer mode after it is unknown.

## 9\. Verification Gate

Nothing is emitted until it passes all three checks.

* **V1, Token preservation:** the output token stream equals the input stream, except for whitelisted edits.
* **V2, Pairing stability:** re-running the matcher on the output yields the same pairing tree, mapped by token index.
* **V3, Idempotence:** formatting the output again returns it byte-identical.
* **V4, Differential (CI only):** pairings and token streams are compared against an independent parser such as tree-sitter on a large corpus, to catch lexer bugs that V1 to V3 would share. The parser is an oracle for tests, not a runtime dependency.

## 10\. Resource Safety and Defaults

* The stack is heap-allocated and explicit. Neither the matcher nor the layout engine recurses.
* Nesting is capped by a configurable maximum, so extreme nesting gives `E\_DEPTH`, not a crash.
* The lexer is a hand-written state machine with no backtracking regex engine, so there is no catastrophic backtracking and no per-line assumption. A multi-MB single line is as fast as any other input.

|Setting|Default|
|-|-|
|Maximum nesting depth|2048|
|Local lookahead|at most 64 significant tokens, never past a profile-defined statement boundary|
|Quarantine mode|off|
|Verification gate|on, and not disableable in release builds|
|Error output|error code, position, and both ends for mismatches|

## 11\. Language Profile Checklist

Every profile must encode the hazards for its family. The table is the minimum, not the whole list.

|Family|Hazards the profile must encode|
|-|-|
|Rust|lifetimes versus char literals, raw strings with hashes, nestable block comments, turbofish, closure bars, macro bodies|
|JavaScript / TypeScript|nested template literals, regex versus division, JSX text and tags, TS generics versus comparison, arrow generics in TSX|
|C / C++|preprocessor branches and line continuation, raw strings with delimiters, digit separators, char literals such as `'('`, `a<b>(c)` ambiguity|
|Java / C# / Kotlin / Swift / Dart|text blocks and triple quotes, interpolation forms (`$x`, `${}`, `\\(…)`, `{}`), verbatim strings with doubled quotes, generics|
|Python|prefixed and triple-quoted strings, nested quotes in f-strings, indentation is not a container, line continuation|
|Shell|`case … esac` patterns use an unpaired `)`, `$(…)` and `$((…))`, heredocs, `'` has no escapes, `#` is a comment only at word start (not in `$#` or `${#x}`)|
|SQL|doubled-quote escapes, `--` comments, nestable block comments in Postgres, dollar quoting|
|HTML / XML|void elements and optional end tags in HTML (implied-close table, or opaque bodies), strict nesting in XML and JSX, raw-text elements (`script`, `style`), CDATA, comments|
|Lua / Ruby / PHP|Lua long brackets, Ruby percent literals and heredocs, PHP heredocs|
|Perl and other context-dependent lexers|ship an opaque-by-default profile|

## 12\. Torture Battery (Must Pass)

|Case|Required outcome|
|-|-|
|JS `if (p < ku \&\& (I = Bu(D, O))) { … } k > wu ? 1 : 2;`|`<` and `>` never containers, and only parens and braces are paired|
|Rust `fn F<'a>(X: \&'a str) -> Vec<Vec<\&'a str>>`|lifetimes lexed as labels, `->` atomic, `>>` splits into two closers|
|Rust `Box<dyn Fn(i32) -> i32>`|`->` atomic, and the final `>` pairs with its `<`|
|Rust raw string `r##"a "# b ( "##`|one payload token, and the `(` is never seen|
|A string holding an escaped quote, then a char literal holding an escaped backslash|escapes resolved before delimiter checks, nothing leaks|
|Template literal whose `${ … }` code contains another template literal with its own `${ … }`|correct tree at every depth, and the lexer returns to the right template at each `}`|
|`a = b / c; d = /\[/)]+/g.test(e)`|division versus regex decided correctly, and the `)` inside the class is payload|
|JSX `<p>It's {Name}</p>`|apostrophe is payload, braces are a container|
|Dart `'${A\['b']}'`|nested quotes inside interpolation pair correctly|
|C++ `int X = 1'000'000;`|digit separators are not char-literal openers|
|Shell `case $X in a) echo hi ;; esac`|the `)` is unpaired punctuation|
|Shell `echo $# ${#Args\[@]}`|`#` is not a comment start here|
|`( \[ ) ]`|`E\_MISMATCH` at `)`, both ends reported, output byte-identical|
|`if (x > 0 {` followed by a block|`E\_MISMATCH` at the first `}` that cannot close the `(`, not 50 lines later|
|An unterminated string quote and a later quote on another line|`E\_UNTERMINATED` at the opening quote|
|`#if A` / `if (a) {` / `#else` / `if (b) {` / `#endif` / `}`|identical branch stacks, accepted as a Branch Group, formatted opaque|
|`#if A` / `{` / `#endif` with no other branch leaving the same stack|`E\_BRANCH`|
|C++ `a<b>(c)`|ambiguous, so demoted, with source whitespace kept|
|Postgres `$$…$$`, `$body$…$body$` and a `$1` parameter|tags match exactly, and `$1` is a parameter|
|HTML `<br>` and `<li>` without end tags|handled by the implied-close table, or the element body stays opaque|
|4 MB minified single line|linear time, no recursion, no stack overflow|
|100,000 nested `(`|`E\_DEPTH`, no crash|
|File truncated at every byte offset|never panics, either valid or fail closed|

## 13\. Fuzzing and Property Tests

* **Mutation:** delete, duplicate, swap or insert a random delimiter in valid files. The result must be either a pairing identical to the reference parser's or a fail-closed error, never a panic and never an altered token.
* **Truncation:** cut every corpus file at every byte offset and require no panic.
* **Idempotence:** format, format again, and require identical bytes.
* **Differential:** compare pairings and token streams against tree-sitter across the full corpus, per language profile.

## 14\. Known Limits

* Genuinely ambiguous glyphs (C++ `a<b>(c)`, TypeScript expression versus type) are resolved by demotion, which can cost a pretty layout but never structure.
* Macro and metaprogramming tricks (Rust `macro\_rules!`, preprocessor token pasting) can produce unbalanced fragments, which fail closed or stay opaque per profile.
* Languages whose lexing needs a full parser (Perl, some Ruby forms, code built by `eval`) get opaque-by-default profiles.
* The matcher validates container structure only, so a file can pass and still contain a syntax error.
* Shape automata and follow-sets are per-language heuristics that need maintenance as syntax evolves, and the differential corpus (V4) is how drift is caught.

## 15\. Definition of Done and Conformance

* The full torture battery passes.
* A fuzzing campaign finishes with zero panics, zero altered tokens, and every failure localized to the first impossible token.
* V1 to V3 run on every formatted file, and V4 is clean on the corpus.
* Every profile has a reviewed checklist entry (§11) and a regression test per hazard.
* A new language ships only with its profile, its torture rows, and its differential corpus.


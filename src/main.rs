#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
mod container_tree;
mod changelog;
mod emitter;
mod formatter;
mod languages;
mod lexer;
mod matcher;
mod planner;
mod profile;
mod types;
mod verifier;

use changelog::Changelog_Generator;

use std::env;
use std::fs;
use std::io:: { self, IsTerminal, Read, Write };
use std::path:: { Path, PathBuf };
use std::sync::atomic:: { AtomicUsize, Ordering };
use std::sync:: { Arc, Mutex };
use std::time::Instant;

use formatter::Source_Formatter;
use types:: { Config_Settings, Kotlin_Style, Source_Language };

const DEFAULT_EXCLUDED_DIRS: & [ & str] = &
[
    ".git", ".svn", ".hg", "target", "bin", "obj", "node_modules", "dist", "build", ".vs", ".vscode", ".idea", ".gradle", "venv", ".env", "vendor", "__pycache__",
    "stable-x86_64-pc-windows-msvc", "rustlib", "llvm-project", "nightly-x86_64-pc-windows-msvc", "beta-x86_64-pc-windows-msvc",
];

const DOCUMENT_EXTENSIONS: & [ & str] = &
[
    "adoc", "asciidoc", "pdf", "docx", "doc", "rst", "rtf", "odt", "pages", "ppt", "pptx", "key", "xls", "xlsx", "numbers", "csv", "tsv", "epub", "tex", "latex", "txt", "org", "textile", "yml", "yaml", "log",
];

const BINARY_EXTENSIONS: & [ & str] = &
[
    "exe", "dll", "so", "dylib", "obj", "o", "a", "lib", "bin", "iso", "img", "zip", "tar", "gz", "tgz", "7z", "rar", "bz2", "xz", "zst", "png", "jpg", "jpeg", "gif", "webp", "ico", "bmp", "tiff", "tif", "psd", "ai", "svgz", "mp3", "mp4", "wav", "ogg", "mkv", "avi", "mov", "flv", "webm", "m4a", "flac", "woff", "woff2", "ttf", "eot", "otf", "pyc", "pyo", "class", "pdb", "idb", "suo", "db", "sqlite", "sqlite3", "lockb",
];

const PYTHON_EXTENSIONS: & [ & str] = &
["py", "pyw", "pyi", "pyx", "pxd", "pyd",];

const DEFAULT_SUPPORTED_EXTENSIONS: & [ & str] = &
[
    "c", "h", "cpp", "hpp", "cc", "cxx", "hh", "hxx", "inl", "cu", "cs", "rs", "java", "kt", "kts", "scala", "js", "mjs", "cjs", "jsx", "ts", "tsx", "mts", "cts", "go", "dart", "php", "swift", "m", "mm", "sh", "bash", "zsh", "ksh", "json", "jsonc", "json5", "toml", "lock", "css", "scss", "less", "sql", "graphql", "proto", "html", "htm", "bat", "cmd", "md", "markdown", "mdown", "mkd", "typ",
];

#[derive(Clone)]
pub struct Terminal_Logger { Out_Buffer: Arc<Mutex<String>>, Err_Buffer: Arc<Mutex<String>>, Timestamp_Str: String, Disable_Log_Files: Arc<Mutex<bool>>, }

impl Terminal_Logger
{
    pub fn New(Disable_Logs: bool) -> Self
    {
        let Timestamp_Str = chrono::Local::now().format("%Y-%m-%d %H-%M-%S").to_string();
        Terminal_Logger
        {
            Out_Buffer: Arc:: new (Mutex::new (String::new ())),
            Err_Buffer: Arc:: new (Mutex::new (String::new ())),
            Timestamp_Str,
            Disable_Log_Files: Arc:: new (Mutex::new (Disable_Logs)),
        }
    }

    pub fn Set_Disable_Logs( & self, Disable: bool) { * self.Disable_Log_Files.lock().unwrap() = Disable; }

    pub fn Println( & self, Msg: & str)
    {
        println!("{}", Msg);
        let mut Buf = self.Out_Buffer.lock().unwrap();
        Buf.push_str(Msg);
        Buf.push('\n');
    }

    pub fn Print( & self, Msg: & str)
    {
        print!("{}", Msg);
        io::stdout().flush().ok();
        let mut Buf = self.Out_Buffer.lock().unwrap();
        Buf.push_str(Msg);
    }

    pub fn Eprintln( & self, Msg: & str)
    {
        eprintln!("{}", Msg);
        { let mut ErrBuf = self.Err_Buffer.lock().unwrap(); ErrBuf.push_str(Msg); ErrBuf.push('\n'); }
        {
            let mut OutBuf = self.Out_Buffer.lock().unwrap();
            OutBuf.push_str(Msg);
            OutBuf.push('\n');
        }
    }

    pub fn Flush_To_Files( & self)
    {
        if * self.Disable_Log_Files.lock().unwrap() { return ; }

        let OutText = self.Out_Buffer.lock().unwrap().clone();
        let ErrText = self.Err_Buffer.lock().unwrap().clone();

        if ! OutText.is_empty()
        {
            let OutFileName = format!("lafs {}.log", self.Timestamp_Str);
            if let Err(E) = fs::write( & OutFileName, & OutText) { eprintln ! ("Failed to write terminal output log '{}': {}", OutFileName, E); }
        }

        if ! ErrText.is_empty()
        {
            let ErrFileName = format!("lafs error {}.log", self.Timestamp_Str);
            if let Err(E) = fs::write( & ErrFileName, & ErrText) { eprintln ! ("Failed to write error log '{}': {}", ErrFileName, E); }
        }
    }
}

fn PrintUsage(Logger: & Terminal_Logger)
{
    Logger.Println("LAFS Container Formatter (Language-Agnostic Formatting Style)");
    Logger.Println("Usage:");
    Logger.Println("  lafs [OPTIONS] <FILES... | DIRECTORIES...>");
    Logger.Println("  lafs -f, --file, --folder <PATH...> [OPTIONS]");
    Logger.Println("  lafs -d, --dir, --directory <DIR...> [OPTIONS]");
    Logger.Println("  lafs format [OPTIONS] <FILES... | DIRECTORIES...>");
    Logger.Println("  lafs format failed [OPTIONS]       (Reformat all files listed in failed.txt)");
    Logger.Println("  lafs apply [OPTIONS] <FILES... | DIRECTORIES...>");
    Logger.Println("  lafs check [OPTIONS] <FILES... | DIRECTORIES...>");
    Logger.Println("  lafs changelog [OPTIONS]           (Generate or update CHANGELOG.md based on git history)");
    Logger.Println("  lafs [OPTIONS] -                   (Read from stdin, output to stdout)");
    Logger.Println("");
    Logger.Println("Batch & Directory Options:");
    Logger.Println("  -a, --all                   Format all files in current directory and all subdirectories");
    Logger.Println("  -f, --file, --folder <PATH> Format specified file(s) or folder(s)");
    Logger.Println("                              (folder with trailing slash '/' or '\\' formats shallow/non-recursive;");
    Logger.Println("                               folder without trailing slash formats recursively)");
    Logger.Println("  -d, --dir, --directory <DIR>Explicitly format specified directory");
    Logger.Println("                              (trailing slash '/' or '\\' formats shallow; without slash formats recursively)");
    Logger.Println("  -e, --ext <EXTENSIONS>      Only format files with specific extension(s) (e.g. -e c,h or -e js)");
    Logger.Println("      --all-types             Format all text files regardless of extension (excludes binary and docs)");
    Logger.Println("  -x, --exclude <PATTERN>     Exclude files or folders (e.g. -x target -x vendor -x \"*.min.js\")");
    Logger.Println("  -y, --yes                   Automatic yes to confirmation prompts");
    Logger.Println("  -j, --threads <COUNT>       Number of parallel worker threads (default: CPU cores)");
    Logger.Println("      --verbose               Show details for unchanged clean files");
    Logger.Println("");
    Logger.Println("Formatting Options:");
    Logger.Println("  -w, --write                 Save and overwrite formatted files in-place (default)");
    Logger.Println("  -o, --output <PATH>         Output file or destination directory for formatted code");
    Logger.Println("      --out-dir <DIR>         Destination directory (replicates directory tree structure)");
    Logger.Println("      --prefix [PREFIX]       Prefix for output file name (default: 'formatted_')");
    Logger.Println("      --suffix [SUFFIX]       Suffix for output file name (default: '.formatted')");
    Logger.Println("      --stdout                Output formatted code to stdout instead of saving files");
    Logger.Println("      --check                 Check if files are formatted (exit code 1 if unformatted)");
    Logger.Println("  -of, --output-failed <FILE> Output failed and unformatted paths to a specific log file");
    Logger.Println("      --no-log                Disable automatic log file generation");
    Logger.Println("  -c, --canvas-width <WIDTH>  Set maximum canvas width before expansion (default: 160)");
    Logger.Println("  -i, --indent <SIZE>         Set indentation width in spaces (default: 4)");
    Logger.Println("  -t, --tabs                  Use tabs instead of spaces for indentation");
    Logger.Println("  -l, --language <LANG>       Force source language: 'c', 'cpp', 'csharp', 'js', 'ts', 'jsx', 'tsx', 'sh', 'kt', 'dart', 'rust', 'md', 'typ'");
    Logger.Println("      --kotlin-version <1|2>  Kotlin formatting version (1 = Allman for all containers, 2 = Official Kotlin)");
    Logger.Println("      --kotlin-style <STYLE>  Kotlin style: 'allman' (Version 1) or 'official' (Version 2)");
    Logger.Println("      --allman-kotlin         Enforce Version 1 (Pure Allman style for all containers)");
    Logger.Println("      --official-kotlin       Enforce Version 2 (Official Kotlin compiler compatible / Brain K&R)");
    Logger.Println("      --cah <-1|0|1>          Enable (1, default) or disable (-1/0) Column-Anchored Hybrid (Section 6.1)");
    Logger.Println("      --preserve-empty-lines <0|1> Preserve empty lines by default (1, default) or strip (0)");
    Logger.Println("      --max-blank-lines <NUM>  Max consecutive blank lines to preserve (default: 1)");
    Logger.Println("      --max-statements <NUM>  Max statements in a monolith block (default: 4)");
    Logger.Println("      --unwrap-paragraphs     Unwrap abrupt line breaks inside Markdown paragraphs");
    Logger.Println("      --no-abrupt-breaks      Alias for --unwrap-paragraphs");
    Logger.Println("      --no-unwrap-paragraphs  Disable unwrapping of Markdown paragraphs");
    Logger.Println("      --stdin-filepath <PATH> Virtual filepath when formatting stdin");
    Logger.Println("  -h, --help                  Show this help message");
    Logger.Println("  -v, --version               Show version information");
}

fn WildcardMatch(Pattern: & str, Text: & str) -> bool
{
    let PChars: Vec<char> = Pattern.chars().collect();
    let TChars: Vec<char> = Text.chars().collect();
    let mut PI = 0;
    let mut TI = 0;
    let mut StarP = None;
    let mut StarT = 0;

    while TI < TChars.len()
    {
        if PI < PChars.len() && (PChars[PI] == '?' || PChars[PI].to_ascii_lowercase() == TChars[TI].to_ascii_lowercase()) { PI += 1; TI += 1; }
        else if PI < PChars.len() && PChars[PI] == '*' { StarP = Some(PI); PI += 1; StarT = TI; }
        else if let Some(SP) = StarP { PI = SP + 1; StarT += 1; TI = StarT; } else { return false; }
    }

    while PI < PChars.len() && PChars[PI] == '*' { PI += 1; }

    PI == PChars.len()
}

fn IsBinaryFile(PathTarget: & Path) -> bool
{
    if let Ok(mut File) = fs::File::open(PathTarget)
    {
        let mut Buffer = [0u8; 8192];
        if let Ok(BytesRead) = File.read( & mut Buffer) { if BytesRead > 0 { return Buffer[..BytesRead].contains( & 0); } }
    }
    false
}

fn IsDirExcluded(DirName: & str, DirPath: & Path, ExcludedPatterns: & [String]) -> bool
{
    if DEFAULT_EXCLUDED_DIRS.iter().any( | & D | D.eq_ignore_ascii_case(DirName)) { return true; }
    if DirName.to_ascii_lowercase().starts_with("old.src") { return true; }
    let LowerDir = DirName.to_ascii_lowercase();
    if LowerDir.ends_with("-pc-windows-msvc") || LowerDir.ends_with("-pc-windows-gnu") || LowerDir.ends_with("-unknown-linux-gnu") || LowerDir.ends_with("-apple-darwin")
    {
        return true;
    }

    let DirPathStr = DirPath.to_string_lossy().replace('\\', "/");
    let LowerPath = DirPathStr.to_ascii_lowercase();
    if LowerPath.starts_with("old.src") || LowerPath.contains("/old.src") { return true; }
    if LowerPath.contains("/rustlib") || LowerPath.contains("/llvm-project") || LowerPath.contains("stable-x86_64") || LowerPath.contains("nightly-x86_64") || LowerPath.contains("beta-x86_64")
    {
        return true;
    }

    for Pattern in ExcludedPatterns
    {
        let CleanPattern = Pattern.trim_matches('/').trim_matches('\\');
        if WildcardMatch(CleanPattern, DirName) || WildcardMatch(CleanPattern, & DirPathStr) || DirPathStr.ends_with(CleanPattern) || DirPathStr.contains( & format ! ("/{}/", CleanPattern))
        || DirPathStr.starts_with( & format ! ("{}/", CleanPattern)) { return true; }
    }

    false
}

fn IsFileEligible(PathTarget: & Path, AllTypes: bool , AllowedExtensions: & [String], ExcludedPatterns: & [String],) -> bool
{
    let FileName = PathTarget.file_name().unwrap_or_default().to_string_lossy();
    let PathStr = PathTarget.to_string_lossy().replace('\\', "/");
    // 1. Check user exclusion patterns
    for Pattern in ExcludedPatterns
    {
        let CleanPattern = Pattern.trim_matches('/').trim_matches('\\');
        if WildcardMatch(CleanPattern, & FileName) || WildcardMatch(CleanPattern, & PathStr) || PathStr.ends_with(CleanPattern) { return false; }
    }

    let Ext = PathTarget.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    if Ext == "log" || FileName.to_ascii_lowercase().ends_with(".log") { return false; }
    // 2. Exclude Python file types (whitespace-delimited block semantics are unsupported)
    if PYTHON_EXTENSIONS.iter().any( | & P | P == Ext.as_str()) { return false; }
    // 3. Specific extension filter (-e / --ext / --extension)
    if ! AllowedExtensions.is_empty() { if ! AllowedExtensions.iter().any( | A | A.eq_ignore_ascii_case( & Ext)) { return false; } }
    else if ! AllTypes && ! DEFAULT_SUPPORTED_EXTENSIONS.iter().any( | & S | S == Ext.as_str()) { return false; }
    // 4. Exclude document file types (adoc, pdf, docx, md, yml, etc.)
    if DOCUMENT_EXTENSIONS.iter().any( | & D | D == Ext.as_str()) { return false; }
    // 5. Exclude known binary file types
    if BINARY_EXTENSIONS.iter().any( | & B | B == Ext.as_str()) { return false; }
    // 6. Fast binary content check for NUL bytes
    if IsBinaryFile(PathTarget) { return false; }

    true
}

fn NormalizePathSpec(Spec: & str) -> (String, bool)
{
    let mut S = Spec.trim().to_string();
    let mut HadTrailingSlash = false;

    // Handle Windows command-line quoting where \" was parsed as a literal quote at the end
    if S.ends_with('"') || S.ends_with('\'') { S.pop(); HadTrailingSlash = true; }

    if S.ends_with('\\') || S.ends_with('/')
    {
        HadTrailingSlash = true;
        while (S.ends_with('\\') || S.ends_with('/')) && S.len() > 1 && ! S.ends_with(":\\") && ! S.ends_with(":/") { S.pop(); }
    }
    (S, HadTrailingSlash)
}

fn FormatFailedEntry(FilePathStr: & str, Lang: & str, Reason: & str) -> String
{ format ! ("File: {}\nLanguage: {}\nReason: {}\n", FilePathStr, Lang, Reason.trim()) }

fn SafelyWriteFile(FilePath: & Path, Content: & str) -> std::io::Result < () >
{
    use std::io::Write;
    if let Some(Parent) = FilePath.parent()
    { if ! Parent.as_os_str().is_empty() && ! Parent.exists() { fs::create_dir_all(Parent)?; } }
    let mut File = fs::OpenOptions:: new ()
   .write(true)
   .create(true)
   .truncate(true)
   .open(FilePath)?;
    File.write_all(Content.as_bytes())?;
    File.sync_all()?;
    Ok(())
}

fn TransformFileName(FileName: & str, Prefix: Option< & str>, Suffix: Option< & str>) -> String
{
    if Prefix.is_none() && Suffix.is_none()
    { return FileName.to_string(); }

    let PathObj = Path:: new (FileName);
    let FileStem = PathObj.file_stem().and_then( | S | S.to_str()).unwrap_or(FileName);
    let Extension = PathObj.extension().and_then( | E | E.to_str());

    let Pfx = Prefix.unwrap_or("");
    let Sfx = Suffix.unwrap_or("");

    match Extension
    { Some(Ext) => { format ! ("{}{}{}.{}", Pfx, FileStem, Sfx, Ext) } None => { format ! ("{}{}{}", Pfx, FileStem, Sfx) } }
}

fn ComputeRelativeSubPath(SourceFilePath: & Path, BaseDirs: & [String]) -> PathBuf
{
    let SourceStr = SourceFilePath.to_string_lossy().replace('\\', "/");
    let CleanSource = SourceStr.trim_start_matches("./");

    if BaseDirs.len() == 1
    {
        let (CleanBase, _) = NormalizePathSpec( & BaseDirs[0]);
        let CleanBaseStr = CleanBase.replace('\\', "/");
        let TrimmedBase = CleanBaseStr.trim_start_matches("./");
        if ! TrimmedBase.is_empty() && TrimmedBase != "."
        {
            let PrefixWithSlash = format!("{}/", TrimmedBase);
            if CleanSource.starts_with( & PrefixWithSlash)
            { return PathBuf::from( & CleanSource[PrefixWithSlash.len()..]); }
            if CleanSource == TrimmedBase
            { if let Some(Name) = SourceFilePath.file_name() { return PathBuf::from(Name); } }
        }
    }

    PathBuf::from(CleanSource)
}

fn ComputeOutputPath
(
    SourceFilePath: & Path,
    OutputDest: Option< & str>,
    Prefix: Option< & str>,
    Suffix: Option< & str>,
    BaseDirs: & [String],
    IsSingleFileTarget: bool,
)
-> PathBuf
{
    let OrigFileName = SourceFilePath.file_name().and_then( | N | N.to_str()).unwrap_or_default();
    let NewFileName = TransformFileName(OrigFileName, Prefix, Suffix);

    match OutputDest
    {
        None =>
        { let Parent = SourceFilePath.parent().unwrap_or(Path::new ("")); Parent.join(NewFileName) }
        Some(DestStr) =>
        {
            let (CleanDest, HadTrailingSlash) = NormalizePathSpec(DestStr);
            let DestPath = Path:: new ( & CleanDest);

            let IsExplicitFile = IsSingleFileTarget
            && ! HadTrailingSlash
            && ! DestPath.is_dir()
            && (DestPath.extension().is_some() || ( ! CleanDest.ends_with('/') && ! CleanDest.ends_with('\\')));

            let mut FinalPath = if IsExplicitFile
            {
                if Prefix.is_some() || Suffix.is_some()
                {
                    let DestParent = DestPath.parent().unwrap_or(Path::new (""));
                    let DestOrigName = DestPath.file_name().and_then( | N | N.to_str()).unwrap_or_default();
                    let TransformedDest = TransformFileName(DestOrigName, Prefix, Suffix);
                    DestParent.join(TransformedDest)
                }
                else
                { DestPath.to_path_buf() }
            }
            else
            {
                if IsSingleFileTarget
                { DestPath.join(NewFileName) }
                else
                {
                    let RelSubPath = ComputeRelativeSubPath(SourceFilePath, BaseDirs);
                    let RelParent = RelSubPath.parent().unwrap_or(Path::new (""));
                    DestPath.join(RelParent).join(NewFileName)
                }
            };

            if FinalPath == SourceFilePath && Prefix.is_none() && Suffix.is_none()
            {
                let FallbackSuffix = Some(".formatted");
                let SafeFileName = TransformFileName(OrigFileName, Prefix, FallbackSuffix);
                if let Some(Parent) = FinalPath.parent()
                { FinalPath = Parent.join(SafeFileName); }
            }

            FinalPath
        }
    }
}

fn ParseFailedLine(Line: & str) -> Option< & str>
{
    let Trimmed = Line.trim();
    if Trimmed.is_empty() { return None; }
    if Trimmed.starts_with("Language:") || Trimmed.starts_with("Reason:") || Trimmed.starts_with("Expected:") || Trimmed.starts_with("Got:") || Trimmed.starts_with("Context:") || Trimmed.starts_with("---")
    {
        return None;
    }

    let AfterPrefix = if let Some(Stripped) = Trimmed.strip_prefix("File:") { Stripped.trim() }
    else if let Some(Stripped) = Trimmed.strip_prefix("Path:") { Stripped.trim() }
    else { Trimmed };

    let LineWithoutReason = if let Some(Idx) = AfterPrefix.find(" Reason:") { & AfterPrefix[..Idx] }
    else { AfterPrefix };
    let FilePath = if let Some(Idx) = LineWithoutReason.rfind(" (")
    { if LineWithoutReason.ends_with(')') { & LineWithoutReason[..Idx] } else { LineWithoutReason } }
    else { LineWithoutReason };
    let CleanPath = FilePath.trim();
    if ! CleanPath.is_empty() { Some(CleanPath) } else { None }
}

fn GetDefaultFailedLogPath() -> String
{
    if let Ok(ExePath) = std::env::current_exe()
    { if let Some(Parent) = ExePath.parent() { return Parent.join("failed.txt").to_string_lossy().to_string(); } }
    "failed.txt".to_string()
}

static FAILED_FILE_MUTEX: Mutex < () > = Mutex:: new (());

fn AppendFailedEntry(Entry: & str, OutputFailedPath: & Option<String>)
{
    let _Guard = FAILED_FILE_MUTEX.lock().unwrap_or_else( | P | P.into_inner());
    let TargetFile = OutputFailedPath.as_deref().unwrap_or("failed.txt");
    if let Ok(mut F) = fs::OpenOptions:: new ().create(true).append(true).open(TargetFile)
    {
        let _ = writeln!(F, "{}\n", Entry.trim_end());
        let _ = F.flush();
    }
    let BinaryFailedPath = GetDefaultFailedLogPath();
    if BinaryFailedPath != TargetFile
    {
        if let Ok(mut F) = fs::OpenOptions:: new ().create(true).append(true).open( & BinaryFailedPath)
        {
            let _ = writeln!(F, "{}\n", Entry.trim_end());
            let _ = F.flush();
        }
    }
}

fn MatchesSpecifiedFile(Candidate: & Path, Spec: & str) -> bool
{
    let CandidateStr = Candidate.to_string_lossy().replace('\\', "/");
    let SpecStr = Spec.replace('\\', "/");
    let CleanCandidate = CandidateStr.trim_start_matches("./");
    let CleanSpec = SpecStr.trim_start_matches("./");
    let CandidateName = Candidate.file_name().unwrap_or_default().to_string_lossy();
    let SpecName = Path:: new (Spec).file_name().unwrap_or_default().to_string_lossy();

    // 1. Direct or relative path match
    if CleanCandidate.eq_ignore_ascii_case(CleanSpec) || CleanCandidate.ends_with( & format ! ("/{}", CleanSpec)) { return true; }

    // 2. File name match if Spec does not specify directory separators
    if ! CleanSpec.contains('/') && CandidateName.eq_ignore_ascii_case( & SpecName) { return true; }

    // 3. Wildcard pattern match
    if (Spec.contains('*') || Spec.contains('?')) && (WildcardMatch(CleanSpec, CleanCandidate) || WildcardMatch(CleanSpec, & CandidateName)) { return true; }

    // 4. Canonical path equality check
    if let (Ok(CanonCandidate), Ok(CanonSpec)) = (fs::canonicalize(Candidate), fs::canonicalize(Path::new (Spec))) { if CanonCandidate == CanonSpec { return true; } }

    false
}

fn MatchesSpecifiedTarget(Candidate: & Path, Spec: & str) -> bool
{
    let (CleanSpec, HadTrailingSlash) = NormalizePathSpec(Spec);
    let CandidateStr = Candidate.to_string_lossy().replace('\\', "/");
    let SpecStr = CleanSpec.replace('\\', "/");
    let CleanCandidate = CandidateStr.trim_start_matches("./");
    let CleanSpecStr = SpecStr.trim_start_matches("./");

    if HadTrailingSlash
    {
        // Shallow match: candidate must be directly inside this directory
        if let Some(Parent) = Candidate.parent()
        {
            let ParentStr = Parent.to_string_lossy().replace('\\', "/");
            let CleanParent = ParentStr.trim_start_matches("./");
            let ParentName = Parent.file_name().unwrap_or_default().to_string_lossy();
            if CleanParent.eq_ignore_ascii_case(CleanSpecStr)
            || CleanParent.ends_with( & format ! ("/{}", CleanSpecStr))
            || ( ! CleanSpecStr.contains('/') && ParentName.eq_ignore_ascii_case(CleanSpecStr)) { return true; }
            if let (Ok(CanonParent), Ok(CanonSpec)) = (fs::canonicalize(Parent), fs::canonicalize(Path::new ( & CleanSpec))) { if CanonParent == CanonSpec { return true; } }
        }
        return false;
    }

    let SpecPath = Path:: new ( & CleanSpec);
    if SpecPath.is_dir()
    {
        // Recursive match: candidate is inside SpecPath directory tree
        if let (Ok(CanonCandidate), Ok(CanonSpec)) = (fs::canonicalize(Candidate), fs::canonicalize(SpecPath)) { if CanonCandidate.starts_with( & CanonSpec) { return true; } }
        if CleanCandidate.starts_with( & format ! ("{}/", CleanSpecStr)) || CleanCandidate.contains( & format ! ("/{}/", CleanSpecStr)) { return true; }
    }

    MatchesSpecifiedFile(Candidate, & CleanSpec)
}

fn IsSpecifiedFileEligible(PathTarget: & Path, AllTypes: bool , AllowedExtensions: & [String], ExcludedPatterns: & [String], ForceLanguage: Source_Language,) -> bool
{
    let Ext = PathTarget.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    let FileName = PathTarget.file_name().unwrap_or_default().to_string_lossy();
    if Ext == "log" || FileName.to_ascii_lowercase().ends_with(".log") { return false; }

    // 1. Exclude known binary file types or binary content
    if BINARY_EXTENSIONS.iter().any( | & B | B == Ext.as_str()) || IsBinaryFile(PathTarget) { return false; }

    // 2. Exclude Python file types (whitespace-delimited block semantics are unsupported)
    if PYTHON_EXTENSIONS.iter().any( | & P | P == Ext.as_str()) { return false; }

    // 3. Check user exclusion patterns
    let FileName = PathTarget.file_name().unwrap_or_default().to_string_lossy();
    let PathStr = PathTarget.to_string_lossy().replace('\\', "/");
    for Pattern in ExcludedPatterns
    {
        let CleanPattern = Pattern.trim_matches('/').trim_matches('\\');
        if WildcardMatch(CleanPattern, & FileName) || WildcardMatch(CleanPattern, & PathStr) || PathStr.ends_with(CleanPattern) { return false; }
    }

    // 3. Extension filter
    if ! AllowedExtensions.is_empty() { if ! AllowedExtensions.iter().any( | A | A.eq_ignore_ascii_case( & Ext)) { return false; } }
    else if ForceLanguage == Source_Language::Generic && ! AllTypes && ! DEFAULT_SUPPORTED_EXTENSIONS.iter().any( | & S | S == Ext.as_str()) { return false; }

    true
}

fn DetectLanguageForFile(FilePathStr: & str, Content: & str) -> Source_Language
{
    let ExtLang = Source_Language::FromExtension(FilePathStr);
    if ExtLang == Source_Language::C && (FilePathStr.ends_with(".h") || FilePathStr.ends_with(".H"))
    { if IsCppHeaderContent(Content) { return Source_Language::Cpp; } }
    ExtLang
}

fn IsCppHeaderContent(Content: & str) -> bool
{
    let CheckSlice = if Content.len() > 8192 { & Content[..8192] } else { Content };
    CheckSlice.contains("-*-C++-*-")
    || CheckSlice.contains("-*- C++ -*-")
    || CheckSlice.contains("-*-c++-*-")
    || CheckSlice.contains("ft=cpp")
    || CheckSlice.contains("extern \"C++\"")
    || CheckSlice.contains("namespace ")
    || CheckSlice.contains("template<")
    || CheckSlice.contains("template <")
    || CheckSlice.contains("nullptr")
    || CheckSlice.contains("constexpr")
    || CheckSlice.contains("reinterpret_cast<")
    || CheckSlice.contains("static_cast<")
    || CheckSlice.contains("dynamic_cast<")
    || CheckSlice.contains("std::")
}

fn CollectFilesShallow(DirPath: & Path, AllTypes: bool , AllowedExtensions: & [String], ExcludedPatterns: & [String], Collected: & mut Vec<String>,)
{
    let DirName = DirPath.file_name().unwrap_or_default().to_string_lossy();
    if ! DirName.is_empty() && DirName != "." && IsDirExcluded( & DirName, DirPath, ExcludedPatterns) { return ; }

    if let Ok(Entries) = fs::read_dir(DirPath)
    {
        for Entry in Entries.flatten()
        {
            let SubPath = Entry.path();
            if SubPath.is_file() { if IsFileEligible( & SubPath, AllTypes, AllowedExtensions, ExcludedPatterns) { Collected.push(SubPath.to_string_lossy().to_string()); } }
        }
    }
}

fn CollectFilesRecursively(DirPath: & Path, AllTypes: bool , AllowedExtensions: & [String], ExcludedPatterns: & [String], Collected: & mut Vec<String>,)
{
    let DirName = DirPath.file_name().unwrap_or_default().to_string_lossy();
    if ! DirName.is_empty() && DirName != "." && IsDirExcluded( & DirName, DirPath, ExcludedPatterns) { return ; }

    if let Ok(Entries) = fs::read_dir(DirPath)
    {
        for Entry in Entries.flatten()
        {
            let SubPath = Entry.path();
            if SubPath.is_dir()
            {
                let SubName = SubPath.file_name().unwrap_or_default().to_string_lossy();
                if ! IsDirExcluded( & SubName, & SubPath, ExcludedPatterns)
                {
                    CollectFilesRecursively( & SubPath, AllTypes, AllowedExtensions, ExcludedPatterns, Collected,);
                }
            }
            else if SubPath.is_file() { if IsFileEligible( & SubPath, AllTypes, AllowedExtensions, ExcludedPatterns) { Collected.push(SubPath.to_string_lossy().to_string()); } }
        }
    }
}

fn ExpandGlobPattern(Pattern: & str, AllTypes: bool , AllowedExtensions: & [String], ExcludedPatterns: & [String], Collected: & mut Vec<String>,)
{
    let Normalized = Pattern.replace('\\', "/");
    let (DirPart, FilePattern) = if let Some(SlashIdx) = Normalized.rfind('/') { ( & Normalized[..SlashIdx], & Normalized[SlashIdx + 1..]) }
    else { (".", Normalized.as_str()) };

    let DirPath = Path:: new (DirPart);
    if let Ok(Entries) = fs::read_dir(DirPath)
    {
        for Entry in Entries.flatten()
        {
            let P = Entry.path();
            if P.is_file()
            {
                if let Some(FileName) = P.file_name().and_then( | N | N.to_str())
                {
                    if WildcardMatch(FilePattern, FileName) && IsFileEligible( & P, AllTypes, AllowedExtensions, ExcludedPatterns)
                    {
                        Collected.push(P.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
}

fn PromptConfirmation(Logger: & Terminal_Logger, Count: usize, TargetDesc: & str, HasCustomOutput: bool) -> bool
{
    let Action = if HasCustomOutput { "Format and write to destination?" } else { "Apply LAFS formatting and save in-place?" };
    Logger.Print( & format ! ("Found {} files in '{}'. {} [Y/N]: ", Count, TargetDesc, Action));

    let mut Input = String:: new ();
    if io::stdin().read_line( & mut Input).is_ok() { let Trimmed = Input.trim().to_lowercase(); matches ! (Trimmed.as_str(), "y" | "yes") } else { false }
}

fn Main()
{
    let Logger = Terminal_Logger::New(false);
    let Args: Vec<String> = env::args().collect();
    let mut OutputToStdout = false;
    let mut CheckOnly = false;
    let mut FormatAll = false;
    let mut AllTypes = false;
    let mut AutoYes = false;
    let mut Verbose = false;
    let mut ReformatFailed = false;
    let mut CustomThreads: Option<usize> = None;
    let mut AllowedExtensions: Vec<String> = Vec:: new ();
    let mut ExcludedPatterns: Vec<String> = Vec:: new ();
    let mut SpecifiedFiles: Vec<String> = Vec:: new ();
    let mut Config = Config_Settings:: default();
    let mut RawTargets: Vec<String> = Vec:: new ();
    let mut ReadFromStdin = false;
    let mut StdinVirtualPath: Option<String> = None;
    let mut OutputFailedPath: Option<String> = None;
    let mut OutputDest: Option<String> = None;
    let mut OutputPrefix: Option<String> = None;
    let mut OutputSuffix: Option<String> = None;

    if Args.len() > 1 && (Args[1] == "changelog" || Args[1] == "--changelog" || Args[1] == "--generate-changelog")
    {
        let ExitCode = Changelog_Generator::Run
        ( & Args[2..], | Msg | Logger.Println(Msg), | ErrMsg | Logger.Eprintln(ErrMsg),);
        std::process::exit(ExitCode);
    }

    if Args.len() <= 1
    {
        if io::stdin().is_terminal()
        {
            Logger.Print("No target files specified. Format all files in current directory and subdirectories? [Y/N]: ");
            let mut Input = String:: new ();
            if io::stdin().read_line( & mut Input).is_ok()
            {
                let Trimmed = Input.trim().to_lowercase();
                if matches!(Trimmed.as_str(), "y" | "yes") { FormatAll = true; AutoYes = true; RawTargets.push(".".to_string()); }
                else { Logger.Set_Disable_Logs(true); PrintUsage( & Logger); return ; }
            }
            else { Logger.Set_Disable_Logs(true); PrintUsage( & Logger); return ; }
        }
        else { ReadFromStdin = true; }
    }

    let mut I = 1;
    while I < Args.len()
    {
        let Arg = & Args[I];
        match Arg.as_str()
        {
            "format" | "apply" | "run" => {}
            "check" => { CheckOnly = true; }
            "failed" | "--failed" => { ReformatFailed = true; }
            "--no-log" => { Logger.Set_Disable_Logs(true); }
            "all" | "-a" | "--all" => { FormatAll = true; }
            "-f" | "--file" | "--files" | "--only-file" | "--only-files" | "-d" | "--dir" | "--directory" | "--folder" | "--folders" | "--only-folder" | "--only-folders" =>
            {
                I += 1;
                if I < Args.len()
                {
                    for Part in Args[I].split(',')
                    {
                        let Trimmed = Part.trim();
                        if ! Trimmed.is_empty() { SpecifiedFiles.push(Trimmed.to_string()); }
                    }
                }
                while I + 1 < Args.len() && ! Args[I + 1].starts_with('-')
                {
                    let NextArg = & Args[I + 1];
                    if matches!(NextArg.as_str(), "format" | "apply" | "run" | "failed" | "all" | "check") { break ; }
                    I += 1;
                    for Part in Args[I].split(',')
                    {
                        let Trimmed = Part.trim();
                        if ! Trimmed.is_empty() { SpecifiedFiles.push(Trimmed.to_string()); }
                    }
                }
            }
            "-e" | "--ext" | "--extension" | "--include-ext" =>
            {
                I += 1;
                if I < Args.len()
                {
                    for Part in Args[I].split(',')
                    {
                        let Clean = Part.trim().trim_start_matches('.').to_lowercase();
                        if ! Clean.is_empty() { AllowedExtensions.push(Clean); }
                    }
                }
            }
            "--all-types" | "--any-type" => { AllTypes = true; }
            "-x" | "--exclude" | "--ignore" =>
            {
                I += 1;
                if I < Args.len()
                {
                    for Part in Args[I].split(',')
                    {
                        let Trimmed = Part.trim();
                        if ! Trimmed.is_empty() { ExcludedPatterns.push(Trimmed.to_string()); }
                    }
                }
            }
            "-y" | "--yes" => { AutoYes = true; }
            "--verbose" => { Verbose = true; }
            "-j" | "--threads" =>
            {
                I += 1;
                if I < Args.len()
                {
                    if let Ok(Count) = Args[I].parse::<usize>() { if Count > 0 { CustomThreads = Some(Count); } }
                }
            }
            "-w" | "--write" => { OutputToStdout = false; }
            "-o" | "--output" | "--out" | "--out-dir" | "--output-dir" =>
            {
                I += 1;
                if I < Args.len() && ! Args[I].starts_with('-') && ! matches!(Args[I].as_str(), "format" | "apply" | "run" | "failed" | "all" | "check")
                {
                    OutputDest = Some(Args[I].clone());
                }
                else
                { if OutputSuffix.is_none() && OutputPrefix.is_none() { OutputSuffix = Some(".formatted".to_string()); } I -= 1; }
            }
            "--prefix" =>
            {
                I += 1;
                if I < Args.len() && ! Args[I].starts_with('-') && ! matches!(Args[I].as_str(), "format" | "apply" | "run" | "failed" | "all" | "check")
                {
                    OutputPrefix = Some(Args[I].clone());
                }
                else
                { OutputPrefix = Some("formatted_".to_string()); I -= 1; }
            }
            "--suffix" =>
            {
                I += 1;
                if I < Args.len() && ! Args[I].starts_with('-') && ! matches!(Args[I].as_str(), "format" | "apply" | "run" | "failed" | "all" | "check")
                {
                    OutputSuffix = Some(Args[I].clone());
                }
                else
                { OutputSuffix = Some(".formatted".to_string()); I -= 1; }
            }
            "--stdout" => { OutputToStdout = true; }
            "--output-failed" | "-of" => { I += 1; if I < Args.len() { OutputFailedPath = Some(Args[I].clone()); } }
            "--check" => { CheckOnly = true; }
            "-c" | "--canvas-width" | "--width" =>
            {
                I += 1;
                if I < Args.len() { if let Ok(W) = Args[I].parse::<usize>() { Config.Canvas_Width = W; } }
            }
            "--max-statements" =>
            {
                I += 1;
                if I < Args.len() { if let Ok(M) = Args[I].parse::<usize>() { Config.Max_Statements_In_Monolith = M; } }
            }
            "--unwrap-paragraphs" | "--no-abrupt-breaks" | "--reflow-markdown" =>
            {
                I += 1;
                if I < Args.len() && ! Args[I].starts_with('-') && (Args[I] == "0" || Args[I].eq_ignore_ascii_case("false"))
                {
                    Config.Unwrap_Markdown_Paragraphs = false;
                }
                else
                {
                    Config.Unwrap_Markdown_Paragraphs = true;
                    if I < Args.len() && (Args[I] == "1" || Args[I].eq_ignore_ascii_case("true")) {}
                    else { I -= 1; }
                }
            }
            "--no-unwrap-paragraphs" =>
            { Config.Unwrap_Markdown_Paragraphs = false; }
            "-i" | "--indent" =>
            {
                I += 1;
                if I < Args.len() { if let Ok(S) = Args[I].parse::<usize>() { Config.Indent_Width = S; } }
            }
            "-t" | "--tabs" => { Config.Use_Tabs = true; }
            "-l" | "--language" =>
            {
                I += 1;
                if I < Args.len()
                {
                    let LangStr = Args[I].to_lowercase();
                    Config.Language = match LangStr.as_str()
                    {
                        "c" => Source_Language::C, "cpp" | "c++" => Source_Language::Cpp, "cs" | "csharp" | "c#" => Source_Language::CSharp, "js" | "javascript" => Source_Language::JavaScript, "ts" | "typescript" => Source_Language::TypeScript, "jsx" => Source_Language::Jsx, "tsx" => Source_Language::Tsx, "sh" | "shell" | "bash" | "zsh" | "ksh" => Source_Language::Shell, "go" => Source_Language::Go, "php" | "phtml" => Source_Language::Php, "polyglot" | "html" | "htm" | "xhtml" | "vue" | "svelte" => Source_Language::Html, "css" | "scss" | "less" => Source_Language::Css, "rs" | "rust" => Source_Language::Rust, "bat" | "cmd" => Source_Language::Batch, "toml" => Source_Language::Toml, "cargo-lock" | "cargolock" | "lock" => Source_Language::CargoLock, "dart" | "flutter" => Source_Language::Dart, "md" | "markdown" => Source_Language::Markdown, "typ" | "typst" => Source_Language::Typst, "kt" | "kotlin" | "kotlin-allman" | "kt-allman" | "kotlin-v1" | "kt-v1" =>
                        {
                            Config.Kotlin_Style = Kotlin_Style::Allman;
                            Source_Language::Kotlin
                        },
                        "kt-official" | "kotlin-official" | "kotlin-v2" | "kt-v2" =>
                        {
                            Config.Kotlin_Style = Kotlin_Style::Official;
                            Source_Language::KotlinOfficial
                        },
                        _ => Source_Language::Generic,
                    };
                }
            }
            "--kotlin-version" | "-kv" =>
            {
                I += 1;
                if I < Args.len()
                {
                    if Args[I] == "2" || Args[I].eq_ignore_ascii_case("v2") || Args[I].eq_ignore_ascii_case("official")
                    {
                        Config.Kotlin_Style = Kotlin_Style::Official;
                    }
                    else { Config.Kotlin_Style = Kotlin_Style::Allman; }
                }
            }
            "--kotlin-style" | "-ks" =>
            {
                I += 1;
                if I < Args.len()
                {
                    if Args[I].eq_ignore_ascii_case("official") || Args[I] == "2" { Config.Kotlin_Style = Kotlin_Style::Official; }
                    else { Config.Kotlin_Style = Kotlin_Style::Allman; }
                }
            }
            "--allman-kotlin" => { Config.Kotlin_Style = Kotlin_Style::Allman; }
            "--official-kotlin" => { Config.Kotlin_Style = Kotlin_Style::Official; }
            "--cah" =>
            {
                I += 1;
                if I < Args.len()
                {
                    let Val = Args[I].as_str();
                    if Val == "-1" || Val == "0" || Val.eq_ignore_ascii_case("false") || Val.eq_ignore_ascii_case("off")
                    {
                        Config.Column_Anchored_Hybrid = false;
                    }
                    else if Val == "1" || Val.eq_ignore_ascii_case("true") || Val.eq_ignore_ascii_case("on")
                    { Config.Column_Anchored_Hybrid = true; }
                    else
                    { Config.Column_Anchored_Hybrid = true; I -= 1; }
                }
                else
                { Config.Column_Anchored_Hybrid = true; }
            }
            "--preserve-empty-lines" =>
            {
                I += 1;
                if I < Args.len() && ! Args[I].starts_with('-')
                {
                    Config.Preserve_Empty_Lines = Args[I] != "0" && ! Args[I].eq_ignore_ascii_case("false");
                }
                else
                { Config.Preserve_Empty_Lines = true; I -= 1; }
            }
            "--no-preserve-empty-lines" =>
            { Config.Preserve_Empty_Lines = false; }
            "--max-blank-lines" | "-mbl" =>
            {
                I += 1;
                if I < Args.len()
                { if let Ok(Val) = Args[I].parse::<usize>() { Config.Max_Blank_Lines = Val; } }
            }
            "--stdin-filepath" => { I += 1; if I < Args.len() { StdinVirtualPath = Some(Args[I].clone()); } }
            "-" => { ReadFromStdin = true; }
            "-h" | "--help" => { Logger.Set_Disable_Logs(true); PrintUsage( & Logger); return ; }
            "-v" | "--version" =>
            {
                Logger.Set_Disable_Logs(true);
                Logger.Println("lafs 0.1.0 (Language-Agnostic Formatting Style Formatter)");
                return ;
            }
            _ =>
            {
                if Arg.starts_with("--file=") || Arg.starts_with("--files=") || Arg.starts_with("-f=") || Arg.starts_with("--only-file=") || Arg.starts_with("--only-files=")
                || Arg.starts_with("--dir=") || Arg.starts_with("--directory=") || Arg.starts_with("-d=") || Arg.starts_with("--folder=") || Arg.starts_with("--folders=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        for Part in Value.split(',')
                        {
                            let Trimmed = Part.trim();
                            if ! Trimmed.is_empty() { SpecifiedFiles.push(Trimmed.to_string()); }
                        }
                    }
                }
                else if Arg.starts_with("--ext=") || Arg.starts_with("--extension=") || Arg.starts_with("-e=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        for Part in Value.split(',')
                        {
                            let Clean = Part.trim().trim_start_matches('.').to_lowercase();
                            if ! Clean.is_empty() { AllowedExtensions.push(Clean); }
                        }
                    }
                }
                else if Arg.starts_with("--exclude=") || Arg.starts_with("-x=") || Arg.starts_with("--ignore=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        for Part in Value.split(',')
                        {
                            let Trimmed = Part.trim();
                            if ! Trimmed.is_empty() { ExcludedPatterns.push(Trimmed.to_string()); }
                        }
                    }
                }
                else if Arg.starts_with("--kotlin-version=") || Arg.starts_with("-kv=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        if Value == "2" || Value.eq_ignore_ascii_case("v2") || Value.eq_ignore_ascii_case("official")
                        {
                            Config.Kotlin_Style = Kotlin_Style::Official;
                        }
                        else { Config.Kotlin_Style = Kotlin_Style::Allman; }
                    }
                }
                else if Arg.starts_with("--kotlin-style=") || Arg.starts_with("-ks=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        if Value.eq_ignore_ascii_case("official") || Value == "2" { Config.Kotlin_Style = Kotlin_Style::Official; }
                        else { Config.Kotlin_Style = Kotlin_Style::Allman; }
                    }
                }
                else if Arg.starts_with("--cah=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        Config.Column_Anchored_Hybrid = Value != "0" && Value != "-1" && ! Value.eq_ignore_ascii_case("false") && ! Value.eq_ignore_ascii_case("off");
                    }
                }
                else if Arg.starts_with("--preserve-empty-lines=")
                { if let Some((_, Value)) = Arg.split_once('=') { Config.Preserve_Empty_Lines = Value != "0" && ! Value.eq_ignore_ascii_case("false"); } }
                else if Arg.starts_with("--max-blank-lines=") || Arg.starts_with("-mbl=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    { if let Ok(Val) = Value.parse::<usize>() { Config.Max_Blank_Lines = Val; } }
                }
                else if Arg.starts_with("--output=") || Arg.starts_with("--out=") || Arg.starts_with("--out-dir=") || Arg.starts_with("--output-dir=") || Arg.starts_with("-o=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        let Trimmed = Value.trim();
                        if ! Trimmed.is_empty()
                        { OutputDest = Some(Trimmed.to_string()); }
                        else if OutputSuffix.is_none() && OutputPrefix.is_none()
                        { OutputSuffix = Some(".formatted".to_string()); }
                    }
                }
                else if Arg.starts_with("--prefix=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        let Trimmed = Value.trim();
                        OutputPrefix = Some(if Trimmed.is_empty() { "formatted_".to_string() } else { Trimmed.to_string() });
                    }
                }
                else if Arg.starts_with("--suffix=")
                {
                    if let Some((_, Value)) = Arg.split_once('=')
                    {
                        let Trimmed = Value.trim();
                        OutputSuffix = Some(if Trimmed.is_empty() { ".formatted".to_string() } else { Trimmed.to_string() });
                    }
                }
                else if Arg.starts_with("--unwrap-paragraphs=") || Arg.starts_with("--no-abrupt-breaks=")
                { if let Some((_, Value)) = Arg.split_once('=') { Config.Unwrap_Markdown_Paragraphs = Value != "0" && ! Value.eq_ignore_ascii_case("false"); } }
                else if ! Arg.starts_with('-') { RawTargets.push(Arg.clone()); }
            }
        }
        I += 1;
    }

    SpecifiedFiles.sort();
    SpecifiedFiles.dedup();

    if OutputToStdout || ReadFromStdin { Logger.Set_Disable_Logs(true); }

    if ReformatFailed
    {
        AutoYes = true;
        let BinaryFailedPath = GetDefaultFailedLogPath();
        let FailedLogPath = OutputFailedPath.clone().unwrap_or_else
        (
            ||
            {
                if Path:: new ("failed.txt").exists() { "failed.txt".to_string() }
                else if Path:: new ( & BinaryFailedPath).exists() { BinaryFailedPath.clone() }
                else { "failed.txt".to_string() }
            }
        );
        let ContentResult = fs::read_to_string( & FailedLogPath).or_else
        (
            | _ |
            {
                if FailedLogPath != "failed.txt" && Path:: new ("failed.txt").exists() { fs::read_to_string("failed.txt") }
                else if FailedLogPath != BinaryFailedPath && Path:: new ( & BinaryFailedPath).exists() { fs::read_to_string( & BinaryFailedPath) }
                else { Err(io::Error::new (io::ErrorKind::NotFound, "not found")) }
            }
        );
        match ContentResult
        {
            Ok(Content) =>
            {
                let mut FoundCount = 0;
                for Line in Content.lines()
                { if let Some(CleanPath) = ParseFailedLine(Line) { RawTargets.push(CleanPath.to_string()); FoundCount += 1; } }
                if FoundCount == 0 { Logger.Println( & format ! ("No failed files listed in '{}'.", FailedLogPath)); Logger.Flush_To_Files(); return ; }
                OutputFailedPath = Some(FailedLogPath);
            }
            Err(_) => { Logger.Eprintln( & format ! ("Failed files list '{}' not found.", FailedLogPath)); Logger.Flush_To_Files(); return ; }
        }
    }

    // If a language was specified and no explicit extensions were provided, restrict to that language's extensions
    if Config.Language != Source_Language::Generic && AllowedExtensions.is_empty() { for Ext in Config.Language.DefaultExtensions() { AllowedExtensions.push(Ext.to_string()); } }

    AllowedExtensions.sort();
    AllowedExtensions.dedup();
    // Default to current directory if --all is explicitly specified without paths
    if FormatAll && RawTargets.is_empty() && SpecifiedFiles.is_empty() { RawTargets.push(".".to_string()); }

    if RawTargets.is_empty() && SpecifiedFiles.is_empty() && ! FormatAll && ! ReadFromStdin
    {
        if io::stdin().is_terminal()
        {
            let LangTag = if Config.Language != Source_Language::Generic { format ! ("{{{}}} ", Config.Language.ShortName()) }
            else if ! AllowedExtensions.is_empty() { format ! ("{{{}}} ", AllowedExtensions.join(", ")) } else { String::new () };

            Logger.Print( & format ! ("No target files specified. Format all {}files in current directory and subdirectories? [Y/N]: ", LangTag));
            let mut Input = String:: new ();
            if io::stdin().read_line( & mut Input).is_ok()
            {
                let Trimmed = Input.trim().to_lowercase();
                if matches!(Trimmed.as_str(), "y" | "yes") { FormatAll = true; AutoYes = true; RawTargets.push(".".to_string()); }
                else { Logger.Set_Disable_Logs(true); PrintUsage( & Logger); return ; }
            }
            else { Logger.Set_Disable_Logs(true); PrintUsage( & Logger); return ; }
        }
        else { ReadFromStdin = true; }
    }

    if ReadFromStdin
    {
        // Format from stdin
        let mut Buffer = String:: new ();
        if io::stdin().read_to_string( & mut Buffer).is_ok()
        {
            if let Some(VirtPath) = StdinVirtualPath { Config.Language = Source_Language::FromExtension( & VirtPath); }

            match Source_Formatter::Format( & Buffer, & Config)
            {
                Ok(Formatted) =>
                {
                    if CheckOnly { if Formatted != Buffer { std::process::exit(1); } } else { print ! ("{}", Formatted); }
                }
                Err(ErrMessage) => { eprintln ! ("Error formatting stdin: {}", ErrMessage); std::process::exit(1); }
            }
        }
        return ;
    }

    let FormattedCount = Arc:: new (AtomicUsize::new (0));
    let CleanCount = Arc:: new (AtomicUsize::new (0));
    let ErrorCount = Arc:: new (AtomicUsize::new (0));
    let CurrentIndex = Arc:: new (AtomicUsize::new (0));
    let FailedPaths = Arc:: new (Mutex::new (Vec::new ()));

    // Expand targets to file list
    let mut TargetFiles: Vec<String> = Vec:: new ();

    if ! SpecifiedFiles.is_empty()
    {
        let mut Candidates: Vec<String> = Vec:: new ();
        if RawTargets.is_empty() && ! FormatAll
        {
            for Spec in & SpecifiedFiles
            {
                let (CleanSpec, HadTrailingSlash) = NormalizePathSpec(Spec);
                let P = Path:: new ( & CleanSpec);

                if P.is_dir()
                {
                    if HadTrailingSlash { CollectFilesShallow(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut Candidates); }
                    else { CollectFilesRecursively(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut Candidates); }
                }
                else if P.is_file()
                {
                    if IsSpecifiedFileEligible(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, Config.Language)
                    {
                        Candidates.push(P.to_string_lossy().to_string());
                    }
                    else if IsBinaryFile(P) { Logger.Eprintln( & format ! ("Skipped binary file: {}", Spec)); }
                }
                else if CleanSpec.contains('*') || CleanSpec.contains('?')
                {
                    ExpandGlobPattern( & CleanSpec, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut Candidates);
                }
                else
                {
                    let mut SearchPool: Vec<String> = Vec:: new ();
                    CollectFilesRecursively(Path::new ("."), AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut SearchPool);
                    let mut Found = false;
                    for PoolPath in SearchPool { if MatchesSpecifiedTarget(Path::new ( & PoolPath), Spec) { Candidates.push(PoolPath); Found = true; } }
                    if ! Found
                    {
                        Logger.Eprintln( & format ! ("Specified file or directory not found: {}", Spec));
                        ErrorCount.fetch_add(1, Ordering::Relaxed);
                        FailedPaths.lock().unwrap().push(format ! ("{} (Not Found) Reason: Specified file or directory not found", Spec));
                    }
                }
            }
        }
        else
        {
            let mut ScannedFiles: Vec<String> = Vec:: new ();
            for Target in & RawTargets
            {
                if Target.contains('*') || Target.contains('?')
                {
                    ExpandGlobPattern(Target, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut ScannedFiles);
                }
                else
                {
                    let (CleanTarget, TargetHadTrailingSlash) = NormalizePathSpec(Target);
                    let P = Path:: new ( & CleanTarget);
                    if P.is_dir()
                    {
                        if TargetHadTrailingSlash { CollectFilesShallow(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut ScannedFiles); }
                        else { CollectFilesRecursively(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut ScannedFiles); }
                    }
                    else if P.is_file() && IsSpecifiedFileEligible(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, Config.Language)
                    {
                        ScannedFiles.push(P.to_string_lossy().to_string());
                    }
                }
            }
            if FormatAll && RawTargets.is_empty()
            {
                CollectFilesRecursively(Path::new ("."), AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut ScannedFiles);
            }

            for Scanned in ScannedFiles { if SpecifiedFiles.iter().any( | Spec | MatchesSpecifiedTarget(Path::new ( & Scanned), Spec)) { Candidates.push(Scanned); } }

            for Spec in & SpecifiedFiles
            {
                let WasMatched = Candidates.iter().any( | C | MatchesSpecifiedTarget(Path::new (C), Spec));
                if ! WasMatched
                {
                    let (CleanSpec, HadTrailingSlash) = NormalizePathSpec(Spec);
                    let P = Path:: new ( & CleanSpec);
                    if P.is_dir()
                    {
                        if HadTrailingSlash { CollectFilesShallow(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut Candidates); }
                        else { CollectFilesRecursively(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut Candidates); }
                    }
                    else if P.is_file() && IsSpecifiedFileEligible(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, Config.Language)
                    {
                        Candidates.push(P.to_string_lossy().to_string());
                    }
                    else
                    {
                        Logger.Eprintln( & format ! ("Specified file or directory not found in targets: {}", Spec));
                        ErrorCount.fetch_add(1, Ordering::Relaxed);
                        FailedPaths.lock().unwrap().push(format ! ("{} (Not Found) Reason: Specified file or directory not found in targets", Spec));
                    }
                }
            }
        }

        TargetFiles = Candidates;
    }
    else
    {
        for Target in & RawTargets
        {
            if Target.contains('*') || Target.contains('?')
            {
                ExpandGlobPattern(Target, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut TargetFiles);
            }
            else
            {
                let (CleanTarget, HadTrailingSlash) = NormalizePathSpec(Target);
                let P = Path:: new ( & CleanTarget);
                if P.is_dir()
                {
                    if HadTrailingSlash { CollectFilesShallow(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut TargetFiles); }
                    else { CollectFilesRecursively(P, AllTypes, & AllowedExtensions, & ExcludedPatterns, & mut TargetFiles); }
                }
                else if P.is_file()
                {
                    if ReformatFailed || IsFileEligible(P, AllTypes, & AllowedExtensions, & ExcludedPatterns)
                    {
                        TargetFiles.push(P.to_string_lossy().to_string());
                    }
                }
                else { Logger.Eprintln( & format ! ("File or directory not found: {}", Target)); }
            }
        }
    }

    // Remove any duplicate file paths
    TargetFiles.sort();
    TargetFiles.dedup();

    let TotalFiles = TargetFiles.len();
    if TotalFiles == 0 { Logger.Println("No matching files found to format."); Logger.Flush_To_Files(); return ; }
    // Prompt user confirmation if formatting directory or multiple files
    let HasCustomOutput = OutputDest.is_some() || OutputPrefix.is_some() || OutputSuffix.is_some();
    let NeedsPrompt = (FormatAll || RawTargets.iter().any( | T | { let (CleanT, _) = NormalizePathSpec(T); CleanT == "." || Path::new ( & CleanT).is_dir() })) && SpecifiedFiles.is_empty() && ! AutoYes && ! CheckOnly && ! OutputToStdout;

    if NeedsPrompt
    {
        let TargetDesc = if RawTargets.len() == 1 { & RawTargets[0] } else { "selected targets" };
        let PromptDesc = if ! AllowedExtensions.is_empty() { format ! ("{} [ext: {}]", TargetDesc, AllowedExtensions.join(", ")) }
        else { TargetDesc.to_string() };

        if ! PromptConfirmation( & Logger, TotalFiles, & PromptDesc, HasCustomOutput)
        {
            Logger.Println("Operation cancelled by user.");
            Logger.Flush_To_Files();
            return ;
        }
    }

    if ! ReformatFailed && ! OutputToStdout
    {
        let TargetFile = OutputFailedPath.as_deref().unwrap_or("failed.txt");
        if Path:: new (TargetFile).exists() { let _ = fs::write(TargetFile, ""); }
        let BinaryFailed = GetDefaultFailedLogPath();
        if BinaryFailed != TargetFile && Path:: new ( & BinaryFailed).exists() { let _ = fs::write( & BinaryFailed, ""); }
    }

    let StartTime = Instant::now();
    let ThreadCount = if OutputToStdout || CheckOnly || TotalFiles <= 1 { 1 }
    else { CustomThreads.unwrap_or_else( || std::thread::available_parallelism().map_or(4, | n | n.get ())) };

    let IsSingleFile = TargetFiles.len() == 1;
    let EffectiveBaseDirs: Vec<String> = if ! RawTargets.is_empty() { RawTargets.clone() }
    else if ! SpecifiedFiles.is_empty() { SpecifiedFiles.clone() }
    else { vec ! [".".to_string()] };

    if ThreadCount <= 1
    {
        for FilePathStr in & TargetFiles
        {
            let FilePath = Path:: new (FilePathStr);
            let FileContent = match fs::read_to_string(FilePath)
            {
                Ok(C) => C, Err(E) =>
                {
                    if E.kind() == io::ErrorKind::InvalidData
                    {
                        if Verbose { Logger.Println( & format ! ("Skipped non-UTF8 / binary file: {}", FilePathStr)); }
                        continue;
                    }
                    Logger.Eprintln( & format ! ("Failed to read file {}: {}", FilePathStr, E));
                    ErrorCount.fetch_add(1, Ordering::Relaxed);
                    let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), & format ! ("Failed to read file: {}", E));
                    AppendFailedEntry( & Entry, & OutputFailedPath);
                    FailedPaths.lock().unwrap().push(Entry);
                    continue;
                }
            };

            let DestPath = ComputeOutputPath
            (FilePath, OutputDest.as_deref(), OutputPrefix.as_deref(), OutputSuffix.as_deref(), & EffectiveBaseDirs, IsSingleFile,);

            let mut FileConfig = Config.clone();
            if FileConfig.Language == Source_Language::Generic { FileConfig.Language = DetectLanguageForFile(FilePathStr, & FileContent); }

            match Source_Formatter::Format( & FileContent, & FileConfig)
            {
                Ok(Formatted) =>
                {
                    let IsAlreadyFormatted = Formatted == FileContent;

                    if CheckOnly
                    {
                        if ! IsAlreadyFormatted
                        {
                            Logger.Println( & format ! ("Requires formatting: {}", FilePathStr));
                            ErrorCount.fetch_add(1, Ordering::Relaxed);
                            let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), "Requires formatting");
                            AppendFailedEntry( & Entry, & OutputFailedPath);
                            FailedPaths.lock().unwrap().push(Entry);
                        }
                        else if Verbose { Logger.Println( & format ! ("Clean: {}", FilePathStr)); }
                    }
                    else if OutputToStdout { print ! ("{}", Formatted); }
                    else if DestPath != FilePath
                    {
                        if let Err(E) = SafelyWriteFile( & DestPath, & Formatted)
                        {
                            Logger.Eprintln( & format ! ("Failed to save output file {}: {}", DestPath.display(), E));
                            ErrorCount.fetch_add(1, Ordering::Relaxed);
                            let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), & format ! ("Failed to save output file: {}", E));
                            AppendFailedEntry( & Entry, & OutputFailedPath);
                            FailedPaths.lock().unwrap().push(Entry);
                        }
                        else
                        {
                            if IsAlreadyFormatted
                            {
                                Logger.Println( & format ! ("Clean (Written to output): {} -> {}", FilePathStr, DestPath.display()));
                            }
                            else
                            { Logger.Println( & format ! ("Formatted & Saved: {} -> {}", FilePathStr, DestPath.display())); }
                            FormattedCount.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    else if ! IsAlreadyFormatted
                    {
                        if let Err(E) = SafelyWriteFile(FilePath, & Formatted)
                        {
                            Logger.Eprintln( & format ! ("Failed to save file {}: {}", FilePathStr, E));
                            ErrorCount.fetch_add(1, Ordering::Relaxed);
                            let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), & format ! ("Failed to save file: {}", E));
                            AppendFailedEntry( & Entry, & OutputFailedPath);
                            FailedPaths.lock().unwrap().push(Entry);
                        }
                        else { Logger.Println( & format ! ("Formatted & Saved: {}", FilePathStr)); FormattedCount.fetch_add(1, Ordering::Relaxed); }
                    }
                    else
                    {
                        if Verbose { Logger.Println( & format ! ("Clean (Unchanged): {}", FilePathStr)); }
                        CleanCount.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Err(ErrMessage) =>
                {
                    Logger.Eprintln( & format ! ("Error formatting {}: {}", FilePathStr, ErrMessage));
                    ErrorCount.fetch_add(1, Ordering::Relaxed);
                    let CleanErr = ErrMessage.trim();
                    let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), CleanErr);
                    AppendFailedEntry( & Entry, & OutputFailedPath);
                    FailedPaths.lock().unwrap().push(Entry);
                }
            }
        }
    }
    else
    {
        std::thread::scope
        (
            | Scope |
            {
                for _ in 0..ThreadCount
                {
                    let WorkerLogger = Logger.clone();
                    let WorkerFailedPaths = FailedPaths.clone();
                    let WorkerFormattedCount = FormattedCount.clone();
                    let WorkerCleanCount = CleanCount.clone();
                    let WorkerErrorCount = ErrorCount.clone();
                    let WorkerCurrentIndex = CurrentIndex.clone();
                    let WorkerConfig = Config.clone();
                    let WorkerTargetFiles = & TargetFiles;
                    let WorkerOutputFailedPath = OutputFailedPath.clone();
                    let WorkerOutputDest = OutputDest.clone();
                    let WorkerOutputPrefix = OutputPrefix.clone();
                    let WorkerOutputSuffix = OutputSuffix.clone();
                    let WorkerBaseDirs = EffectiveBaseDirs.clone();
                    let WorkerIsSingleFile = IsSingleFile;

                    Scope.spawn
                    (
                        move || loop
                        {
                            let Idx = WorkerCurrentIndex.fetch_add(1, Ordering::Relaxed);
                            if Idx >= TotalFiles { break ; }
                            let FilePathStr = & WorkerTargetFiles[Idx];
                            let FilePath = Path:: new (FilePathStr);
                            let FileContent = match fs::read_to_string(FilePath)
                            {
                                Ok(C) => C, Err(E) =>
                                {
                                    if E.kind() == io::ErrorKind::InvalidData
                                    {
                                        if Verbose { WorkerLogger.Println( & format ! ("Skipped non-UTF8 / binary file: {}", FilePathStr)); }
                                        continue;
                                    }
                                    WorkerLogger.Eprintln( & format ! ("Failed to read file {}: {}", FilePathStr, E));
                                    WorkerErrorCount.fetch_add(1, Ordering::Relaxed);
                                    let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), & format ! ("Failed to read file: {}", E));
                                    AppendFailedEntry( & Entry, & WorkerOutputFailedPath);
                                    WorkerFailedPaths.lock().unwrap().push(Entry);
                                    continue;
                                }
                            };
                            let mut FileConfig = WorkerConfig.clone();
                            if FileConfig.Language == Source_Language::Generic { FileConfig.Language = DetectLanguageForFile(FilePathStr, & FileContent); }

                            let DestPath = ComputeOutputPath
                            (FilePath, WorkerOutputDest.as_deref(), WorkerOutputPrefix.as_deref(), WorkerOutputSuffix.as_deref(), & WorkerBaseDirs, WorkerIsSingleFile,);

                            match Source_Formatter::Format( & FileContent, & FileConfig)
                            {
                                Ok(Formatted) =>
                                {
                                    let IsAlreadyFormatted = Formatted == FileContent;
                                    if DestPath != FilePath
                                    {
                                        if let Err(E) = SafelyWriteFile( & DestPath, & Formatted)
                                        {
                                            WorkerLogger.Eprintln( & format ! ("Failed to save output file {}: {}", DestPath.display(), E));
                                            WorkerErrorCount.fetch_add(1, Ordering::Relaxed);
                                            let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), & format ! ("Failed to save output file: {}", E));
                                            AppendFailedEntry( & Entry, & WorkerOutputFailedPath);
                                            WorkerFailedPaths.lock().unwrap().push(Entry);
                                        }
                                        else
                                        {
                                            if IsAlreadyFormatted
                                            {
                                                WorkerLogger.Println( & format ! ("Clean (Written to output): {} -> {}", FilePathStr, DestPath.display()));
                                            }
                                            else
                                            {
                                                WorkerLogger.Println( & format ! ("Formatted & Saved: {} -> {}", FilePathStr, DestPath.display()));
                                            }
                                            WorkerFormattedCount.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                    else if ! IsAlreadyFormatted
                                    {
                                        if let Err(E) = SafelyWriteFile(FilePath, & Formatted)
                                        {
                                            WorkerLogger.Eprintln( & format ! ("Failed to save file {}: {}", FilePathStr, E));
                                            WorkerErrorCount.fetch_add(1, Ordering::Relaxed);
                                            let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), & format ! ("Failed to save file: {}", E));
                                            AppendFailedEntry( & Entry, & WorkerOutputFailedPath);
                                            WorkerFailedPaths.lock().unwrap().push(Entry);
                                        }
                                        else
                                        {
                                            WorkerLogger.Println( & format ! ("Formatted & Saved: {}", FilePathStr));
                                            WorkerFormattedCount.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                    else
                                    {
                                        if Verbose { WorkerLogger.Println( & format ! ("Clean (Unchanged): {}", FilePathStr)); }
                                        WorkerCleanCount.fetch_add(1, Ordering::Relaxed);
                                    }
                                }
                                Err(ErrMessage) =>
                                {
                                    WorkerLogger.Eprintln( & format ! ("Error formatting {}: {}", FilePathStr, ErrMessage));
                                    WorkerErrorCount.fetch_add(1, Ordering::Relaxed);
                                    let CleanErr = ErrMessage.trim();
                                    let Entry = FormatFailedEntry(FilePathStr, Source_Language::FromExtension(FilePathStr).ShortName(), CleanErr);
                                    AppendFailedEntry( & Entry, & WorkerOutputFailedPath);
                                    WorkerFailedPaths.lock().unwrap().push(Entry);
                                }
                            }
                        }
                    );
                }
            }
        );
    }

    let Elapsed = StartTime.elapsed();
    let FCount = FormattedCount.load(Ordering::Relaxed);
    let CCount = CleanCount.load(Ordering::Relaxed);
    let ECount = ErrorCount.load(Ordering::Relaxed);

    if ! OutputToStdout && ! CheckOnly
    {
        Logger.Println( & format ! ("Summary: {} formatted and saved, {} already clean, {} errors ({:.2?}, {} threads).", FCount, CCount, ECount, Elapsed, ThreadCount));
    }

    let DefaultBinaryFailed = GetDefaultFailedLogPath();
    if OutputFailedPath.is_none() && ECount > 0 && ! OutputToStdout { OutputFailedPath = Some("failed.txt".to_string()); }

    if let Some(FailedFile) = OutputFailedPath
    {
        let mut FailedPathsStr = String:: new ();
        let FailedMutex = FailedPaths.lock().unwrap();
        for Path in FailedMutex.iter() { FailedPathsStr.push_str(Path); FailedPathsStr.push('\n'); }
        if let Err(E) = fs::write( & FailedFile, & FailedPathsStr)
        {
            Logger.Eprintln( & format ! ("Failed to write failed paths to {}: {}", FailedFile, E));
        }
        else if ReformatFailed && FailedMutex.is_empty()
        {
            Logger.Println("All previously failed files were successfully formatted!");
            if DefaultBinaryFailed != FailedFile && Path:: new ( & DefaultBinaryFailed).exists() { let _ = fs::write( & DefaultBinaryFailed, ""); }
        }
        else if ! FailedMutex.is_empty()
        {
            Logger.Println( & format ! ("Failed file paths recorded to '{}'. Use 'lafs format failed' to retry.", FailedFile));
            if DefaultBinaryFailed != FailedFile
            { let _ = fs::write( & DefaultBinaryFailed, & FailedPathsStr); }
        }
    }

    Logger.Flush_To_Files();

    if CheckOnly && ECount > 0 { std::process::exit(1); }
}

fn main() { Main(); }

#[cfg(test)]
mod Tests
{
    use super:: * ;
    use std::path:: { Path, PathBuf };

   #[test] fn Test_Wildcard_Matching()
    {
        assert!(WildcardMatch("*.c", "main.c"));
        assert!(WildcardMatch("*.cpp", "test.cpp"));
        assert!(WildcardMatch("test_*.rs", "test_lexer.rs"));
        assert!( ! WildcardMatch("*.c", "main.rs"));
        assert!(WildcardMatch("vendor", "vendor"));
        assert!(WildcardMatch("target*", "target_output"));
        assert!(WildcardMatch("?at", "cat"));
        assert!(WildcardMatch("?at", "bat"));
        assert!( ! WildcardMatch("?at", "chat"));
    }

   #[test] fn Test_Document_Extension_Filter()
    {
        let EmptyExts: Vec<String> = Vec:: new ();
        let Excludes: Vec<String> = Vec:: new ();
        assert!( ! IsFileEligible(Path::new ("doc.adoc"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("doc.asciidoc"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("paper.pdf"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("notes.docx"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("notes.doc"), true, & EmptyExts, & Excludes));
        assert!(IsFileEligible(Path::new ("README.md"), true, & EmptyExts, & Excludes));
        assert!(IsFileEligible(Path::new ("guide.markdown"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("manual.rst"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("todo.txt"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("workflow.yml"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("config.yaml"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("workflow.yml"), false, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("config.yaml"), false, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("lafs.log"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("session 2026-11-09.log"), false, & EmptyExts, & Excludes));
    }

   #[test] fn Test_Binary_Extension_Filter()
    {
        let EmptyExts: Vec<String> = Vec:: new ();
        let Excludes: Vec<String> = Vec:: new ();
        assert!( ! IsFileEligible(Path::new ("app.exe"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("lib.dll"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("archive.zip"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("photo.png"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("video.mp4"), true, & EmptyExts, & Excludes));
    }

   #[test] fn Test_Python_Extension_Filter()
    {
        let EmptyExts: Vec<String> = Vec:: new ();
        let Excludes: Vec<String> = Vec:: new ();
        assert!( ! IsFileEligible(Path::new ("script.py"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("module.pyw"), true, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("types.pyi"), false, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("ext.pyx"), false, & EmptyExts, & Excludes));
        assert!( ! IsFileEligible(Path::new ("def.pxd"), false, & EmptyExts, & Excludes));
        assert!( ! IsSpecifiedFileEligible(Path::new ("publish.py"), false, & EmptyExts, & Excludes, Source_Language::Generic));
        assert!( ! IsSpecifiedFileEligible(Path::new ("setup.py"), true, & EmptyExts, & Excludes, Source_Language::Generic));
    }

   #[test] fn Test_Directory_Exclusion()
    {
        let CustomExcludes = vec!["vendor".to_string(), "build*".to_string(), "cache".to_string(),];
        // Default exclusions
        assert!(IsDirExcluded(".git", Path::new (".git"), & CustomExcludes));
        assert!(IsDirExcluded("target", Path::new ("target"), & CustomExcludes));
        assert!(IsDirExcluded("node_modules", Path::new ("node_modules"), & CustomExcludes));
        assert!(IsDirExcluded("old.src(unclear)", Path::new ("old.src(unclear)"), & CustomExcludes));
        assert!(IsDirExcluded("old.src", Path::new ("deep/nested/old.src"), & CustomExcludes));
        // Custom exclusions
        assert!(IsDirExcluded("vendor", Path::new ("vendor"), & CustomExcludes));
        assert!(IsDirExcluded("build_debug", Path::new ("build_debug"), & CustomExcludes));
        assert!(IsDirExcluded("cache", Path::new ("deeply/nested/cache"), & CustomExcludes));
        // Non-excluded
        assert!( ! IsDirExcluded("src", Path::new ("src"), & CustomExcludes));
        assert!( ! IsDirExcluded("examples", Path::new ("lafs/examples"), & CustomExcludes));
    }

   #[test] fn Test_File_Exclusion_Patterns()
    {
        let EmptyExts: Vec<String> = Vec:: new ();
        let CustomExcludes = vec!["*.min.js".to_string(), "generated_*".to_string(), "secret.key".to_string(),];

        assert!( ! IsFileEligible(Path::new ("bundle.min.js"), true, & EmptyExts, & CustomExcludes));
        assert!( ! IsFileEligible(Path::new ("generated_code.c"), true, & EmptyExts, & CustomExcludes));
        assert!( ! IsFileEligible(Path::new ("secret.key"), true, & EmptyExts, & CustomExcludes));
        // Allowed
        assert!(IsFileEligible(Path::new ("bundle.js"), true, & EmptyExts, & CustomExcludes));
        assert!(IsFileEligible(Path::new ("normal_code.c"), false, & EmptyExts, & CustomExcludes));
    }

   #[test] fn Test_Allowed_Extension_Filter()
    {
        let EmptyExcludes: Vec<String> = Vec:: new ();
        let AllowedC = vec!["c".to_string(), "h".to_string()];
        let AllowedSh = vec!["sh".to_string()];

        assert!(IsFileEligible(Path::new ("main.c"), false, & AllowedC, & EmptyExcludes));
        assert!(IsFileEligible(Path::new ("header.h"), false, & AllowedC, & EmptyExcludes));
        assert!( ! IsFileEligible(Path::new ("source.cpp"), false, & AllowedC, & EmptyExcludes));
        assert!( ! IsFileEligible(Path::new ("script.sh"), false, & AllowedC, & EmptyExcludes));
        assert!( ! IsFileEligible(Path::new ("app.cs"), false, & AllowedC, & EmptyExcludes));
        assert!( ! IsFileEligible(Path::new ("workflow.yml"), false, & AllowedC, & EmptyExcludes));

        assert!(IsFileEligible(Path::new ("deploy.sh"), false, & AllowedSh, & EmptyExcludes));
        assert!( ! IsFileEligible(Path::new ("main.c"), false, & AllowedSh, & EmptyExcludes));
    }

   #[test] fn Test_Matches_Specified_File()
    {
        assert!(MatchesSpecifiedFile(Path::new ("examples/sample.c"), "examples/sample.c"));
        assert!(MatchesSpecifiedFile(Path::new ("examples\\sample.c"), "examples/sample.c"));
        assert!(MatchesSpecifiedFile(Path::new ("examples/sample.c"), "sample.c"));
        assert!(MatchesSpecifiedFile(Path::new ("src/main.rs"), "main.rs"));
        assert!(MatchesSpecifiedFile(Path::new ("examples/sample.c"), "*.c"));
        assert!(MatchesSpecifiedFile(Path::new ("examples/sample.cpp"), "*.cpp"));
        assert!( ! MatchesSpecifiedFile(Path::new ("examples/sample.cpp"), "sample.c"));
        assert!( ! MatchesSpecifiedFile(Path::new ("src/main.rs"), "lexer.rs"));
    }

   #[test] fn Test_Specified_File_Eligibility()
    {
        let EmptyExts: Vec<String> = Vec:: new ();
        let Excludes: Vec<String> = Vec:: new ();
        assert!(IsSpecifiedFileEligible(Path::new ("sample.c"), false, & EmptyExts, & Excludes, Source_Language::Generic));
        assert!(IsSpecifiedFileEligible(Path::new ("sample.cpp"), false, & EmptyExts, & Excludes, Source_Language::Generic));
        assert!(IsSpecifiedFileEligible(Path::new ("sample.cs"), false, & EmptyExts, & Excludes, Source_Language::Generic));
        assert!(IsSpecifiedFileEligible(Path::new ("sample.typ"), false, & EmptyExts, & Excludes, Source_Language::Generic));
        assert!(IsSpecifiedFileEligible(Path::new ("sample.custom"), false, & EmptyExts, & Excludes, Source_Language::C));
        assert!( ! IsSpecifiedFileEligible(Path::new ("app.exe"), false, & EmptyExts, & Excludes, Source_Language::Generic));
    }

   #[test]
    fn Test_Normalize_Path_Spec()
    {
        assert_eq!(NormalizePathSpec("E:\\Typst\\scratch\\"),("E:\\Typst\\scratch".to_string(), true));
        assert_eq!(NormalizePathSpec("E:\\Typst\\scratch/"),("E:\\Typst\\scratch".to_string(), true));
        assert_eq!(NormalizePathSpec("E:\\Typst\\scratch\""),("E:\\Typst\\scratch".to_string(), true));
        assert_eq!(NormalizePathSpec("E:\\Typst\\scratch"),("E:\\Typst\\scratch".to_string(), false));
        assert_eq!(NormalizePathSpec("scratch/"),("scratch".to_string(), true));
        assert_eq!(NormalizePathSpec("scratch\\"),("scratch".to_string(), true));
        assert_eq!(NormalizePathSpec("scratch"),("scratch".to_string(), false));
        assert_eq!(NormalizePathSpec("sample.c"),("sample.c".to_string(), false));
    }

   #[test]
    fn Test_Matches_Specified_Target()
    {
        // Trailing slash (shallow): matches direct child only
        assert!(MatchesSpecifiedTarget(Path::new ("scratch/test.js"), "scratch/"));
        assert!(MatchesSpecifiedTarget(Path::new ("scratch\\test.js"), "scratch\\"));
        assert!( ! MatchesSpecifiedTarget(Path::new ("scratch/nested/test.js"), "scratch/"));
        assert!( ! MatchesSpecifiedTarget(Path::new ("scratch\\nested\\test.js"), "scratch\\"));

        // Direct file matching
        assert!(MatchesSpecifiedTarget(Path::new ("src/main.rs"), "main.rs"));
        assert!(MatchesSpecifiedTarget(Path::new ("src/main.rs"), "src/main.rs"));
    }

   #[test]
    fn Test_Collect_Files_Shallow_Vs_Recursively()
    {
        let TempBase = std::env::temp_dir().join(format ! ("lafs_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let NestedDir = TempBase.join("nested");
        fs::create_dir_all( & NestedDir).unwrap();

        let RootFile = TempBase.join("root.js");
        let NestedFile = NestedDir.join("inner.js");
        fs::write( & RootFile, "const A = 1;\n").unwrap();
        fs::write( & NestedFile, "const B = 2;\n").unwrap();

        let EmptyExts: Vec<String> = Vec:: new ();
        let Excludes: Vec<String> = Vec:: new ();

        let mut ShallowList = Vec:: new ();
        CollectFilesShallow( & TempBase, false, & EmptyExts, & Excludes, & mut ShallowList);
        assert_eq!(ShallowList.len(), 1, "Shallow collection must only gather files directly in folder");

        let mut RecursiveList = Vec:: new ();
        CollectFilesRecursively( & TempBase, false, & EmptyExts, & Excludes, & mut RecursiveList);
        assert_eq!(RecursiveList.len(), 2, "Recursive collection must gather root and nested files");

        let _ = fs::remove_dir_all( & TempBase);
    }

   #[test]
    fn Test_Parse_Failed_Line()
    {
        assert_eq!(ParseFailedLine("C:\\path\\cross_compile.rs (rs) Reason: Safety Gate Failed: Token mismatch"), Some("C:\\path\\cross_compile.rs"));
        assert_eq!(ParseFailedLine("C:\\path\\cross_compile.rs Reason: Safety Gate Failed: Token mismatch"), Some("C:\\path\\cross_compile.rs"));
        assert_eq!(ParseFailedLine("C:\\path\\cross_compile.rs (rs)"), Some("C:\\path\\cross_compile.rs"));
        assert_eq!(ParseFailedLine("C:\\path\\cross_compile.rs"), Some("C:\\path\\cross_compile.rs"));
        assert_eq!(ParseFailedLine("C:\\path\\nonexistent.rs (Not Found) Reason: Specified file or directory not found"), Some("C:\\path\\nonexistent.rs"));
        assert_eq!(ParseFailedLine("File: C:\\path\\cross_compile.rs"), Some("C:\\path\\cross_compile.rs"));
        assert_eq!(ParseFailedLine("File: C:\\path\\cross_compile.rs (rs)"), Some("C:\\path\\cross_compile.rs"));
        assert_eq!(ParseFailedLine("Language: rs"), None);
        assert_eq!(ParseFailedLine("Reason: Token mismatch"), None);
        assert_eq!(ParseFailedLine("   \n"), None);
    }

   #[test]
    fn Test_Default_Failed_Log_Path()
    {
        let PathStr = GetDefaultFailedLogPath();
        assert!(PathStr.ends_with("failed.txt"), "Default failed log path must end with failed.txt");
    }

   #[test]
    fn Test_Transform_File_Name()
    {
        // 1. No prefix, no suffix
        assert_eq!(TransformFileName("test.rs", None, None), "test.rs");

        // 2. Prefix only
        assert_eq!(TransformFileName("test.rs", Some("formatted_"), None), "formatted_test.rs");

        // 3. Suffix only
        assert_eq!(TransformFileName("test.rs", None, Some(".formatted")), "test.formatted.rs");

        // 4. Prefix and suffix
        assert_eq!(TransformFileName("test.rs", Some("pre_"), Some("_post")), "pre_test_post.rs");

        // 5. File without extension
        assert_eq!(TransformFileName("Makefile", Some("formatted_"), None), "formatted_Makefile");
        assert_eq!(TransformFileName("Makefile", None, Some(".formatted")), "Makefile.formatted");

        // 6. Dotfile
        assert_eq!(TransformFileName(".gitignore", Some("formatted_"), None), "formatted_.gitignore");
        assert_eq!(TransformFileName(".gitignore", None, Some(".formatted")), ".gitignore.formatted");

        // 7. Multi-dot file
        assert_eq!(TransformFileName("test.min.js", None, Some(".formatted")), "test.min.formatted.js");
    }

   #[test]
    fn Test_Compute_Output_Path()
    {
        // 1. Single file with explicit output file
        let Out1 = ComputeOutputPath(Path::new ("src/main.rs"), Some("out/formatted_main.rs"), None, None, & ["src/main.rs".to_string()], true);
        assert_eq!(Out1, PathBuf::from("out/formatted_main.rs"));

        // 2. Single file with output directory (trailing slash)
        let Out2 = ComputeOutputPath(Path::new ("src/main.rs"), Some("dist/"), None, None, & ["src/main.rs".to_string()], true);
        assert_eq!(Out2, PathBuf::from("dist/main.rs"));

        // 3. Directory target with output directory (replicates directory tree structure)
        let Out3 = ComputeOutputPath(Path::new ("src/languages/js_ts/mod.rs"), Some("dist"), None, None, & ["src".to_string()], false);
        assert_eq!(Out3, PathBuf::from("dist/languages/js_ts/mod.rs"));

        // 4. Project root target with output directory
        let Out4 = ComputeOutputPath(Path::new ("src/languages/js_ts/mod.rs"), Some("dist"), None, None, & [".".to_string()], false);
        assert_eq!(Out4, PathBuf::from("dist/src/languages/js_ts/mod.rs"));

        // 5. Prefix and suffix with output directory
        let Out5 = ComputeOutputPath(Path::new ("src/languages/js_ts/mod.rs"), Some("dist"), Some("f_"), Some(".lafs"), & ["src".to_string()], false);
        assert_eq!(Out5, PathBuf::from("dist/languages/js_ts/f_mod.lafs.rs"));

        // 6. Prefix without destination directory (saves alongside original)
        let Out6 = ComputeOutputPath(Path::new ("src/languages/js_ts/mod.rs"), None, Some("f_"), None, & ["src".to_string()], false);
        assert_eq!(Out6, PathBuf::from("src/languages/js_ts/f_mod.rs"));

        // 7. Suffix without destination directory
        let Out7 = ComputeOutputPath(Path::new ("src/languages/js_ts/mod.rs"), None, None, Some(".formatted"), & ["src".to_string()], false);
        assert_eq!(Out7, PathBuf::from("src/languages/js_ts/mod.formatted.rs"));

        // 8. Safety fallback: identical path with explicit output defaults to .formatted
        let Out8 = ComputeOutputPath(Path::new ("src/main.rs"), Some("src/main.rs"), None, None, & ["src/main.rs".to_string()], true);
        assert_eq!(Out8, PathBuf::from("src/main.formatted.rs"));
    }
}

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

pub mod container_tree;
pub mod changelog;
pub mod emitter;
pub mod formatter;
pub mod languages;
pub mod lexer;
pub mod matcher;
pub mod planner;
pub mod profile;
pub mod types;
pub mod verifier;

// Re-export language modules and formatters for clean top-level API and backward compatibility
pub use languages::batch;
pub use languages::dart;
pub use languages::go;
pub use languages::js_ts;
pub use languages::kotlin;
pub use languages::markdown;
pub use languages::polyglot;
pub use languages::shell;

pub use languages::batch::Batch_Formatter;
pub use languages::dart::Dart_Formatter;
pub use languages::go::Go_Formatter;
pub use languages::js_ts::JS_TS_Formatter;
pub use languages::kotlin::Kotlin_Formatter;
pub use languages::markdown::Markdown_Formatter;
pub use languages::polyglot::Polyglot_Formatter;
pub use languages::shell::Shell_Formatter;

pub use formatter::Source_Formatter;
pub use matcher:: { Container_Matcher, Match_Error, Match_Error_Code };
pub use types:: { Config_Settings, Container_Kind, Kotlin_Style, Layout_State, Source_Language };
pub use verifier::Safety_Verifier;

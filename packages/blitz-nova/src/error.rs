
use std::fmt;

use miette as oxc_miette;
use oxc_diagnostics::OxcDiagnostic;
use oxc_miette::{Diagnostic, NamedSource, SourceSpan};

#[derive(Diagnostic, Debug, Clone)]
pub enum NovaError {
    #[diagnostic(
        code(andromeda::parse::syntax_error),
        help(
            "🔍 Check the syntax of your JavaScript/TypeScript code.\n💡 Look for missing semicolons, brackets, or quotes.\n📖 Refer to the JavaScript/TypeScript language specification."
        ),
        url("https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types")
    )]
    ParseError {
        errors: Vec<OxcDiagnostic>,
        source_path: String,
        #[source_code]
        source_code: NamedSource<String>,
        #[label("❌ Parse error occurred here")]
        primary_error_span: Option<SourceSpan>,
        related_spans: Vec<SourceSpan>,
    },
    RuntimeError {
        message: String,
        #[label("⚡ Runtime error occurred here")]
        location: Option<SourceSpan>,
        #[source_code]
        source_code: Option<NamedSource<String>>,
        /// Stack trace information for better debugging
        stack_trace: Option<String>,
        /// Variable context at the time of error
        variable_context: Vec<(String, String)>,
        related_locations: Vec<SourceSpan>,
    },
}

impl NovaError {
    pub fn parse_error(
        errors: Vec<OxcDiagnostic>,
        source_path: impl Into<String>,
        source_code: impl Into<String>,
    ) -> Self {
        let source_path = source_path.into();
        let source_code = source_code.into();

        // Extract primary error span from the first diagnostic
        let primary_error_span = errors.first().and_then(|diagnostic| {
            diagnostic.labels.as_ref().and_then(|labels| {
                labels
                    .first()
                    .map(|label| SourceSpan::new(label.offset().into(), label.len()))
            })
        });

        // Extract additional spans for related errors
        let related_spans = errors
            .iter()
            .skip(1)
            .filter_map(|diagnostic| {
                diagnostic.labels.as_ref().and_then(|labels| {
                    labels
                        .first()
                        .map(|label| SourceSpan::new(label.offset().into(), label.len()))
                })
            })
            .collect();

        Self::ParseError {
            errors,
            source_path: source_path.clone(),
            source_code: NamedSource::new(source_path, source_code),
            primary_error_span,
            related_spans,
        }
    }
    
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location: None,
            source_code: None,
            stack_trace: None,
            variable_context: Vec::new(),
            related_locations: Vec::new(),
        }
    }
}

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NovaError::ParseError { source_path, .. } => {
                write!(f, "Parse error in {source_path}")
            }
            NovaError::RuntimeError { message, .. } => {
                write!(f, "Runtime error: {message}")
            }
        }
    }
}

impl std::error::Error for NovaError {}

pub struct ErrorReporter;

impl ErrorReporter {
    pub fn print_error(error: &NovaError) {
        
    }
}



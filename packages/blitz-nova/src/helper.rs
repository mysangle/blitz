
use oxc_diagnostics::OxcDiagnostic;

use crate::error::{ErrorReporter, NovaError};

pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    let parse_error = NovaError::parse_error(errors, source_path, source);
    ErrorReporter::print_error(&parse_error);
    std::process::exit(1);
}

pub fn print_enhanced_error(error: &NovaError) {
    ErrorReporter::print_error(error);
}

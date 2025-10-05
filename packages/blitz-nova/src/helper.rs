
use oxc_diagnostics::OxcDiagnostic;

use crate::error::{ErrorReporter, NovaError};

pub fn event_dispatch_js(node_id: usize, event_type: &str) -> String {
    format!("new Node({}).dispatchEvent(new Event('{}'))", node_id, event_type)
}

pub fn exit_with_parse_errors(errors: Vec<OxcDiagnostic>, source_path: &str, source: &str) -> ! {
    assert!(!errors.is_empty());

    let parse_error = NovaError::parse_error(errors, source_path, source);
    ErrorReporter::print_error(&parse_error);
    std::process::exit(1);
}

pub fn print_enhanced_error(error: &NovaError) {
    ErrorReporter::print_error(error);
}

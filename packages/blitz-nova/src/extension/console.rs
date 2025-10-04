
use std::io::{Write, stdout};

use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList},
        execution::{
            agent::{Agent, JsResult},
        },
        types::{Value},
    },
    engine::{
        context::{Bindable, GcScope},
    },
};

use crate::error::{BlitzError, ErrorReporter};
use crate::extension::{Extension, ExtensionOp};

#[derive(Default)]
pub struct ConsoleExt;

impl ConsoleExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "document",
            ops: vec![
                ExtensionOp::new("internal_print", Self::internal_print, 1, false),
            ],
            storage: None,
        }
    }
    
    fn internal_print<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        if let Err(e) = stdout().write_all(
            args[0]
                .to_string(agent, gc.reborrow())
                .unbind()?
                .as_str(agent)
                .expect("String is not valid UTF-8")
                .as_bytes(),
        ) {
            let error = BlitzError::runtime_error(format!("Failed to write to stdout: {e}"));
            ErrorReporter::print_error(&error);
        }
        if let Err(e) = stdout().flush() {
            let error = BlitzError::runtime_error(format!("Failed to flush stdout: {e}"));
            ErrorReporter::print_error(&error);
        }
        Ok(Value::Undefined)
    }
}

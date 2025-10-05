
use nova_vm::{
    ecmascript::{
        builtins::{Behaviour, BuiltinFunctionArgs, RegularFn, create_builtin_function},
        execution::Agent,
        scripts_and_modules::script::{HostDefined, parse_script, script_evaluation},
        types::{InternalMethods, IntoValue, Object, PropertyDescriptor, PropertyKey},
    },
    engine::context::{Bindable, GcScope},
};

mod console;
mod document;
mod node;
mod time;
pub mod timeout;

use console::ConsoleExt;
use document::DocumentExt;
use node::NodeExt;
use time::TimeExt;

use crate::{
    error::NovaError,
    helper::{exit_with_parse_errors, print_enhanced_error},
    host_data::{HostData, OpsStorage},
};

pub type ExtensionStorageInit = Box<dyn FnOnce(&mut OpsStorage)>;

pub struct Extension {
    pub name: &'static str,
    pub ops: Vec<ExtensionOp>,
    pub storage: Option<ExtensionStorageInit>,
    pub files: Vec<&'static str>,
}

impl Extension {
    pub(crate) fn load<ScriptMacroTask: 'static>(
        &mut self,
        agent: &mut Agent,
        global_object: Object,
        blitz_object: Object,
        gc: &mut GcScope<'_, '_>,
    ) {
        for (idx, file_source) in self.files.iter().enumerate() {
            let specifier = format!("<ext:{}:{}>", self.name, idx);
            let source_text =
                nova_vm::ecmascript::types::String::from_str(agent, file_source, gc.nogc());

            let script = match parse_script(
                agent,
                source_text,
                agent.current_realm(gc.nogc()),
                false,
                Some(std::rc::Rc::new(specifier.clone()) as HostDefined),
                gc.nogc(),
            ) {
                Ok(script) => script,
                Err(errors) => {
                    // Borrow the string data from the Agent
                    let source_text = source_text.to_string_lossy(agent);
                    exit_with_parse_errors(errors, &specifier, &source_text)
                }
            };
            let _ = script_evaluation(agent, script.unbind(), gc.reborrow());
        }
        for op in &self.ops {
            let function = create_builtin_function(
                agent,
                Behaviour::Regular(op.function),
                BuiltinFunctionArgs::new(op.args, op.name),
                gc.nogc(),
            );
            function.unbind();
            let property_key = PropertyKey::from_static_str(agent, op.name, gc.nogc());
            if op.global {
                global_object
                    .internal_define_own_property(
                        agent,
                        property_key.unbind(),
                        PropertyDescriptor {
                            value: Some(function.into_value().unbind()),
                            ..Default::default()
                        },
                        gc.reborrow(),
                    )
                    .unwrap();
            } else {
                blitz_object
                    .internal_define_own_property(
                        agent,
                        property_key.unbind(),
                        PropertyDescriptor {
                            value: Some(function.into_value().unbind()),
                            ..Default::default()
                        },
                        gc.reborrow(),
                    )
                    .unwrap();
            }
        }
        
        if let Some(storage_hook) = self.storage.take() {
            let host_data = agent.get_host_data();
            let host_data: &HostData = host_data.downcast_ref().unwrap();
            let mut storage = host_data.storage.borrow_mut();
            (storage_hook)(&mut storage)
        }
    }
}

pub struct ExtensionOp {
    pub name: &'static str,
    pub function: RegularFn,
    pub args: u32,
    pub global: bool,
}

impl ExtensionOp {
    pub fn new(name: &'static str, function: RegularFn, args: u32, global: bool) -> Self {
        Self {
            name,
            args,
            function,
            global,
        }
    }
}

pub fn recommended_extensions() -> Vec<Extension> {
    vec![
        ConsoleExt::new_extension(),
        DocumentExt::new_extension(),
        NodeExt::new_extension(),
        TimeExt::new_extension(),
    ]
}

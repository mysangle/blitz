
use nova_vm::{
    ecmascript::{
        builtins::{Behaviour, BuiltinFunctionArgs, RegularFn, create_builtin_function},
        execution::Agent,
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

use crate::host_data::{HostData, OpsStorage};

pub type ExtensionStorageInit = Box<dyn FnOnce(&mut OpsStorage)>;

pub struct Extension {
    pub name: &'static str,
    pub ops: Vec<ExtensionOp>,
    pub storage: Option<ExtensionStorageInit>,
}

impl Extension {
    pub(crate) fn load<ScriptMacroTask: 'static>(
        &mut self,
        agent: &mut Agent,
        global_object: Object,
        blitz_object: Object,
        gc: &mut GcScope<'_, '_>,
    ) {
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
            let host_data: &HostData<ScriptMacroTask> = host_data.downcast_ref().unwrap();
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

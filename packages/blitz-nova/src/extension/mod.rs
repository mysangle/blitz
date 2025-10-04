
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

use document::DocumentExt;

use crate::extension::{console::ConsoleExt, node::NodeExt};

pub struct Extension {
    pub name: &'static str,
    pub ops: Vec<ExtensionOp>,
}

impl Extension {
    pub(crate) fn load(
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
    ]
}

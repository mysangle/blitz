
use nova_vm::{
    SmallInteger,
    ecmascript::{
        builtins::{ArgumentsList, Array},
        execution::{
            agent::{Agent, ExceptionType, JsResult},
        },
        types::{
            String, Value,
        },
    },
    engine::{
        context::{Bindable, GcScope},
    },
};

use crate::{extension::{Extension, ExtensionOp}, HostHandler};

#[derive(Default)]
pub struct DocumentExt;

impl DocumentExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "document",
            ops: vec![
                ExtensionOp::new("internal_query_selector_all", Self::internal_query_selector_all, 1, false),
            ],
            storage: None,
            files: vec![],
        }
    }
    
    fn internal_query_selector_all<'gc>(
        agent: &mut Agent,
        _: Value,
        args: ArgumentsList,
        gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let args = args.bind(gc.nogc());
        if args.len() != 1 {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected 1 argument",
                gc.into_nogc(),
            ));
        }
        let Ok(selector) = String::try_from(args.get(0)) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected a string argument",
                gc.into_nogc(),
            ));
        };
        
        let Some(selector) = selector.as_str(agent) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected a string argument",
                gc.into_nogc(),
            ));
        };
        
        let host_data = agent.get_host_data();
        let host_data: &Box<dyn HostHandler> = host_data.downcast_ref().unwrap();
        let node_ids = host_data.query_selector_all(selector);
        
        let keys = node_ids
            .iter()
            .map(|v| {
                Value::Integer(SmallInteger::from(*v as u32))
            })
            .collect::<Vec<_>>();
        
        Ok(Array::from_slice(agent, keys.as_slice(), gc.nogc())
            .unbind()
            .into())
    }
}

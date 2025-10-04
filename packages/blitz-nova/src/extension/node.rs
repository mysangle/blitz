

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
pub struct NodeExt;

impl NodeExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "node",
            ops: vec![
                ExtensionOp::new("internal_get_attribute", Self::internal_get_attribute, 2, false),
                ExtensionOp::new("internal_inner_html_set", Self::internal_inner_html_set, 2, false),
            ],
        }
    }
    
    fn internal_get_attribute<'gc>(
        agent: &mut Agent,
        _: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let Ok(handle) = args.get(0).to_uint32(agent, gc.reborrow()) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected a int argument",
                gc.into_nogc(),
            ));
        };
        
        let args = args.bind(gc.nogc());
        if args.len() != 2 {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected 2 argument",
                gc.into_nogc(),
            ));
        }
        
        let Ok(name) = String::try_from(args.get(1)) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected a string argument",
                gc.into_nogc(),
            ));
        };
        
        let Some(name) = name.as_str(agent) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected a string argument",
                gc.into_nogc(),
            ));
        };
        
        let host_data = agent.get_host_data();
        let host_data: &Box<dyn HostHandler> = host_data.downcast_ref().unwrap();
        let Some(attr) = host_data.get_attribute(handle as usize, &name) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "no attribute",
                gc.into_nogc(),
            ));
        };
        
        Ok(Value::from_string(agent, attr.to_string(), gc.nogc()).unbind())
    }
    
    fn internal_inner_html_set<'gc>(
        agent: &mut Agent,
        _: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let handle = args.get(0).to_uint32(agent, gc.reborrow()).unbind()?;
        let inner_html = args.get(1).to_string(agent, gc.reborrow()).unbind()?;
        let Some(inner_html) = inner_html.as_str(agent) else {
            return Err(agent.throw_exception_with_static_message(
                ExceptionType::Error,
                "Expected a string argument",
                gc.into_nogc(),
            ));
        };
        
        let host_data = agent.get_host_data();
        let host_data: &Box<dyn HostHandler> = host_data.downcast_ref().unwrap();
        host_data.inner_html_set(handle as usize, &inner_html);
        
        Ok(Value::Undefined)
    }
}

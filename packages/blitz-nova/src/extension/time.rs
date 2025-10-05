
use std::time::Duration;

use nova_vm::{
    ecmascript::{
        builtins::{ArgumentsList},
        execution::{
            agent::{Agent, JsResult},
        },
        types::{Value},
    },
    engine::{
        Global,
        context::{Bindable, GcScope},
    },
};

use crate::{
    extension::{
        Extension, ExtensionOp,
        timeout::{Timeout, TimeoutId, TimeoutsStorage},
    },
    host_data::{HostData, OpsStorage},
    runtime::{MacroTask, BlitzMacroTask},
};

#[derive(Default)]
pub struct TimeExt;

impl TimeExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "time",
            ops: vec![
                ExtensionOp::new("setTimeout", Self::set_timeout, 2, true),
                ExtensionOp::new("clearTimeout", Self::clear_timeout, 1, true),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(TimeoutsStorage::default());
            })),
            files: vec![],
        }
    }
    
    pub fn set_timeout<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let callback = args[0];
        let time_ms = args[1].to_uint32(agent, gc.reborrow()).unwrap();
        let duration = Duration::from_millis(time_ms as u64);
        
        let root_callback = Global::new(agent, callback.unbind());
        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();
        let task_provider = host_data.task_sender();
        
        let timeout_id = Timeout::create(host_data, duration, root_callback, |timeout_id| {
            host_data.spawn_macro_task(async move {
                tokio::time::sleep(duration).await;
                task_provider.send(MacroTask::Script(BlitzMacroTask::RunAndClearTimeout(timeout_id))).unwrap();
            })
        });
        let timeout_id_value =
            Value::from_f64(agent, timeout_id.index() as f64, gc.nogc()).unbind();

        Ok(timeout_id_value)
    }
    
    pub fn clear_timeout<'gc>(
        agent: &mut Agent,
        _this: Value,
        args: ArgumentsList,
        mut gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        let timeout_id_value = args[0];
        let timeout_id_u32 = timeout_id_value.to_uint32(agent, gc.reborrow()).unwrap();
        let timeout_id = TimeoutId::from_index(timeout_id_u32);

        let host_data = agent.get_host_data();
        let host_data: &HostData = host_data.downcast_ref().unwrap();

        let task_provider = host_data.task_sender();
        task_provider
            .send(MacroTask::Script(BlitzMacroTask::ClearTimeout(timeout_id)))
            .unwrap();

        Ok(Value::Undefined)
    }
}

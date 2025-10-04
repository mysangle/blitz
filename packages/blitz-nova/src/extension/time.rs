
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
    event_loop::{MacroTask, BlitzMacroTask},
    extension::{
        Extension, ExtensionOp,
        timeout::{Timeout, TimeoutsStorage},
    },
    host_data::{HostData, OpsStorage},
};

#[derive(Default)]
pub struct TimeExt;

impl TimeExt {
    pub fn new_extension() -> Extension {
        Extension {
            name: "time",
            ops: vec![
                ExtensionOp::new("setTimeout", Self::set_timeout, 2, true),
            ],
            storage: Some(Box::new(|storage: &mut OpsStorage| {
                storage.insert(TimeoutsStorage::default());
            })),
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
        let host_data: &HostData<BlitzMacroTask> = host_data.downcast_ref().unwrap();
        let macro_task_tx = host_data.macro_task_tx();
        
        let timeout_id = Timeout::create(host_data, duration, root_callback, |timeout_id| {
            host_data.spawn_macro_task(async move {
                tokio::time::sleep(duration).await;
                macro_task_tx
                    .send(MacroTask::Script(BlitzMacroTask::RunAndClearTimeout(
                        timeout_id,
                    )))
                    .unwrap();
            })
        });
        let timeout_id_value =
            Value::from_f64(agent, timeout_id.index() as f64, gc.nogc()).unbind();

        Ok(timeout_id_value)
    }
}

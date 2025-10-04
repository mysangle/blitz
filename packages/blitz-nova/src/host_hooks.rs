
use std::{
    any::Any,
};

use nova_vm::{
    ecmascript::{
        execution::{
            agent::{Agent, HostHooks, Job},
        },
        scripts_and_modules::{
            module::module_semantics::{
                cyclic_module_records::GraphLoadingStateRecord, ModuleRequest, Referrer,
            },
            script::{HostDefined},
        },
    },
    engine::{
        context::{NoGcScope},
    },
};

use crate::host_data::HostData;

pub struct BlitzHostHooks<ScriptMacroTask> {
    pub host_data: HostData<ScriptMacroTask>,
}

impl<ScriptMacroTask> BlitzHostHooks<ScriptMacroTask> {
    pub fn new(host_data: HostData<ScriptMacroTask>) -> Self {
        Self {
            host_data,
        }
    }
}

impl<ScriptMacroTask> std::fmt::Debug for BlitzHostHooks<ScriptMacroTask> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlitzHostHooks")
            .field("handler", &std::any::type_name::<dyn HostHandler>())
            .finish()
    }
}

impl<ScriptMacroTask: 'static> HostHooks for BlitzHostHooks<ScriptMacroTask> {
    fn enqueue_promise_job(&self, _job: Job) { unimplemented!(); }
    fn load_imported_module<'gc>( &self, _agent: &mut Agent, _referrer: Referrer<'gc>, _module_request: ModuleRequest<'gc>, _host_defined: Option<HostDefined>, _payload: &mut GraphLoadingStateRecord<'gc>, _gc: NoGcScope<'gc, '_>) { unimplemented!(); }
    
    fn get_host_data(&self) -> &dyn Any {
        &self.host_data
    }
}

pub trait HostHandler {
    fn query_selector_all(&self, selector: &str) -> Vec<usize>;
    fn get_attribute(&self, node_id: usize, name: &str) -> Option<String>;
    fn inner_html_set(&self, node_id: usize, html: &str);
}

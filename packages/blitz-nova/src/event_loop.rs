
use std::sync::mpsc::Receiver;

use nova_vm::ecmascript::execution::agent::{GcAgent, RealmRoot};

use crate::{
    extension::timeout::TimeoutId,
    host_data::HostData,
    host_hooks::{BlitzHostHooks, HostHandler},
    context::JSContext,
};

pub type EventLoopHandler<ScriptMacroTask> = fn(
    macro_task: ScriptMacroTask,
    agent: &mut GcAgent,
    realm_root: &RealmRoot,
    host_data: &HostData<ScriptMacroTask>,
);

pub struct JSEventLoop<ScriptMacroTask: 'static> {
    pub context: JSContext,
    pub host_hooks: &'static BlitzHostHooks<ScriptMacroTask>,
    pub eventloop_handler: EventLoopHandler<ScriptMacroTask>,
    pub macro_task_rx: Receiver<MacroTask<ScriptMacroTask>>,
}

impl<ScriptMacroTask> JSEventLoop<ScriptMacroTask> {
    pub fn new(
        host_data: HostData<ScriptMacroTask>,
        macro_task_rx: Receiver<MacroTask<ScriptMacroTask>>,
        eventloop_handler: EventLoopHandler<ScriptMacroTask>,
    ) -> Self {
        let host_hooks = BlitzHostHooks::new(host_data);
        let host_hooks: &BlitzHostHooks<ScriptMacroTask> = &*Box::leak(Box::new(host_hooks));
        let context = JSContext::new::<ScriptMacroTask>(host_hooks);
        
        Self {
            context,
            host_hooks,
            eventloop_handler,
            macro_task_rx
        }
    }
    
    pub fn run(mut self, script: &str) {
        let _ = self.context.run(script);
        
        loop {
            self.handle_macro_task();
        }
    }
    
    pub fn handle_macro_task(&mut self) {
        match self.macro_task_rx.recv() {
            Ok(MacroTask::Script(task)) => {
                (self.eventloop_handler)(
                    task,
                    &mut self.context.agent,
                    &self.context.realm_root,
                    &self.host_hooks.host_data,
                )
            }
            _ => {}
        }
    }
}

pub fn recommended_eventloop_handler(
    macro_task: BlitzMacroTask,
    agent: &mut GcAgent,
    realm_root: &RealmRoot,
    host_data: &HostData<BlitzMacroTask>,
) {
    match macro_task {
        BlitzMacroTask::RunAndClearTimeout(timeout_id) => {
            timeout_id.run_and_clear(agent, host_data, realm_root)
        }
    }
}

#[derive(Debug)]
pub enum MacroTask<ScriptMacroTask> {
    Script(ScriptMacroTask),
}

pub enum BlitzMacroTask {
    RunAndClearTimeout(TimeoutId),
}

pub struct BlitzHostHandler {
    
}

impl HostHandler for BlitzHostHandler {
    fn query_selector_all(&self, selector: &str) -> Vec<usize> {
        vec![]
    }
    
    fn get_attribute(&self, node_id: usize, name: &str) -> Option<String> {
        None
    }
    
    fn inner_html_set(&self, node_id: usize, html: &str) {
        
    }
}

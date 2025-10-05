
use std::sync::mpsc::Receiver;

use nova_vm::ecmascript::execution::agent::GcAgent;

use crate::{
    extension::timeout::TimeoutId,
    host_data::HostData,
    host_hooks::BlitzHostHooks,
    context::JSContext,
};

pub struct Runtime {
    pub context: JSContext,
    pub host_hooks: &'static BlitzHostHooks,
    pub macro_task_rx: Receiver<MacroTask>,
}

impl Runtime {
    pub fn new(
        host_data: HostData,
        macro_task_rx: Receiver<MacroTask>,
    ) -> Self {
        let host_hooks = BlitzHostHooks::new(host_data);
        let host_hooks: &BlitzHostHooks = &*Box::leak(Box::new(host_hooks));
        let context = JSContext::new::<BlitzMacroTask>(host_hooks);
        
        Self {
            context,
            host_hooks,
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
                self.handle_script_task(task)
            }
            _ => {}
        }
    }
    
    fn handle_script_task(&mut self, macro_task: BlitzMacroTask) {
        let agent: &mut GcAgent = &mut self.context.agent;
        let host_data = &self.host_hooks.host_data;
        let realm_root = &self.context.realm_root;
        match macro_task {
            BlitzMacroTask::RunAndClearTimeout(timeout_id) => {
                timeout_id.run_and_clear(agent, host_data, realm_root)
            }
            BlitzMacroTask::ClearTimeout(timeout_id) => {
                timeout_id.clear_and_abort(host_data);
            }
        }
    }
}

#[derive(Debug)]
pub enum MacroTask {
    Script(BlitzMacroTask),
}

#[derive(Debug)]
pub enum BlitzMacroTask {
    RunAndClearTimeout(TimeoutId),
    ClearTimeout(TimeoutId),
}


use std::sync::{
    Arc,
    mpsc::{Sender, SendError},
};

mod error;
mod extension;
mod helper;
mod host_data;
mod host_hooks;
mod context;
mod runtime;
mod task;

pub use context::{JSContext, JSContextError};
pub use runtime::{BlitzMacroTask, MacroTask, Runtime};
pub use host_data::{HostData, TaskSender};
pub use host_hooks::HostHandler;

struct BlitzHostHandler {
    
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

struct BlitzTaskSender {
    pub macro_task_tx: Sender<MacroTask>,
}

impl TaskSender for BlitzTaskSender {
    fn send(&self, task: MacroTask) -> Result<(), SendError<MacroTask>> {
        self.macro_task_tx.send(task)
    }
}

pub fn run(script: &str) {
    let handler: Box<dyn HostHandler> = Box::new(BlitzHostHandler {
        // document
    });
    let (macro_task_tx, macro_task_rx) = std::sync::mpsc::channel();
    let task_sender = Arc::new(BlitzTaskSender { macro_task_tx });
    let host_data = HostData::new(handler, task_sender);
    
    let runtime = Runtime::new(host_data, macro_task_rx);
    
    runtime.run(script);
}

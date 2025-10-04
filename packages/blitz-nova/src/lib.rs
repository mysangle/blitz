
mod error;
mod event_loop;
mod extension;
mod host_data;
mod host_hooks;
mod context;
mod runtime;
mod task;

pub use context::{JSContext, JSContextError};
pub use event_loop::{BlitzHostHandler, BlitzMacroTask, JSEventLoop};
pub use host_data::HostData;
pub use host_hooks::HostHandler;
pub use runtime::event_dispatch_js;

use runtime::TEST_JS;

use crate::event_loop::recommended_eventloop_handler;

pub fn run_event_loop() {
    let handler: Box<dyn HostHandler> = Box::new(BlitzHostHandler {
        
    });
    let (macro_task_tx, macro_task_rx) = std::sync::mpsc::channel();
    let host_data: HostData<BlitzMacroTask> = HostData::new(handler, macro_task_tx);
    
    let event_loop = JSEventLoop::new(
        host_data,
        macro_task_rx,
        recommended_eventloop_handler,
    );
    
    event_loop.run(TEST_JS);
}

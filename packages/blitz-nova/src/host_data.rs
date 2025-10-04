
use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
        mpsc::Sender,
    },
};

use anymap::AnyMap;
use tokio::task::JoinHandle;

use crate::{
    event_loop::MacroTask,
    host_hooks::HostHandler,
    task::TaskId,
};

pub type OpsStorage = AnyMap;
pub type LocalOpsStorage = RefCell<OpsStorage>;

pub struct HostData<ScriptMacroTask> {
    pub handler: Box<dyn HostHandler>,
    pub storage: LocalOpsStorage,
    pub macro_task_tx: Sender<MacroTask<ScriptMacroTask>>,
    pub macro_task_count: Arc<AtomicU32>,
    pub tasks: RefCell<HashMap<TaskId, JoinHandle<()>>>,
    pub task_count: Arc<AtomicU32>,
}

impl<ScriptMacroTask> HostData<ScriptMacroTask> {
    pub fn new(handler: Box<dyn HostHandler>, macro_task_tx: Sender<MacroTask<ScriptMacroTask>>) -> Self {
        Self {
            handler,
            storage: RefCell::new(AnyMap::new()),
            macro_task_tx,
            macro_task_count: Arc::new(AtomicU32::new(0)),
            tasks: RefCell::default(),
            task_count: Arc::default(),
        }
    }
    
    pub fn macro_task_tx(&self) -> Sender<MacroTask<ScriptMacroTask>> {
        self.macro_task_tx.clone()
    }
    
    pub fn spawn_macro_task<F>(&self, future: F) -> TaskId
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let macro_task_count = self.macro_task_count.clone();
        macro_task_count.fetch_add(1, Ordering::Relaxed);

        let task_handle = tokio::spawn(async move {
            future.await;
            macro_task_count.fetch_sub(1, Ordering::Relaxed);
        });

        let task_id = TaskId::from_index(self.task_count.fetch_add(1, Ordering::Relaxed));
        self.tasks.borrow_mut().insert(task_id, task_handle);

        task_id
    }
    
    pub fn clear_macro_task(&self, task_id: TaskId) {
        self.tasks.borrow_mut().remove(&task_id).unwrap();
    }
}

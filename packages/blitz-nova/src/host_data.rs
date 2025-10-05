
use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
        mpsc::SendError,
    },
};

use anymap::AnyMap;
use tokio::task::JoinHandle;

use crate::{
    host_hooks::HostHandler,
    runtime::MacroTask,
    task::TaskId,
};

pub type OpsStorage = AnyMap;
pub type LocalOpsStorage = RefCell<OpsStorage>;

pub trait TaskSender: Send + Sync {
    fn send(&self, task: MacroTask) -> Result<(), SendError<MacroTask>>;
}

pub struct HostData {
    pub handler: Box<dyn HostHandler>,
    pub storage: LocalOpsStorage,
    pub task_sender: Arc<dyn TaskSender>,
    pub macro_task_count: Arc<AtomicU32>,
    pub tasks: RefCell<HashMap<TaskId, JoinHandle<()>>>,
    pub task_count: Arc<AtomicU32>,
}

impl HostData {
    pub fn new(handler: Box<dyn HostHandler>, task_sender: Arc<dyn TaskSender>) -> Self {
        Self {
            handler,
            storage: RefCell::new(AnyMap::new()),
            task_sender,
            macro_task_count: Arc::new(AtomicU32::new(0)),
            tasks: RefCell::default(),
            task_count: Arc::default(),
        }
    }
    
    pub fn task_sender(&self) -> Arc<dyn TaskSender> {
        self.task_sender.clone()
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
    
    pub fn abort_macro_task(&self, task_id: TaskId) {
        let tasks = self.tasks.borrow();
        let task = tasks.get(&task_id).unwrap();
        task.abort();

        // Manually decrease the macro tasks counter as the task was aborted.
        self.macro_task_count.fetch_sub(1, Ordering::Relaxed);
    }
    
    pub fn clear_macro_task(&self, task_id: TaskId) {
        self.tasks.borrow_mut().remove(&task_id).unwrap();
    }
}

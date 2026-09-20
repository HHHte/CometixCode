//! Reactive unified command-queue snapshot.
//!
//! Maps to: CC `hooks/useCommandQueue.ts`.

use crate::utils::message_queue_manager::{
    QueuedCommand, get_command_queue_snapshot, subscribe_to_command_queue,
};
use iocraft::prelude::*;

pub fn use_command_queue(hooks: &mut Hooks<'_, '_>) -> Vec<QueuedCommand> {
    let mut revision = hooks.use_state(|| 0u64);
    let receiver = hooks.use_const(|| std::sync::Arc::new(subscribe_to_command_queue()));
    hooks.use_future(async move {
        while receiver.recv().await.is_ok() {
            revision += 1;
        }
    });
    let _ = revision.get();
    get_command_queue_snapshot()
}

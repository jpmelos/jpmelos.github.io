use std::{
    sync::mpsc,
    thread::{self, ScopedJoinHandle},
    time::{Duration, Instant},
};

pub fn spawn_self_shipping_thread_in_scope<'scope, F, R>(
    scope: &'scope thread::Scope<'scope, '_>,
    tx: mpsc::Sender<ScopedJoinHandle<'scope, R>>,
    func: F,
) where
    F: FnOnce() -> R + Send + 'scope,
    R: Send + 'scope,
{
    // Create the channel that will be used to transfer the new thread's handle from the parent
    // thread to the new thread.
    let (handle_tx, handle_rx) = mpsc::channel();

    // Spawn the new thread in the scope.
    let handle = scope.spawn(move || {
        // Execute the target function.
        let result = func();

        // Receive the handler that was sent by the parent thread to the new thread via the
        // channel.
        let handle = handle_rx.recv().unwrap();
        // And immediately send it to the caller of `spawn_self_shipping_thread_in_scope`. It is
        // responsibility of the caller to make sure that the `rx` side of this channel is alive
        // until after this thread is finished.
        tx.send(handle).unwrap();

        // Return the same result as the target function.
        result
    });

    // Send the new thread's handle into the new thread itself.
    handle_tx.send(handle).unwrap();
}

// I'll call this function from my threads.
fn target_fn(sleep_duration: u64) -> u64 {
    println!("Sleeping for {sleep_duration} seconds.");
    thread::sleep(Duration::from_secs(sleep_duration));
    sleep_duration
}

fn main() {
    // Create the thread scope.
    thread::scope(move |scope| {
        // Create the channel that the new threads will use to send their handles to the main
        // thread.
        let (handle_tx, handle_rx) = mpsc::channel();

        // Spawn the new threads.
        spawn_self_shipping_thread_in_scope(scope, handle_tx.clone(), || target_fn(1));
        spawn_self_shipping_thread_in_scope(scope, handle_tx.clone(), || target_fn(3));
        spawn_self_shipping_thread_in_scope(scope, handle_tx.clone(), || target_fn(5));

        // Drop this `handle_tx` so that when all the threads are finished and all the `handle_tx`
        // clones are dropped, `handle_rx` will return `Err`.
        drop(handle_tx);

        let start_time = Instant::now();
        // Receive the handle from the next thread that finishes.
        while let Ok(handle) = handle_rx.recv() {
            // Join the thread and get the result.
            match handle.join() {
                Ok(sleep_duration) => {
                    let time_elapsed = start_time.elapsed().as_secs_f64();
                    println!(
                        "{time_elapsed:.2}: Thread finished: slept for {sleep_duration} seconds."
                    );
                }
                Err(e) => eprintln!("Error joining thread: {e:?}"),
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    mod spawn_self_shipping_thread_in_scope {
        use super::*;

        #[test]
        fn handles_panic() {
            // Create the thread scope.
            thread::scope(|scope| {
                // And the channel.
                let (tx, rx) = mpsc::channel();

                // Spawn the self-shipping thread. Make it panic.
                spawn_self_shipping_thread_in_scope(scope, tx, || {
                    panic!("Thread is panicking on purpose for testing");
                });

                // Ensure something was sent over the channel.
                let handle = rx.recv().unwrap();
                // Join the self-shipping thread.
                let join_result = handle.join();
                // And see that it errored with our custom panic message.
                let err = join_result.unwrap_err();
                let panic_msg = err.downcast_ref::<&str>().unwrap();
                assert!(panic_msg.contains("Thread is panicking on purpose for testing"));
            });
        }
    }
}

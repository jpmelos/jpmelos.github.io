use std::{
    panic,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

// We'll use this enum to represent either a successful result or a panic.
enum ThreadResult<T> {
    Success(T),
    Panic(String),
}

fn spawn_thread_in_scope<'scope, F, R>(
    scope: &'scope thread::Scope<'scope, '_>,
    tx: mpsc::Sender<ThreadResult<R>>,
    func: F,
) where
    F: FnOnce() -> R + Send + panic::UnwindSafe + 'scope,
    R: Send + 'scope,
{
    // Spawn the new thread in the scope.
    scope.spawn(move || {
        // Catch any panic that might occur during the execution of the function.
        let result = panic::catch_unwind(func);

        match result {
            Ok(success) => {
                // The function executed successfully.
                tx.send(ThreadResult::Success(success)).unwrap();
            }
            Err(panic_info) => {
                // Function panicked, extract panic message.
                let panic_msg = if let Some(msg) = panic_info.downcast_ref::<&str>() {
                    (*msg).to_string()
                } else if let Some(msg) = panic_info.downcast_ref::<String>() {
                    msg.clone()
                } else {
                    "Unknown panic".to_string()
                };

                // Send the panic message.
                tx.send(ThreadResult::Panic(panic_msg)).unwrap();

                panic::resume_unwind(panic_info);
            }
        }
    });
}

// I'll call this function from my threads.
fn target_fn(sleep_duration: u64) -> u64 {
    println!("Sleeping for {sleep_duration} seconds.");
    thread::sleep(Duration::from_secs(sleep_duration));

    // To demonstrate panic handling, let's make it panic for a specific duration.
    #[allow(clippy::manual_assert)]
    if sleep_duration == 3 {
        panic!("Intentional panic in thread with sleep_duration = 3");
    }

    sleep_duration
}

fn main() {
    // Create the thread scope.
    thread::scope(move |scope| {
        // Create the channel that the new threads will use to send their results to the main
        // thread.
        let (tx, rx) = mpsc::channel();

        spawn_thread_in_scope(scope, tx.clone(), move || target_fn(1));
        spawn_thread_in_scope(scope, tx.clone(), move || target_fn(3));
        spawn_thread_in_scope(scope, tx.clone(), move || target_fn(5));

        // Drop `tx` so that when all the threads are finished and all the `tx` clones are dropped,
        // `rx` will return `Err`.
        drop(tx);

        let start_time = Instant::now();
        // Receive the result from the next thread that sends one.
        while let Ok(thread_result) = rx.recv() {
            let time_elapsed = start_time.elapsed().as_secs_f64();

            match thread_result {
                ThreadResult::Success(result) => {
                    println!(
                        "{time_elapsed:.2}: Thread finished successfully: slept for {result} seconds."
                    );
                }
                ThreadResult::Panic(panic_msg) => {
                    println!("{time_elapsed:.2}: Thread panicked with message: {panic_msg}");
                }
            }
        }
    });
}

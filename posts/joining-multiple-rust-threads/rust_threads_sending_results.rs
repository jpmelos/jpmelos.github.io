use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn spawn_thread_in_scope<'scope, F, R>(
    scope: &'scope thread::Scope<'scope, '_>,
    tx: mpsc::Sender<R>,
    func: F,
) where
    F: FnOnce() -> R + Send + 'scope,
    R: Send + 'scope,
{
    // Spawn the new thread in the scope.
    scope.spawn(move || {
        let result = func();
        tx.send(result).unwrap();
    });
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
        while let Ok(result) = rx.recv() {
            let time_elapsed = start_time.elapsed().as_secs_f64();
            println!("{time_elapsed:.2}: Thread finished: slept for {result} seconds.");
        }
    });
}

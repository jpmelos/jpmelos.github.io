use std::thread;
use std::time::{Duration, Instant};

// I'll call this function from my threads.
fn target_fn(sleep_duration: u64) -> u64 {
    println!("Sleeping for {sleep_duration} seconds.");
    thread::sleep(Duration::from_secs(sleep_duration));
    sleep_duration
}

fn main() {
    // Spawn the threads and put all the handles in a vector.
    let mut handles = vec![
        thread::spawn(|| target_fn(5)),
        thread::spawn(|| target_fn(3)),
        thread::spawn(|| target_fn(1)),
    ];

    let start_time = Instant::now();
    while !handles.is_empty() {
        let mut i = 0;

        while i < handles.len() {
            // Call `JoinHandle.is_finished` for each thread, until one of them is.
            if handles[i].is_finished() {
                let handle = handles.remove(i);
                // Call `JoinHandle.join`, get the result, print it.
                let sleep_duration = handle.join().unwrap();
                let time_elapsed = start_time.elapsed().as_secs_f64();
                println!("{time_elapsed:.2}: Thread finished: slept for {sleep_duration} seconds.");
            } else {
                i += 1;
            }
        }

        // Sleep, the main thread yields control so the others can continue.
        thread::sleep(Duration::from_millis(10));
    }
}

import concurrent.futures
import time
from concurrent.futures import ThreadPoolExecutor


# I'll call this function from my threads.
def target_fn(sleep_duration: int) -> int:
    print(f"Sleeping for {sleep_duration} seconds.")
    time.sleep(sleep_duration)
    return sleep_duration


start_time = time.monotonic()
# Using `ThreadPoolExecutor` as a context manager guarantees that all threads
# will be joined before the context manager's scope ends. This is very similar
# to Rust's scoped threads.
with ThreadPoolExecutor() as executor:
    # Collect the futures into a set.
    futures = {
        executor.submit(target_fn, 5),
        executor.submit(target_fn, 3),
        executor.submit(target_fn, 1),
    }
    # Get the futures as they finish using `as_completed`.
    for future in concurrent.futures.as_completed(futures):
        time_elapsed = time.monotonic() - start_time
        return_value = future.result()
        print(
            f"{time_elapsed:.2f}:"
            f" Thread finished:"
            f" slept for {return_value} seconds."
        )

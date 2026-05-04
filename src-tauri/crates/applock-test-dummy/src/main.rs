use std::thread;
use std::time::Duration;

fn main() {
    // Infinite sleep loop to simulate a running process
    loop {
        thread::sleep(Duration::MAX);
    }
}

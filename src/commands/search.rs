use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::Result;
use crate::output::printer::{TreeConfig, TreeWriter};

pub fn run(
    pattern: &str,
    path: &Path,
    show_all: bool,
    max_depth: usize,
    output: Option<&str>,
    json: bool,
) -> Result<()> {
    let config = TreeConfig {
        path,
        max_depth,
        show_all,
        search_pattern: Some(pattern),
        spinner_stop: None,
        json_output: json,
    };

    if let Some(output_path) = output {
        // Write to file without spinner
        TreeWriter::for_file().write_to_file(output_path, &config)
    } else {
        // Terminal output with spinner
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);

        let spinner_handle = std::thread::spawn(move || {
            use std::time::Duration;
            let frames = ["|", "/", "-", "\\"];
            let mut i = 0usize;
            eprint!("\x1B[?25l"); // Hide cursor
            let _ = std::io::stderr().flush();
            while !stop_flag.load(Ordering::Relaxed) {
                eprint!("\rSearching... {}", frames[i % frames.len()]);
                let _ = std::io::stderr().flush();
                i = i.wrapping_add(1);
                std::thread::sleep(Duration::from_millis(120));
            }
            eprint!("\r                \r\x1B[?25h"); // Clear and show cursor
            let _ = std::io::stderr().flush();
        });

        let config_with_spinner = TreeConfig {
            spinner_stop: Some(Arc::clone(&stop)),
            json_output: json,
            ..config
        };

        let res = TreeWriter::for_terminal().write_to_terminal(&config_with_spinner);
        stop.store(true, Ordering::Relaxed);
        let _ = spinner_handle.join();
        res
    }
}

use std::process::Command;
use std::io::{self};
use std::thread;
use std::time::Duration;

pub fn run_fastapi_container() -> io::Result<()> {
    // Run the FastAPI Docker container
    let output = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("-p")
        .arg("8000:8000")
        .arg("fastapi-server")
        .output()
        .expect("Failed to start FastAPI Docker container");

    if output.status.success() {
        println!("Successfully started FastAPI container");
    } else {
        eprintln!(
            "Error starting FastAPI container: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to start container"));
    }

    // Retry logic for checking FastAPI connectivity
    let mut retries = 5; // Number of retries
    let mut success = false;

    while retries > 0 {
        // Wait for 2 seconds before making the request
        thread::sleep(Duration::from_secs(2));

        // Make a curl request to test FastAPI connectivity
        let curl_output = Command::new("curl")
            .arg("http://localhost:8000")
            .output()
            .expect("Failed to make curl request");

        if curl_output.status.success() {
            let response = String::from_utf8_lossy(&curl_output.stdout);
            println!("FastAPI server response: {}", response);
            success = true;
            break;
        } else {
            eprintln!(
                "Failed to connect to FastAPI server: {}. Retrying...",
                String::from_utf8_lossy(&curl_output.stderr)
            );
        }

        retries -= 1;
    }

    if !success {
        eprintln!("Failed to connect to FastAPI server after multiple attempts.");
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to connect to FastAPI server"));
    }

    Ok(())
}

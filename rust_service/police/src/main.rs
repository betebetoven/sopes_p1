use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process::Command;
use serde::{Deserialize, Serialize};
use chrono::Local; // For timestamps
mod docker_manager;

// Helper function to deserialize strings to u64
fn deserialize_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

// Helper function to deserialize strings to u32
fn deserialize_from_str_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u32>().map_err(serde::de::Error::custom)
}

// Helper function to deserialize strings to f64
fn deserialize_from_str_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ContainerProcess {
    #[serde(deserialize_with = "deserialize_from_str_u32")]
    pid: u32,
    
    process_name: String,
    container_id: String,
    
    #[serde(deserialize_with = "deserialize_from_str")]
    vsz_kb: u64,
    
    #[serde(deserialize_with = "deserialize_from_str")]
    rss_kb: u64,
    
    #[serde(deserialize_with = "deserialize_from_str_f64")]
    memory_usage_percent: f64,
    
    #[serde(deserialize_with = "deserialize_from_str_f64")]
    cpu_usage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContainerMemInfo {
    #[serde(deserialize_with = "deserialize_from_str")]
    total_memory_kb: u64,
    
    #[serde(deserialize_with = "deserialize_from_str")]
    free_memory_kb: u64,
    
    #[serde(deserialize_with = "deserialize_from_str")]
    used_memory_kb: u64,
    
    processes: Vec<ContainerProcess>,
}

// Function to stop a container
fn stop_container(container_id: &str) {
    let output = Command::new("docker")
        .arg("stop")
        .arg(container_id)
        .output()
        .expect("Failed to execute docker stop");

    if output.status.success() {
        println!("Successfully stopped container with ID: {}", container_id);
    } else {
        eprintln!(
            "Failed to stop container with ID: {}. Error: {}",
            container_id,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// Function to log memory information
fn log_memory_info(total_memory: u64, free_memory: u64, used_memory: u64) -> std::io::Result<()> {
    let now = Local::now();
    let log_entry = format!(
        "{} - Total Memory: {} KB, Free Memory: {} KB, Used Memory: {} KB\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        total_memory,
        free_memory,
        used_memory
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("memory_log.txt")?;
    file.write_all(log_entry.as_bytes())?;
    Ok(())
}

// Function to log container process information
fn log_process_info(container_id: &str, action: &str) -> std::io::Result<()> {
    let now = Local::now();
    let log_entry = format!(
        "{} - Container ID: {} was {}\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        container_id,
        action
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("process_log.txt")?;
    file.write_all(log_entry.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Run FastAPI container and get the container ID
    let fastapi_container_id = docker_manager::run_fastapi_container()?;

    // Open the file
    let mut file = File::open("/proc/container_info_201903553")?;
    let mut contents = String::new();
    
    // Read the file contents into a string
    file.read_to_string(&mut contents)?;
    
    // Parse the JSON data
    let parsed_data: ContainerMemInfo = serde_json::from_str(&mut contents).expect("Failed to parse JSON");

    // Print and log system memory information
    println!("====== System Memory Information ======");
    println!("Total RAM: {} KB", parsed_data.total_memory_kb);
    println!("Free RAM: {} KB", parsed_data.free_memory_kb);
    println!("Used RAM: {} KB", parsed_data.used_memory_kb);
    
    log_memory_info(parsed_data.total_memory_kb, parsed_data.free_memory_kb, parsed_data.used_memory_kb)?;

    // Initialize vectors for low-performance and high-performance containers
    let mut low_performance_containers = vec![];
    let mut high_performance_containers = vec![];
    let mut eliminated_containers = vec![];

    // Analyze each container based on CPU and memory usage
    for process in &parsed_data.processes {
        // Ensure FastAPI container is skipped in performance analysis
        if process.container_id == fastapi_container_id {
            continue;
        }

        if process.cpu_usage_percent == 0.00 {
            low_performance_containers.push(process.clone());
        } else if process.cpu_usage_percent <= 0.09 && process.memory_usage_percent <= 0.16 {
            low_performance_containers.push(process.clone());
        } else {
            high_performance_containers.push(process.clone());
        }
    }

    // Sort the containers by CPU usage (you can modify this to sort by memory, VSZ, or RSS)
    low_performance_containers.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());
    high_performance_containers.sort_by(|a, b| b.cpu_usage_percent.partial_cmp(&a.cpu_usage_percent).unwrap());

    // Limit to 2 high-performance containers and 3 low-performance containers
    if high_performance_containers.len() > 2 {
        let excess_high_performance = &high_performance_containers[2..];
        for process in excess_high_performance {
            stop_container(&process.container_id);
            log_process_info(&process.container_id, "stopped")?;
            eliminated_containers.push(process.clone());
        }
        high_performance_containers.truncate(2);
    }

    if low_performance_containers.len() > 3 {
        let excess_low_performance = &low_performance_containers[3..];
        for process in excess_low_performance {
            stop_container(&process.container_id);
            log_process_info(&process.container_id, "stopped")?;
            eliminated_containers.push(process.clone());
        }
        low_performance_containers.truncate(3);
    }

    // Print high-performance containers
    println!("\n====== High-Performance Containers (Max 2) ======");
    for process in &high_performance_containers {
        println!("PID: {}", process.pid);
        println!("Name: {}", process.process_name);
        println!("Container ID: {}", process.container_id);
        println!("Vsz: {} KB", process.vsz_kb);
        println!("Rss: {} KB", process.rss_kb);
        println!("Memory Usage: {:.2}% of system", process.memory_usage_percent);
        println!("CPU Usage: {:.2}%\n", process.cpu_usage_percent);
    }

    // Print low-performance containers
    println!("\n====== Low-Performance Containers (Max 3) ======");
    for process in &low_performance_containers {
        println!("PID: {}", process.pid);
        println!("Name: {}", process.process_name);
        println!("Container ID: {}", process.container_id);
        println!("Vsz: {} KB", process.vsz_kb);
        println!("Rss: {} KB", process.rss_kb);
        println!("Memory Usage: {:.2}% of system", process.memory_usage_percent);
        println!("CPU Usage: {:.2}%\n", process.cpu_usage_percent);
    }

    // Print eliminated containers
    if !eliminated_containers.is_empty() {
        println!("\n====== Eliminated Containers ======");
        for process in &eliminated_containers {
            println!("PID: {}", process.pid);
            println!("Name: {}", process.process_name);
            println!("Container ID: {}", process.container_id);
            println!("Vsz: {} KB", process.vsz_kb);
            println!("Rss: {} KB", process.rss_kb);
            println!("Memory Usage: {:.2}% of system", process.memory_usage_percent);
            println!("CPU Usage: {:.2}%\n", process.cpu_usage_percent);
        }
    }

    // Print FastAPI container as a side service
    println!("\n====== Side Service: FastAPI Container ======");
    println!("FastAPI Container ID: {}", fastapi_container_id);

    Ok(())
}

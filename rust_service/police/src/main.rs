use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SystemMemory {
    total_ram: u64,
    free_ram: u64,
    used_ram: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContainerProcess {
    pid: u32,
    name: String,
    vsz: u64,
    rss: u64,
    memory_usage: u64,
    cpu_usage: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContainerMemInfo {
    system_memory: SystemMemory,
    container_processes: Vec<ContainerProcess>,
}

fn main() -> std::io::Result<()> {
    // Open the file
    let mut file = File::open("/proc/container_meminfo")?;
    let mut contents = String::new();
    
    // Read the file contents into a string
    file.read_to_string(&mut contents)?;
    
    // Parse the JSON data
    let parsed_data: ContainerMemInfo = serde_json::from_str(&contents).expect("Failed to parse JSON");

    // Print the system memory information
    println!("System Memory Information:");
    println!("Total RAM: {} KB", parsed_data.system_memory.total_ram);
    println!("Free RAM: {} KB", parsed_data.system_memory.free_ram);
    println!("Used RAM: {} KB", parsed_data.system_memory.used_ram);

    // Print the container processes with updated memory and CPU usage calculations
    println!("\nContainer Processes:");
    for process in parsed_data.container_processes {
        // Calculate memory usage percentage
        let memory_usage_percent = (process.rss as f64 / parsed_data.system_memory.total_ram as f64) * 100.0;

        // Convert CPU usage from nanoseconds to milliseconds
        let cpu_usage_ms = process.cpu_usage as f64 / 1_000_000.0;

        // Print container process info with formatted memory and CPU usage
        println!("PID: {}", process.pid);
        println!("Name: {}", process.name);
        println!("Vsz: {} KB", process.vsz);
        println!("Rss: {} KB", process.rss);
        println!("Memory Usage: {:.2}%", memory_usage_percent);
        println!("CPU Usage: {:.2} ms\n", cpu_usage_ms);
    }

    Ok(())
}

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

    // Initialize vectors for high-performance and low-performance containers
    let mut high_performance_containers = vec![];
    let mut low_performance_containers = vec![];

    // Threshold for high-performance (adjust based on limits you defined)
    let high_cpu_threshold = 0.05;  // 25% of a core
    let high_memory_threshold = 64 * 1024;  // 256MB in KB

    // Analyze each container
    for process in parsed_data.container_processes {
        // Calculate memory usage percentage
        let memory_usage_percent = (process.rss as f64 / parsed_data.system_memory.total_ram as f64) * 100.0;

        // Convert CPU usage from nanoseconds to milliseconds
        let cpu_usage_ms = process.cpu_usage as f64 / 1_000_000.0;

        // Classify as high-performance or low-performance
        if process.rss >= high_memory_threshold || process.cpu_usage as f64 / 1_000_000_000.0 >= high_cpu_threshold {
            high_performance_containers.push(process);
        } else {
            low_performance_containers.push(process);
        }
    }

    // Print high-performance containers
    println!("\nHigh-Performance Containers (>= 0.25 CPUs or >= 256MB RAM):");
    for process in &high_performance_containers {
        let memory_usage_percent = (process.rss as f64 / parsed_data.system_memory.total_ram as f64) * 100.0;
        let cpu_usage_ms = process.cpu_usage as f64 / 1_000_000.0;

        println!("PID: {}", process.pid);
        println!("Name: {}", process.name);
        println!("Vsz: {} KB", process.vsz);
        println!("Rss: {} KB", process.rss);
        println!("Memory Usage: {:.2}%", memory_usage_percent);
        println!("CPU Usage: {:.2} ms\n", cpu_usage_ms);
    }

    // Print low-performance containers
    println!("\nLow-Performance Containers (< 0.25 CPUs and < 256MB RAM):");
    for process in &low_performance_containers {
        let memory_usage_percent = (process.rss as f64 / parsed_data.system_memory.total_ram as f64) * 100.0;
        let cpu_usage_ms = process.cpu_usage as f64 / 1_000_000.0;

        println!("PID: {}", process.pid);
        println!("Name: {}", process.name);
        println!("Vsz: {} KB", process.vsz);
        println!("Rss: {} KB", process.rss);
        println!("Memory Usage: {:.2}%", memory_usage_percent);
        println!("CPU Usage: {:.2} ms\n", cpu_usage_ms);
    }

    Ok(())
}

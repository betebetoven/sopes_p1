use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
struct SystemMemory {
    #[serde(deserialize_with = "deserialize_from_str")]
    total_memory_kb: u64,
    
    #[serde(deserialize_with = "deserialize_from_str")]
    free_memory_kb: u64,
    
    #[serde(deserialize_with = "deserialize_from_str")]
    used_memory_kb: u64,
}

#[derive(Debug, Serialize, Deserialize)]
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

fn main() -> std::io::Result<()> {
    // Open the file
    let mut file = File::open("/proc/container_info_201903553")?;
    let mut contents = String::new();
    
    // Read the file contents into a string
    file.read_to_string(&mut contents)?;
    
    // Parse the JSON data
    let parsed_data: ContainerMemInfo = serde_json::from_str(&mut contents).expect("Failed to parse JSON");

    // Print the system memory information
    println!("System Memory Information:");
    println!("Total RAM: {} KB", parsed_data.total_memory_kb);
    println!("Free RAM: {} KB", parsed_data.free_memory_kb);
    println!("Used RAM: {} KB", parsed_data.used_memory_kb);

    // Initialize vectors for low-performance and high-performance containers
    let mut low_performance_containers = vec![];
    let mut high_performance_containers = vec![];

    // Analyze each container based on CPU and memory usage
    for process in &parsed_data.processes {
        // Identify low-performance containers (CPU usage <= 0.02% or memory usage <= 0.14%)
        // if process.cpu_usage_percent == 0.00 then put it in low_performance_containers
        if process.cpu_usage_percent == 0.00 {
            low_performance_containers.push(process);
        } else
        if process.cpu_usage_percent <= 0.09 && process.memory_usage_percent <= 0.16 {
            low_performance_containers.push(process);
        } else {
            high_performance_containers.push(process);
        }
    }

    // Print low-performance containers
    println!("\nLow-Performance Containers (CPU usage <= 0.02% or Memory usage <= 0.14%):");
    for process in &low_performance_containers {
        println!("PID: {}", process.pid);
        println!("Name: {}", process.process_name);
        println!("Container ID: {}", process.container_id);
        println!("Vsz: {} KB", process.vsz_kb);
        println!("Rss: {} KB", process.rss_kb);
        println!("Memory Usage: {:.2}% of system", process.memory_usage_percent);
        println!("CPU Usage: {:.2}%\n", process.cpu_usage_percent);
    }

    // Print high-performance containers
    println!("\nHigh-Performance Containers (CPU usage > 0.02% and Memory usage > 0.14%):");
    for process in &high_performance_containers {
        println!("PID: {}", process.pid);
        println!("Name: {}", process.process_name);
        println!("Container ID: {}", process.container_id);
        println!("Vsz: {} KB", process.vsz_kb);
        println!("Rss: {} KB", process.rss_kb);
        println!("Memory Usage: {:.2}% of system", process.memory_usage_percent);
        println!("CPU Usage: {:.2}%\n", process.cpu_usage_percent);
    }

    Ok(())
}

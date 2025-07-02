use chrono::Local;
use std::fs;
use std::process::Command;
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, NetworkExt};
use users::get_current_username;

fn main() {
    println!("=========================================");
    println!("       SERVER PERFORMANCE STATS");
    println!("=========================================");
    println!("Generated on: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        println!("Hostname: {}", hostname);
    } else if let Ok(output) = Command::new("hostname").output() {
        println!("Hostname: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    
    println!("=========================================");

    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU Usage
    print_cpu_usage(&mut sys);
    
    // Memory Usage
    print_memory_usage(&sys);
    
    // Disk Usage
    print_disk_usage(&sys);
    
    // Top 5 processes by CPU
    print_top_processes_cpu(&sys);
    
    // Top 5 processes by Memory
    print_top_processes_memory(&sys);
    
    // Additional system information
    print_additional_info(&sys);
    
    println!();
    println!("=========================================");
    println!("       END OF REPORT");
    println!("=========================================");
}

fn print_header(title: &str) {
    println!();
    println!("--- {} ---", title);
}

fn print_cpu_usage(sys: &mut System) {
    print_header("CPU USAGE");
    
    // Refresh CPU info
    sys.refresh_cpu();
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu();
    
    let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
    
    println!("CPU Usage: {:.2}%", cpu_usage);
    println!("CPU Idle: {:.2}%", 100.0 - cpu_usage);
    println!("CPU Cores: {}", sys.cpus().len());
}

fn print_memory_usage(sys: &System) {
    print_header("MEMORY USAGE");
    
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let available_memory = sys.available_memory();
    
    let used_percent = (used_memory as f64 / total_memory as f64) * 100.0;
    let available_percent = (available_memory as f64 / total_memory as f64) * 100.0;
    
    println!("Total Memory: {:.2} GB", bytes_to_gb(total_memory));
    println!("Used Memory: {:.2} GB ({:.2}%)", bytes_to_gb(used_memory), used_percent);
    println!("Available Memory: {:.2} GB ({:.2}%)", bytes_to_gb(available_memory), available_percent);
    
    // Swap information
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    
    if total_swap > 0 {
        let swap_percent = (used_swap as f64 / total_swap as f64) * 100.0;
        println!("Total Swap: {:.2} GB", bytes_to_gb(total_swap));
        println!("Used Swap: {:.2} GB ({:.2}%)", bytes_to_gb(used_swap), swap_percent);
    } else {
        println!("Swap: Not configured");
    }
}

fn print_disk_usage(sys: &System) {
    print_header("DISK USAGE");
    
    println!("{:<20} {:<10} {:<10} {:<10} {:<8} {}", 
             "Filesystem", "Size", "Used", "Available", "Use%", "Mounted on");
    
    for disk in sys.disks() {
        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space - available_space;
        let used_percent = if total_space > 0 {
            (used_space as f64 / total_space as f64) * 100.0
        } else {
            0.0
        };
        
        println!("{:<20} {:<10} {:<10} {:<10} {:<7.1}% {}", 
                 disk.name().to_string_lossy(),
                 format!("{:.1}G", bytes_to_gb(total_space)),
                 format!("{:.1}G", bytes_to_gb(used_space)),
                 format!("{:.1}G", bytes_to_gb(available_space)),
                 used_percent,
                 disk.mount_point().to_string_lossy());
    }
}

fn print_top_processes_cpu(sys: &System) {
    print_header("TOP 5 PROCESSES BY CPU USAGE");
    
    let mut processes: Vec<_> = sys.processes().values().collect();
    processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
    
    println!("{:<8} {:<12} {:<8} {}", "PID", "USER", "CPU%", "COMMAND");
    
    for process in processes.iter().take(5) {
        let user = get_process_user(process.pid().as_u32());
        println!("{:<8} {:<12} {:<7.2} {}", 
                 process.pid(),
                 user,
                 process.cpu_usage(),
                 process.name());
    }
}

fn print_top_processes_memory(sys: &System) {
    print_header("TOP 5 PROCESSES BY MEMORY USAGE");
    
    let mut processes: Vec<_> = sys.processes().values().collect();
    processes.sort_by(|a, b| b.memory().cmp(&a.memory()));
    
    println!("{:<8} {:<12} {:<8} {:<10} {}", "PID", "USER", "MEM%", "MEMORY", "COMMAND");
    
    let total_memory = sys.total_memory() as f64;
    
    for process in processes.iter().take(5) {
        let user = get_process_user(process.pid().as_u32());
        let memory_percent = (process.memory() as f64 / total_memory) * 100.0;
        println!("{:<8} {:<12} {:<7.2} {:<10} {}", 
                 process.pid(),
                 user,
                 memory_percent,
                 format!("{:.1}M", process.memory() as f64 / 1024.0 / 1024.0),
                 process.name());
    }
}

fn print_additional_info(sys: &System) {
    print_header("ADDITIONAL SYSTEM INFORMATION");
    
    // OS Information
    println!("OS: {} {}", sys.name().unwrap_or("Unknown".to_string()), 
             sys.os_version().unwrap_or("Unknown".to_string()));
    println!("Kernel: {}", sys.kernel_version().unwrap_or("Unknown".to_string()));
    
    // System uptime
    let uptime_seconds = sys.uptime();
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    println!("Uptime: {} days, {} hours, {} minutes", days, hours, minutes);
    
    // Load average
    let load_avg = sys.load_average();
    println!("Load Average: {:.2}, {:.2}, {:.2}", load_avg.one, load_avg.five, load_avg.fifteen);
    
    // Load per core
    let load_per_core = load_avg.one / sys.cpus().len() as f64;
    println!("Load per core: {:.2}", load_per_core);
    
    // Network interfaces
    print_network_info(sys);
    
    // Logged in users
    print_logged_users();
    
    // Boot time
    println!("Boot time: {}", 
             chrono::DateTime::from_timestamp(sys.boot_time() as i64, 0)
                 .unwrap_or_default()
                 .format("%Y-%m-%d %H:%M:%S"));
}

fn print_network_info(sys: &System) {
    println!();
    println!("Network Interfaces:");
    
    for (interface_name, network) in sys.networks() {
        println!("  {}: RX: {:.2} MB, TX: {:.2} MB", 
                 interface_name,
                 network.received() as f64 / 1024.0 / 1024.0,
                 network.transmitted() as f64 / 1024.0 / 1024.0);
    }
    
    // Count listening ports
    if let Ok(output) = Command::new("netstat").args(&["-tuln"]).output() {
        let netstat_output = String::from_utf8_lossy(&output.stdout);
        let listening_ports = netstat_output.lines()
            .filter(|line| line.contains("LISTEN"))
            .count();
        println!("Listening ports: {}", listening_ports);
    }
}

fn print_logged_users() {
    println!();
    println!("Currently Logged in Users:");
    
    if let Ok(output) = Command::new("who").output() {
        let who_output = String::from_utf8_lossy(&output.stdout);
        let user_count = who_output.lines().count();
        
        for line in who_output.lines().take(10) {
            println!("  {}", line);
        }
        
        println!("Total logged in users: {}", user_count);
    } else {
        println!("  Unable to retrieve user information");
    }
    
    // Failed login attempts
    println!();
    println!("Recent Failed Login Attempts:");
    if let Ok(output) = Command::new("lastb").args(&["-n", "5"]).output() {
        let lastb_output = String::from_utf8_lossy(&output.stdout);
        if !lastb_output.trim().is_empty() {
            for line in lastb_output.lines().take(5) {
                if !line.trim().is_empty() && !line.starts_with("btmp begins") {
                    println!("  {}", line);
                }
            }
        } else {
            println!("  No failed login attempts found");
        }
    } else {
        println!("  Unable to retrieve failed login information (may require sudo)");
    }
}

fn get_process_user(pid: u32) -> String {
    // Try to get user from /proc/PID/status
    if let Ok(status) = fs::read_to_string(format!("/proc/{}/status", pid)) {
        for line in status.lines() {
            if line.starts_with("Uid:") {
                if let Some(uid_str) = line.split_whitespace().nth(1) {
                    if let Ok(uid) = uid_str.parse::<u32>() {
                        if let Some(user) = users::get_user_by_uid(uid) {
                            return user.name().to_string_lossy().to_string();
                        }
                    }
                }
                break;
            }
        }
    }
    "unknown".to_string()
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}
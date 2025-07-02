#!/bin/bash

# server-stats.sh - Server Performance Statistics Script
# Usage: ./server-stats.sh

echo "========================================="
echo "       SERVER PERFORMANCE STATS"
echo "========================================="
echo "Generated on: $(date)"
echo "Hostname: $(hostname)"
echo "========================================="

# Function to print section headers
print_header() {
    echo ""
    echo "--- $1 ---"
}

# 1. Total CPU Usage
print_header "CPU USAGE"
cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | sed 's/%us,//')
cpu_idle=$(top -bn1 | grep "Cpu(s)" | awk '{print $8}' | sed 's/%id,//')
cpu_used=$(echo "100 - $cpu_idle" | bc -l 2>/dev/null || echo "scale=2; 100 - $cpu_idle" | bc)
echo "CPU Usage: ${cpu_used}%"
echo "CPU Idle: ${cpu_idle}%"

# Alternative method if top format differs
if [ -z "$cpu_used" ] || [ "$cpu_used" = "100 - " ]; then
    cpu_usage_alt=$(vmstat 1 2 | tail -1 | awk '{print 100-$15}')
    echo "CPU Usage (alternative): ${cpu_usage_alt}%"
fi

# 2. Memory Usage
print_header "MEMORY USAGE"
memory_info=$(free -h)
echo "$memory_info"
echo ""

# Memory percentage calculation
memory_stats=$(free | grep '^Mem:')
total_mem=$(echo $memory_stats | awk '{print $2}')
used_mem=$(echo $memory_stats | awk '{print $3}')
free_mem=$(echo $memory_stats | awk '{print $4}')
available_mem=$(echo $memory_stats | awk '{print $7}')

mem_used_percent=$(echo "scale=2; ($used_mem * 100) / $total_mem" | bc)
mem_free_percent=$(echo "scale=2; ($available_mem * 100) / $total_mem" | bc)

echo "Memory Usage: ${mem_used_percent}%"
echo "Memory Available: ${mem_free_percent}%"

# 3. Disk Usage
print_header "DISK USAGE"
echo "Filesystem usage:"
df -h | grep -E '^/dev/' | while read line; do
    echo "$line"
done

echo ""
echo "Summary of main partitions:"
df -h | grep -E '^/dev/' | awk '{print $6 " - Used: " $5 " (" $3 "/" $2 ")"}'

# 4. Top 5 Processes by CPU Usage
print_header "TOP 5 PROCESSES BY CPU USAGE"
echo "PID       USER      CPU%    COMMAND"
ps aux --sort=-%cpu | head -6 | tail -5 | awk '{printf "%-8s %-10s %-7s %s\n", $2, $1, $3, $11}'

# 5. Top 5 Processes by Memory Usage
print_header "TOP 5 PROCESSES BY MEMORY USAGE"
echo "PID       USER      MEM%    COMMAND"
ps aux --sort=-%mem | head -6 | tail -5 | awk '{printf "%-8s %-10s %-7s %s\n", $2, $1, $4, $11}'

# STRETCH GOALS - Additional Stats
print_header "ADDITIONAL SYSTEM INFORMATION"

# OS Version
echo "OS Version:"
if [ -f /etc/os-release ]; then
    . /etc/os-release
    echo "  $PRETTY_NAME"
elif [ -f /etc/redhat-release ]; then
    echo "  $(cat /etc/redhat-release)"
elif [ -f /etc/debian_version ]; then
    echo "  Debian $(cat /etc/debian_version)"
else
    echo "  $(uname -s) $(uname -r)"
fi

# Kernel Version
echo "Kernel: $(uname -r)"

# System Uptime
echo "Uptime: $(uptime -p 2>/dev/null || uptime | awk -F'up ' '{print $2}' | awk -F',' '{print $1}')"

# Load Average
echo "Load Average: $(uptime | awk -F'load average:' '{print $2}')"

# Logged in Users
echo ""
echo "Currently Logged in Users:"
who | awk '{print "  " $1 " - " $3 " " $4 " (from " $5 ")"}'

# User count
user_count=$(who | wc -l)
echo "Total logged in users: $user_count"

# Failed Login Attempts (last 10)
print_header "RECENT FAILED LOGIN ATTEMPTS"
if command -v lastb >/dev/null 2>&1; then
    echo "Last 10 failed login attempts:"
    lastb -n 10 2>/dev/null | head -10 || echo "No failed login attempts found or insufficient permissions"
else
    echo "lastb command not available"
fi

# Network Connections
print_header "NETWORK CONNECTIONS"
echo "Active network connections:"
netstat -tuln 2>/dev/null | grep LISTEN | wc -l | awk '{print "Listening ports: " $1}'

# System Load and CPU cores
echo ""
echo "CPU Cores: $(nproc)"
echo "Load per core: $(echo "scale=2; $(uptime | awk -F'load average:' '{print $2}' | awk -F',' '{print $1}') / $(nproc)" | bc 2>/dev/null || echo "N/A")"

# Swap Usage
print_header "SWAP USAGE"
swap_info=$(free -h | grep '^Swap:')
echo "$swap_info"

if [ "$(echo $swap_info | awk '{print $2}')" != "0B" ]; then
    swap_stats=$(free | grep '^Swap:')
    total_swap=$(echo $swap_stats | awk '{print $2}')
    used_swap=$(echo $swap_stats | awk '{print $3}')
    if [ $total_swap -gt 0 ]; then
        swap_used_percent=$(echo "scale=2; ($used_swap * 100) / $total_swap" | bc)
        echo "Swap Usage: ${swap_used_percent}%"
    fi
fi

echo ""
echo "========================================="
echo "       END OF REPORT"
echo "========================================="
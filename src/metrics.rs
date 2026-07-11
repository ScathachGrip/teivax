use std::sync::Arc;
use std::time::Duration;

use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use sysinfo::{Disks, Networks, Pid, ProcessStatus, System};
use tokio::sync::Mutex;
use tokio::time::interval;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub fn describe_metrics() {
    // HTTP
    describe_counter!("http_requests_total", "Total HTTP requests");
    describe_histogram!("http_requests_duration_seconds", "Request latency");

    // System CPU
    describe_gauge!("system_cpu_usage_percent", "System-wide CPU usage");
    describe_gauge!("system_cpu_core_count", "Logical CPU cores");
    describe_gauge!("system_cpu_brand", "CPU brand string (1 if present)");

    // System memory
    describe_gauge!("system_memory_used_bytes", "Used system memory");
    describe_gauge!("system_memory_total_bytes", "Total system memory");
    describe_gauge!("system_memory_available_bytes", "Available system memory");
    describe_gauge!("system_memory_free_bytes", "Free system memory");
    describe_gauge!("system_memory_used_percent", "Memory used percent");
    describe_gauge!("system_swap_used_bytes", "Used swap");
    describe_gauge!("system_swap_total_bytes", "Total swap");

    // Load average
    describe_gauge!("system_load1", "1-minute load average");
    describe_gauge!("system_load5", "5-minute load average");
    describe_gauge!("system_load15", "15-minute load average");

    // Network
    describe_gauge!("system_network_rx_bytes_total", "Cumulative network rx");
    describe_gauge!("system_network_tx_bytes_total", "Cumulative network tx");
    describe_gauge!("system_network_rx_per_sec", "Network rx rate (bytes/s)");
    describe_gauge!("system_network_tx_per_sec", "Network tx rate (bytes/s)");
    describe_gauge!("system_network_interfaces_total", "Total NICs");

    // Filesystems
    describe_gauge!("system_disk_total_bytes", "Total disk space across mounts");
    describe_gauge!("system_disk_available_bytes", "Available disk space");
    describe_gauge!("system_disk_used_bytes", "Used disk space");
    describe_gauge!("system_disk_used_percent", "Disk used percent");

    // Uptime
    describe_gauge!("system_uptime_seconds", "Host uptime in seconds");
    describe_gauge!("process_uptime_seconds", "Seconds since process start");
    describe_gauge!(
        "process_started_timestamp_seconds",
        "Process start UNIX time"
    );

    // Process
    describe_gauge!("process_cpu_usage_percent", "Process CPU usage");
    describe_gauge!("process_memory_rss_bytes", "Process RSS");
    describe_gauge!("process_memory_virtual_bytes", "Process virtual memory");
    describe_gauge!("process_memory_used_bytes", "Process used memory");
    describe_gauge!("process_threads", "Process thread count");
    describe_gauge!("process_open_fds", "Process open file descriptors");
    describe_gauge!("process_status_running", "1 if running");
    describe_gauge!(
        "process_disk_read_bytes_total",
        "Cumulative disk reads by process"
    );
    describe_gauge!(
        "process_disk_written_bytes_total",
        "Cumulative disk writes by process"
    );

    // Tokio runtime
    describe_gauge!("tokio_workers_count", "Tokio worker threads (set at boot)");

    // App-specific
    describe_gauge!("anime_tags_count", "Number of registered tags per anime");
    describe_counter!("json_dump_total", "JSON dumps performed");
    describe_gauge!("json_dump_last_unix_seconds", "Last JSON dump UNIX time");
    describe_histogram!("json_dump_duration_seconds", "JSON dump duration");

    // mimalloc — real GC-equivalent memory stats via mi_stats_get_json
    describe_gauge!(
        "mimalloc_commit_current_bytes",
        "Currently committed (in-use) bytes"
    );
    describe_gauge!(
        "mimalloc_commit_peak_bytes",
        "Peak committed bytes since boot"
    );
    describe_gauge!(
        "mimalloc_rss_current_bytes",
        "Current RSS allocated by mimalloc"
    );
    describe_gauge!("mimalloc_rss_peak_bytes", "Peak RSS allocated by mimalloc");
    describe_gauge!(
        "mimalloc_reserved_current_bytes",
        "Currently reserved (mapped) bytes"
    );
    describe_gauge!(
        "mimalloc_malloc_huge_count_total",
        "Cumulative huge malloc calls"
    );
    describe_gauge!(
        "mimalloc_page_faults_total",
        "Cumulative page faults from mimalloc"
    );
    describe_gauge!("mimalloc_mmap_calls_total", "Cumulative mmap calls");
    describe_gauge!("mimalloc_commit_calls_total", "Cumulative commit calls");
    describe_gauge!("mimalloc_purge_calls_total", "Cumulative purge calls");
    describe_gauge!("mimalloc_arena_count", "Number of mimalloc arenas");
    describe_gauge!("mimalloc_heap_count", "Number of mimalloc heaps");
    describe_gauge!("mimalloc_thread_count", "Number of threads using mimalloc");
    describe_gauge!(
        "mimalloc_fragmentation_bytes",
        "Reserved − committed (waste)"
    );
    describe_gauge!(
        "mimalloc_fragmentation_ratio",
        "Committed / reserved (efficiency)"
    );
}

pub fn spawn_system_updater() {
    let sys = Arc::new(Mutex::new(System::new_all()));
    let networks = Arc::new(Mutex::new(Networks::new_with_refreshed_list()));
    let disks = Arc::new(Mutex::new(Disks::new_with_refreshed_list()));
    let pid = Pid::from_u32(std::process::id());

    // Static process info
    let started_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    gauge!("process_started_timestamp_seconds").set(started_unix);

    // Tokio worker thread count (set once, stable)
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        gauge!("tokio_workers_count").set(handle.metrics().num_workers() as f64);
    }

    // Prime CPU usage (sysinfo needs 2 samples for non-zero readings)
    {
        let mut s = sys.try_lock().expect("init lock");
        s.refresh_cpu_usage();
    }

    let host_boot = System::uptime();
    gauge!("system_uptime_seconds").set(host_boot as f64);

    let mut last_rx = 0u64;
    let mut last_tx = 0u64;
    let mut last_sample = std::time::Instant::now();

    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(2));
        loop {
            tick.tick().await;
            let now = std::time::Instant::now();
            let dt = now.duration_since(last_sample).as_secs_f64().max(0.001);
            last_sample = now;

            // CPU + memory
            {
                let mut s = sys.lock().await;
                s.refresh_cpu_usage();
                s.refresh_memory();

                gauge!("system_cpu_usage_percent").set(s.global_cpu_usage() as f64);
                gauge!("system_cpu_core_count").set(s.cpus().len() as f64);
                gauge!("system_memory_used_bytes").set(s.used_memory() as f64);
                gauge!("system_memory_total_bytes").set(s.total_memory() as f64);
                gauge!("system_memory_available_bytes").set(s.available_memory() as f64);
                gauge!("system_memory_free_bytes").set(s.free_memory() as f64);
                let used_pct = if s.total_memory() > 0 {
                    (s.used_memory() as f64 / s.total_memory() as f64) * 100.0
                } else {
                    0.0
                };
                gauge!("system_memory_used_percent").set(used_pct);
                gauge!("system_swap_used_bytes").set(s.used_swap() as f64);
                gauge!("system_swap_total_bytes").set(s.total_swap() as f64);

                let load = System::load_average();
                gauge!("system_load1").set(load.one as f64);
                gauge!("system_load5").set(load.five as f64);
                gauge!("system_load15").set(load.fifteen as f64);

                // Disks
                {
                    let mut d = disks.lock().await;
                    d.refresh();
                    let mut total = 0u64;
                    let mut avail = 0u64;
                    for disk in d.iter() {
                        total = total.saturating_add(disk.total_space());
                        avail = avail.saturating_add(disk.available_space());
                    }
                    let used = total.saturating_sub(avail);
                    gauge!("system_disk_total_bytes").set(total as f64);
                    gauge!("system_disk_available_bytes").set(avail as f64);
                    gauge!("system_disk_used_bytes").set(used as f64);
                    let disk_pct = if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    };
                    gauge!("system_disk_used_percent").set(disk_pct);
                }

                // Process
                s.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::Some(&[pid]),
                    true,
                    sysinfo::ProcessRefreshKind::everything(),
                );
                if let Some(proc_) = s.process(pid) {
                    gauge!("process_cpu_usage_percent").set(proc_.cpu_usage() as f64);
                    gauge!("process_memory_rss_bytes").set(proc_.memory() as f64);
                    gauge!("process_memory_virtual_bytes").set(proc_.virtual_memory() as f64);
                    gauge!("process_threads")
                        .set(proc_.tasks().map(|t| t.len() as f64).unwrap_or(0.0));
                    gauge!("process_disk_read_bytes_total")
                        .set(proc_.disk_usage().total_read_bytes as f64);
                    gauge!("process_disk_written_bytes_total")
                        .set(proc_.disk_usage().total_written_bytes as f64);
                    gauge!("process_status_running").set(if proc_.status() == ProcessStatus::Run {
                        1.0
                    } else {
                        0.0
                    });
                    #[cfg(target_os = "linux")]
                    {
                        gauge!("process_open_fds")
                            .set(proc_.open_files().map(|f| f.len() as f64).unwrap_or(0.0));
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        gauge!("process_open_fds").set(0.0);
                    }
                }
            }

            // Network
            {
                let mut n = networks.lock().await;
                n.refresh();
                let mut rx = 0u64;
                let mut tx = 0u64;
                let total = n.iter().count() as u64;
                for (_, iface) in n.iter() {
                    rx = rx.saturating_add(iface.total_received());
                    tx = tx.saturating_add(iface.total_transmitted());
                }
                gauge!("system_network_rx_bytes_total").set(rx as f64);
                gauge!("system_network_tx_bytes_total").set(tx as f64);
                let drx = rx.saturating_sub(last_rx) as f64 / dt;
                let dtx = tx.saturating_sub(last_tx) as f64 / dt;
                gauge!("system_network_rx_per_sec").set(drx);
                gauge!("system_network_tx_per_sec").set(dtx);
                gauge!("system_network_interfaces_total").set(total as f64);
                last_rx = rx;
                last_tx = tx;
            }

            // mimalloc — real GC-equivalent allocator stats via mi_stats_get_json
            {
                if let Ok(s) = mimalloc::MiMalloc::stats_json() {
                    if let Some(s) = s.to_str().ok() {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
                            let p = |k: &str| {
                                v.get("process")
                                    .and_then(|x| x.get(k))
                                    .and_then(|x| x.as_u64())
                                    .unwrap_or(0) as f64
                            };
                            let r = |k: &str| {
                                v.get(k)
                                    .and_then(|x| x.get("current"))
                                    .and_then(|x| x.as_u64())
                                    .unwrap_or(0) as f64
                            };
                            let num =
                                |k: &str| v.get(k).and_then(|x| x.as_u64()).unwrap_or(0) as f64;
                            let commit = p("commit_current");
                            let commit_peak = p("commit_peak");
                            let rss = p("rss_current");
                            let rss_peak = p("rss_peak");
                            let reserved = r("reserved");
                            let huge = num("malloc_huge_count");
                            let faults = p("page_faults");
                            let mmap = num("mmap_calls");
                            let ccalls = num("commit_calls");
                            let pcalls = num("purge_calls");
                            let arenas = num("arena_count");
                            let heaps = r("heaps");
                            let threads = r("threads");
                            gauge!("mimalloc_commit_current_bytes").set(commit);
                            gauge!("mimalloc_commit_peak_bytes").set(commit_peak);
                            gauge!("mimalloc_rss_current_bytes").set(rss);
                            gauge!("mimalloc_rss_peak_bytes").set(rss_peak);
                            gauge!("mimalloc_reserved_current_bytes").set(reserved);
                            gauge!("mimalloc_malloc_huge_count_total").set(huge);
                            gauge!("mimalloc_page_faults_total").set(faults);
                            gauge!("mimalloc_mmap_calls_total").set(mmap);
                            gauge!("mimalloc_commit_calls_total").set(ccalls);
                            gauge!("mimalloc_purge_calls_total").set(pcalls);
                            gauge!("mimalloc_arena_count").set(arenas);
                            gauge!("mimalloc_heap_count").set(heaps);
                            gauge!("mimalloc_thread_count").set(threads);
                            let frag = (reserved - commit).max(0.0);
                            gauge!("mimalloc_fragmentation_bytes").set(frag);
                            gauge!("mimalloc_fragmentation_ratio").set(if reserved > 0.0 {
                                commit / reserved
                            } else {
                                0.0
                            });
                        }
                    }
                }
            }
        }
    });
}

/// Track JSON dump timing + counts. Called from main.
pub fn record_json_dump(name: &str, duration_secs: f64) {
    counter!("json_dump_total", "target" => name.to_string()).increment(1);
    histogram!("json_dump_duration_seconds", "target" => name.to_string()).record(duration_secs);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    gauge!("json_dump_last_unix_seconds").set(now);
}

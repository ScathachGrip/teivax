#![allow(dead_code)]
use std::collections::HashMap;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Audit tags against the Danbooru JSON API (no auth required).
pub fn check_danbooru(tags: &'static [&'static str], name: &str) {
    let (tx, rx) = mpsc::channel();
    let n_threads = 2;
    let chunk_size = (tags.len() + n_threads - 1).max(1);

    for chunk in tags.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let tx = tx.clone();
        thread::spawn(move || {
            let agent = ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(10))
                .build();
            for tag in chunk {
                let url = format!(
                    "https://danbooru.donmai.us/tags.json?search[name]={}",
                    url_encode(tag)
                );
                let count = match agent.get(&url).call() {
                    Ok(resp) => match resp.into_json::<serde_json::Value>() {
                        Ok(json) => json[0]["post_count"].as_u64().unwrap_or(0),
                        Err(_) => 0,
                    },
                    Err(_) => 0,
                };
                tx.send((tag.to_owned(), count)).ok();
                thread::sleep(Duration::from_millis(500));
            }
        });
    }
    drop(tx);
    write_report(tags, rx, name);
}

/// Audit tags against the Rule34 XML API (requires RULE34_API_KEY + RULE34_API_ID in .env).
pub fn check_rule34(tags: &'static [&'static str], name: &str) {
    dotenvy::dotenv().ok();
    let api_key =
        std::env::var("RULE34_API_KEY").expect("RULE34_API_KEY not set (add to .env)");
    let user_id =
        std::env::var("RULE34_API_ID").expect("RULE34_API_ID not set (add to .env)");

    let (tx, rx) = mpsc::channel::<(String, u64)>();
    let ak = api_key.clone();
    let ui = user_id.clone();
    thread::spawn(move || {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(10))
            .build();
        for tag in tags {
            let url = format!(
                "https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&api_key={}&user_id={}&limit=0&tags={}",
                ak, ui, url_encode(tag)
            );
            let count = match agent.get(&url).call() {
                Ok(resp) => {
                    let body = resp.into_string().unwrap_or_default();
                    extract_count(&body)
                }
                Err(_) => 0,
            };
            tx.send((tag.to_string(), count)).ok();
            thread::sleep(Duration::from_millis(1100));
        }
    });
    write_report(tags, rx, name);
}

// ── private helpers ──────────────────────────────────────────────

fn write_report(tags: &[&str], rx: mpsc::Receiver<(String, u64)>, name: &str) {
    let outpath = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("check_{name}_output.txt"));
    let mut out = std::fs::File::create(&outpath).expect("create output file");
    let mut w = |line: String| {
        println!("{line}");
        writeln!(out, "{line}").ok();
    };

    let mut counts: HashMap<String, u64> = HashMap::new();
    for (tag, count) in rx {
        counts.insert(tag, count);
    }

    let mut sorted: Vec<(&str, u64)> = tags
        .iter()
        .map(|t| (*t, counts.get(*t).copied().unwrap_or(0)))
        .collect();
    sorted.sort_by_key(|(_, a)| *a);

    w(format!("{:<60} {}", "TAG", "POST_COUNT"));
    w("-".repeat(75));
    for (t, c) in &sorted {
        w(format!("{:<60} {}", t, c));
    }

    w(String::new());
    w("=== SUMMARY ===".into());
    w(format!("Total: {}", tags.len()));
    w(format!("Total posts: {}", sorted.iter().map(|(_, c)| c).sum::<u64>()));
    w(String::new());
    w(format!("Output: {}", outpath.display()));
}

fn extract_count(body: &str) -> u64 {
    for line in body.lines() {
        if let Some(start) = line.find(r#"count=""#) {
            let rest = &line[start + 7..];
            if let Some(end) = rest.find('"') {
                return rest[..end].parse().unwrap_or(0);
            }
        }
    }
    0
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

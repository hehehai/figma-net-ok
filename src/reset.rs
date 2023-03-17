use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const LINUX_HOSTS_PATH: &str = "/etc/hosts";
const MACOS_HOSTS_PATH: &str = "/etc/hosts";
const WINDOWS_HOSTS_PATH: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";

#[inline]
fn get_hosts_path() -> PathBuf {
    match std::env::consts::OS {
        "linux" => Path::new(LINUX_HOSTS_PATH).to_path_buf(),
        "windows" => Path::new(WINDOWS_HOSTS_PATH).to_path_buf(),
        "macos" => Path::new(MACOS_HOSTS_PATH).to_path_buf(),
        _ => panic!("不支持的操作系统！"),
    }
}

fn read_hosts_file() -> std::io::Result<Vec<String>> {
    let file = File::open(get_hosts_path())?;
    let reader = BufReader::new(&file);

    let mut lines = Vec::new();

    for line in reader.lines() {
        lines.push(line?);
    }

    Ok(lines)
}

fn write_hosts_file(lines: &[String]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(get_hosts_path())?;

    let output = lines.join("\n") + "\n";
    file.write_all(output.as_bytes())?;

    Ok(())
}

pub fn reset_host(host_names: &[&str]) -> std::io::Result<()> {
    let mut lines = read_hosts_file()?;

    let mut finded = false;

    lines.retain(|line| {
        if line.starts_with('#') {
            if line == "# set by figma net ok" {
                return false;
            }
            return true;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 0 {
            return true;
        }

        let hostname = parts[1];

        if host_names.contains(&hostname) {
            finded = true;
            return false;
        }

        return true;
    });

    if finded {
        write_hosts_file(&lines)?;
    }

    Ok(())
}

pub fn add_hosts(hosts: &[(String, String)]) -> std::io::Result<()> {
    let mut lines = read_hosts_file()?;
    let mut added = false;

    for (ip, hostname) in hosts {
        let line = format!("{}\t{}", ip, hostname);

        if !lines.contains(&line) {
            if !added {
                lines.push("# set by figma net ok".to_string());
            }
            added = true;
            lines.push(line);
        }
    }

    if added {
        write_hosts_file(&lines)?;
    }

    Ok(())
}

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

fn filter_hosts_lines(lines: &mut Vec<String>, host_names: &[&str]) -> bool {
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

    finded
}

pub fn reset_host(host_names: &[&str]) -> std::io::Result<()> {
    let mut lines = read_hosts_file()?;

    if filter_hosts_lines(&mut lines, host_names) {
        write_hosts_file(&lines)?;
    }

    Ok(())
}

pub fn add_hosts(hosts: &[(String, String)]) -> std::io::Result<()> {
    let mut lines = read_hosts_file()?;
    let added = add_host_lines(&mut lines, hosts);

    if added {
        write_hosts_file(&lines)?;
    }

    Ok(())
}

fn add_host_lines(lines: &mut Vec<String>, hosts: &[(String, String)]) -> bool {
    let mut added = false;
    for (ip, hostname) in hosts {
        let line = format!("{} {}", ip, hostname);

        if !lines.contains(&line) {
            if !added {
                lines.push("# set by figma net ok".to_string());
            }
            added = true;

            lines.push(line);
        }
    }

    added
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_hosts_lines() {
        let mut lines = vec![
            "# set by figma net ok".to_string(),
            "127.0.0.1 localhost".to_string(),
            "::1 localhost".to_string(),
            "192.168.0.1 myrouter".to_string(),
            "192.168.0.2 myprinter".to_string(),
            "192.168.0.3 myserver".to_string(),
            "192.168.0.4 mydatabase".to_string(),
            "fe80::1%lo0 localhost".to_string(),
            "ff02::fb ip6-allnodes".to_string(),
            "ff02::1 ip6-allnodes".to_string(),
            "ff02::2 ip6-allrouters".to_string(),
        ];
        let host_names = &["myprinter", "myserver"];

        let filtered = filter_hosts_lines(&mut lines, host_names);

        assert!(filtered);
        assert_eq!(
            lines,
            vec![
                "127.0.0.1 localhost".to_string(),
                "::1 localhost".to_string(),
                "192.168.0.1 myrouter".to_string(),
                "192.168.0.4 mydatabase".to_string(),
                "fe80::1%lo0 localhost".to_string(),
                "ff02::fb ip6-allnodes".to_string(),
                "ff02::1 ip6-allnodes".to_string(),
                "ff02::2 ip6-allrouters".to_string(),
            ]
        );
    }

    #[test]
    fn test_add_host_lines() {
        let mut lines = vec![
            "127.0.0.1 localhost".to_string(),
            "::1 localhost".to_string(),
            "192.168.0.1 myrouter".to_string(),
            "fe80::1%lo0 localhost".to_string(),
            "ff02::fb ip6-allnodes".to_string(),
            "ff02::1 ip6-allnodes".to_string(),
            "192.168.0.2 myprinter".to_string(),
            "ff02::2 ip6-allrouters".to_string(),
        ];
        let hosts = &[
            ("192.168.0.2".to_string(), "myprinter".to_string()),
            ("192.168.0.3".to_string(), "myserver".to_string()),
            ("192.168.0.4".to_string(), "mydatabase".to_string()),
        ];

        let added = add_host_lines(&mut lines, hosts);

        assert!(added);
        assert_eq!(
            lines,
            vec![
                "127.0.0.1 localhost".to_string(),
                "::1 localhost".to_string(),
                "192.168.0.1 myrouter".to_string(),
                "fe80::1%lo0 localhost".to_string(),
                "ff02::fb ip6-allnodes".to_string(),
                "ff02::1 ip6-allnodes".to_string(),
                "192.168.0.2 myprinter".to_string(),
                "ff02::2 ip6-allrouters".to_string(),
                "# set by figma net ok".to_string(),
                "192.168.0.3 myserver".to_string(),
                "192.168.0.4 mydatabase".to_string(),
            ]
        );
    }
}

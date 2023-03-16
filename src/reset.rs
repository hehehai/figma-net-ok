use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Result, Write},
};

const LINUX_HOSTS_PATH: &str = "/etc/hosts";
const MACOS_HOSTS_PATH: &str = "/etc/hosts";
const WINDOWS_HOSTS_PATH: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";

#[inline]
const fn get_hosts_path() -> &'static str {
    if cfg!(target_os = "linux") {
        LINUX_HOSTS_PATH
    } else if cfg!(target_os = "windows") {
        WINDOWS_HOSTS_PATH
    } else if cfg!(target_os = "macos") {
        MACOS_HOSTS_PATH
    } else {
        panic!("Unsuported operating system!")
    }
}

pub fn reset_host(host_name: &[&str]) -> Result<()> {
    println!("host name: {}", host_name.len());

    let file = File::open(get_hosts_path())?;
    let reader = BufReader::new(file);

    let mut new_lines = Vec::new();
    let mut finded = false;

    for line in reader.lines() {
        let line = line?;

        if !line.starts_with('#') && line.contains(' ') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 0 {
                continue;
            }
            let ip = parts[0];
            let hostname = parts[1];

            println!("[{}] - {}", ip, hostname);
            if !host_name.contains(&hostname) {
                new_lines.push(line);
            } else {
                finded = true
            }
        } else {
            new_lines.push(line);
        }
    }

    if finded {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true) // 清空 hosts 文件
            .open(get_hosts_path())?;

        for line in new_lines {
            writeln!(file, "{}", line).unwrap();
        }
    }

    Ok(())
}

mod find_ip;
mod reset;

use std::{
    io::{stdout, Write},
    process,
};

use dialoguer::{console::Term, theme::ColorfulTheme, Select};
use reset::add_hosts;
use tokio::runtime;
use yansi::{Color, Paint, Style};

use crate::find_ip::{dns_look_up, race_ips};

const RESET_HOSTNAME: &[&str] = &[
    "s3-alpha-sig.figma.com",
    "www.figma.com",
    "static.figma.com",
];

enum DNSServerType {
    Full,
    Fast,
}

const DNS_SERVERS: &[(&str, &str, bool)] = &[
    ("8.8.8.8", "Google DNS", false),
    ("180.76.76.76", "百度 DNS", false),
    ("223.5.5.5", "阿里 DNS", true),
    ("114.114.114.114", "114 DNS", true),
    ("1.1.1.1", "Cloudflare DNS", true),
    ("9.9.9.9", "Quad9 DNS", false),
    ("119.29.29.29", "腾讯 DNS", false),
    ("4.2.2.1", "level3.net", false),
];

const HOST_NAMES: &[(&str, &str)] = &[
    (
        "s3-alpha-sig.figma.com",
        "https://s3-alpha.figma.com/profile/9b3f693e-0677-4743-89ff-822b9f6b72be",
    ),
    (
        "www.figma.com",
        "https://www.figma.com/api/community_categories/all?page_size=10",
    ),
    (
        "static.figma.com",
        "https://static.figma.com/app/icon/1/icon-192.png",
    ),
];

fn main() -> std::io::Result<()> {
    print_banner();

    let items = vec![
        "快速 - 快速测试常用的 DNS 服务商",
        "全面 - 尝试全部 DNS 服务商",
        "重置 - 清除 Hosts 中的 Figma 配置",
    ];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择测试模式(使用键盘方向键选择一个选项，按回车键确认)")
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())?;

    let host_url = match selection {
        Some(0) => Some(get_best_host(DNSServerType::Fast)),
        Some(1) => Some(get_best_host(DNSServerType::Full)),
        Some(2) => {
            println!("User selected item: rest");
            reset::reset_host(&RESET_HOSTNAME)?;
            None
        }
        _ => panic!("invliad model"),
    };

    if let Some(hosts) = host_url {
        choose_add_host(hosts)?;
    };

    Ok(())
}

fn choose_add_host(hosts: Vec<(String, String)>) -> std::io::Result<()> {
    let items = vec!["添加 - 将 hosts 域名映射添加到 hosts 文件中", "退出"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("是否自动添加 hosts (使用键盘方向键选择一个选项，按回车键确认)")
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr())?;

    match selection {
        Some(0) => {
            println!("User selected item: add");
            add_hosts(&hosts[..])?;
        }
        _ => process::exit(0),
    }

    Ok(())
}

fn get_best_host(model: DNSServerType) -> Vec<(String, String)> {
    let dns_list: Vec<_> = match model {
        DNSServerType::Full => DNS_SERVERS.iter().collect(),
        DNSServerType::Fast => DNS_SERVERS.iter().filter(|(_, _, fast)| *fast).collect(),
    };

    let gray_color = Style::new(Color::RGB(105, 105, 105));
    let host_len = HOST_NAMES.len();

    let mut best_ips = vec![];
    let rt = runtime::Runtime::new().expect("异步程序执行异常");

    for (idx, host) in HOST_NAMES.iter().enumerate() {
        let runner_paint = gray_color.paint(format!("[{}/{}]", idx + 1, host_len));

        let (hostname, host_test_url) = host;

        println!("{} find DNS ips: {}", runner_paint, hostname);

        let mut pre_ips = vec![];

        // 根据 dns 找到域名 lookup 到的 ip
        for dns_server in &dns_list {
            let ips = dns_look_up(hostname, dns_server.0).expect("oh host ip query error");
            // 在 main 函数中使用异步任务的结果
            println!("\t {} \t->\t {:?}", dns_server.1, ips);

            pre_ips.extend(ips);
        }

        // 测试链接和那个 ip 的数据通信速度快
        println!("{} race ips: {}", runner_paint, host_test_url);

        let (best_ip, best_ip_desc, best_time) = rt.block_on(async {
            // 等待异步事件
            let result = race_ips(host_test_url, &pre_ips).await;
            result
        });

        // 找到 ip 到测试连接连通速率最佳的
        println!(
            "{} best ip: {} [{}] {}",
            runner_paint,
            Paint::green(best_ip).bold(),
            Paint::green(best_ip_desc),
            Paint::white(format!("{best_time}ms"))
                .bg(Color::Cyan)
                .italic()
        );

        best_ips.push((best_ip.to_string(), hostname.to_string()));
    }

    best_ips
}

// 清除控制台
fn clear_terminal() {
    print!("{}[2J{}[1;1H", 27 as char, 27 as char);
    stdout().flush().unwrap();
}

// 打印说明
fn print_banner() {
    clear_terminal();
    let gray_style = Style::new(Color::RGB(105, 105, 105));

    let title = Paint::green("                  FigmaNetOK             ").bold();
    let name_version = Paint::green("       🐌 Figma 网络最佳线路测试 v2.3.0 🐙    ");
    let site = "                🌕 Moonvy.com      ";
    let github = "    https://github.com/Moonvy/Figma-Net-OK   ";
    let tips = Paint::new("！本工具查找「此时」最佳的 Hosts 配置，具有一定的时效性 \n 当你的网络环境变换或者 Figma 服务器调整，就需要重新测速了\n").italic();

    println!(
        "{}",
        gray_style.paint("----------------------------------------------")
    );
    println!("{}", title);
    println!("{}", name_version);
    println!("{}", site);
    println!("{}", github);
    println!(
        "{}",
        gray_style.paint("----------------------------------------------")
    );
    println!("{}", tips);
}

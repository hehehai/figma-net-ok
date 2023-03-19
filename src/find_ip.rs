use reqwest::{
    dns::{Addrs, Resolve, Resolving},
    ClientBuilder, Error,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::min,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::{self, Instant},
};
use yansi::{Color, Style};

use trust_dns_resolver::{
    config::{NameServerConfigGroup, ResolverConfig},
    system_conf::read_system_conf,
    Resolver,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpInfo {
    pub status: String,
    pub country: String,
    pub country_code: String,
    pub region: String,
    pub region_name: String,
    pub city: String,
    pub zip: String,
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub isp: String,
    pub org: String,
    #[serde(rename = "as")]
    pub as_field: String,
    pub query: String,
}

pub fn dns_look_up(
    hostname: &str,
    dns_server: &str,
) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
    let dns_ip: IpAddr = dns_server.parse::<IpAddr>()?;

    // 创建自定义 DNS 服务器配置
    let resolver_config = ResolverConfig::from_parts(
        None,
        vec![],
        NameServerConfigGroup::from_ips_clear(&[dns_ip], 53, true),
    );
    let (_, resolver_opts) = read_system_conf()?;

    // 创建异步 DNS 解析器
    let resolver = Resolver::new(resolver_config, resolver_opts)?;

    // 执行 DNS 查询
    let response = resolver.lookup_ip(hostname)?;
    let ips: Vec<IpAddr> = response.iter().map(|r| r).collect();

    Ok(ips)
}

pub async fn race_ips(url: &str, ips: &Vec<IpAddr>) -> (IpAddr, String, u64) {
    let gray_color = Style::new(Color::RGB(105, 105, 105));

    let mut fast_time: u64 = 5500;
    let mut point = 0;

    let ip_len = ips.len();

    for (idx, ip) in ips.iter().enumerate() {
        let runner_paint = gray_color.paint(format!("[{}/{}]", idx + 1, ip_len));

        let timeout = min(fast_time + 200, 5500);
        let test_res = test_url(url, ip, timeout).await;
        let test_res = test_res.unwrap_or(99999);

        println!("\t {} \t {} \t->\t {}ms", runner_paint, ip, test_res);

        if fast_time >= test_res {
            fast_time = test_res;
            point = idx;
        }
    }

    let ip_desc = get_ip_info(&ips[point]).await;

    (ips[point], ip_desc, fast_time)
}

#[derive(Debug)]
pub struct TestResolver {
    ip: IpAddr,
}

impl TestResolver {
    pub fn new(ip: IpAddr) -> Self {
        Self { ip }
    }
}

impl Resolve for TestResolver {
    fn resolve(&self, _name: hyper::client::connect::dns::Name) -> Resolving {
        let fixed_ip = self.ip;
        let addrs: Addrs = Box::new(vec![SocketAddr::new(fixed_ip, 8080)].into_iter());
        Box::pin(async move { Ok(addrs) })
    }
}

async fn test_url(url: &str, ip: &IpAddr, timeout: u64) -> Result<u64, Error> {
    let start = Instant::now();

    let client = ClientBuilder::new()
        .use_rustls_tls()
        .dns_resolver(Arc::new(TestResolver::new(*ip)))
        .build()?;

    client
        .get(url)
        .timeout(time::Duration::from_millis(timeout))
        .send()
        .await?;

    let end = Instant::now();
    let elapsed = end.duration_since(start);

    Ok(elapsed.as_millis() as u64)
}

async fn get_ip_info(ip: &IpAddr) -> String {
    let unknown_area = "未知地区";
    let url = format!("http://ip-api.com/json/{}?lang=zh-CN", *ip);

    let res = reqwest::get(url).await;

    let res = match res {
        Ok(v) => match v.json::<IpInfo>().await {
            Ok(data) => format!("{},{},{}", data.country_code, data.country, data.city),
            Err(_) => unknown_area.to_string(),
        },
        Err(_) => unknown_area.to_string(),
    };

    res
}

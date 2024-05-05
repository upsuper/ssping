use anyhow::{bail, Context as _, Error};
use clap::Parser as _;
use hyper::body::Bytes;
use hyper::client::conn::http1;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use shadowsocks::config::ServerType;
use shadowsocks::context::SharedContext;
use shadowsocks::relay::Address;
use shadowsocks::{ProxyClientStream, ServerConfig};
use std::net::{IpAddr, SocketAddr};
use std::process::ExitCode;
use std::time::{Duration, Instant};
use url::{Host, Origin, Url};

#[derive(clap::Parser, Debug)]
#[command(about = "Command-line tool for testing connectivity of Shadowsocks server.")]
struct Opts {
    /// Shadowsocks URL of the proxy to probe.
    ss_url: String,
    /// URL used to test through the Shadowsocks proxy.
    ///
    /// By default, http://www.google.com/generate_204 is used.
    /// Any URL that normally returns a successful HTTP status code (200~299) should work.
    #[arg(long, short)]
    url: Option<String>,
    /// Stop after n probes. By default, no limit will be applied.
    #[arg(long, short)]
    count: Option<u64>,
    /// Interval in seconds between each probe.
    #[arg(long, short, default_value_t = 1f32)]
    interval: f32,
}

#[tokio::main]
async fn main() -> ExitCode {
    let opts = Opts::parse();
    let mut counts = Counts::default();
    let start = Instant::now();
    let result = {
        let main_future = ping_loop(&opts, &mut counts);
        let signal_future = tokio::signal::ctrl_c();
        tokio::pin!(main_future);
        tokio::pin!(signal_future);
        tokio::select! {
            main_res = main_future => main_res,
            signal_res = signal_future => signal_res.context("Failed to listen for event"),
        }
    };
    let duration = start.elapsed();
    if counts.total > 0 {
        eprintln!("--- ping statistics ---");
        eprintln!(
            "{} attempted, {} succeeded, {} errors, time {:.0}ms",
            counts.total,
            counts.success,
            counts.error,
            duration.as_secs_f32() * 1000.,
        );
    }
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        ExitCode::from(2)
    } else if counts.success > 0 {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

#[derive(Default)]
struct Counts {
    total: u64,
    success: u64,
    error: u64,
}

async fn ping_loop(opts: &Opts, counts: &mut Counts) -> Result<(), Error> {
    let config = ServerConfig::from_url(&opts.ss_url).context("Failed to parse Shadowsocks URL")?;
    let interval = Duration::from_millis((opts.interval * 1000.) as u64);
    let context = shadowsocks::context::Context::new_shared(ServerType::Local);
    let url = opts
        .url
        .as_deref()
        .unwrap_or("http://www.google.com/generate_204");
    let url = Url::parse(url).context("Invalid test URL")?;
    let addr = match url.origin() {
        Origin::Opaque(_) => unreachable!("Unexpected opaque origin"),
        Origin::Tuple(scheme, host, port) => {
            if scheme != "http" {
                bail!("Unsupported scheme. Only HTTP is supported.");
            }
            match host {
                Host::Domain(domain) => Address::DomainNameAddress(domain, port),
                Host::Ipv4(ip) => Address::SocketAddress(SocketAddr::new(IpAddr::V4(ip), port)),
                Host::Ipv6(ip) => Address::SocketAddress(SocketAddr::new(IpAddr::V6(ip), port)),
            }
        }
    };

    let target_host = addr.host();
    let proxy_host = config.addr().host();
    eprintln!("PING {target_host} via {proxy_host}.");

    while opts.count.map(|c| counts.total < c).unwrap_or(true) {
        if counts.total > 0 {
            tokio::time::sleep(interval).await;
        }
        let start = Instant::now();
        let result = send_request(context.clone(), &config, &addr, &url).await;
        let duration = start.elapsed();
        counts.total += 1;
        match result {
            Ok(status) => {
                eprintln!(
                    "Status {} from {target_host} via {proxy_host}: time={:.1}ms",
                    status.as_u16(),
                    duration.as_secs_f32() * 1000.
                );
                if status.is_success() {
                    counts.success += 1;
                } else {
                    counts.error += 1;
                }
            }
            Err(e) => {
                eprintln!("From {target_host} via {proxy_host}: {e}");
                counts.error += 1;
            }
        }
    }
    Ok(())
}

async fn send_request(
    context: SharedContext,
    config: &ServerConfig,
    addr: &Address,
    url: &Url,
) -> Result<StatusCode, &'static str> {
    let proxy = ProxyClientStream::connect(context, config, addr)
        .await
        .map_err(|_| "Failed to connect to Shadowsocks")?;
    let (mut request_sender, connection) = http1::handshake(TokioIo::new(proxy))
        .await
        .map_err(|_| "Failed to start HTTP connection")?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let mut request = hyper::Request::get(url.path())
        .header(
            "User-Agent",
            concat!(env!("CARGO_BIN_NAME"), "/", env!("CARGO_PKG_VERSION")),
        )
        .header("Accept", "*/*");
    if let Address::DomainNameAddress(domain, _) = addr {
        request = request.header("Host", domain);
    }
    let request = request
        .body(http_body_util::Empty::<Bytes>::new())
        .expect("Invalid request");
    let response = request_sender
        .send_request(request)
        .await
        .map_err(|_| "Failed to send HTTP request")?;
    Ok(response.status())
}

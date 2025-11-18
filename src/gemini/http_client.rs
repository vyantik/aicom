use reqwest::{Client, Proxy};
use std::process::Command;
use std::time::Duration;

fn detect_system_proxy() -> Option<String> {
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.gnome.system.proxy", "mode"])
        .output()
    {
        if let Ok(mode) = String::from_utf8(output.stdout) {
            let mode = mode.trim().trim_matches('\'');
            if mode == "manual" {
                if let Ok(https_host) = Command::new("gsettings")
                    .args(["get", "org.gnome.system.proxy.https", "host"])
                    .output()
                {
                    if let Ok(host) = String::from_utf8(https_host.stdout) {
                        let host = host.trim().trim_matches('\'');
                        if !host.is_empty() {
                            if let Ok(port) = Command::new("gsettings")
                                .args(["get", "org.gnome.system.proxy.https", "port"])
                                .output()
                            {
                                if let Ok(port_str) = String::from_utf8(port.stdout) {
                                    let port = port_str.trim();
                                    if port != "0" && !port.is_empty() {
                                        return Some(format!("http://{}:{}", host, port));
                                    }
                                }
                            }
                        }
                    }
                }

                if let Ok(http_host) = Command::new("gsettings")
                    .args(["get", "org.gnome.system.proxy.http", "host"])
                    .output()
                {
                    if let Ok(host) = String::from_utf8(http_host.stdout) {
                        let host = host.trim().trim_matches('\'');
                        if !host.is_empty() {
                            if let Ok(port) = Command::new("gsettings")
                                .args(["get", "org.gnome.system.proxy.http", "port"])
                                .output()
                            {
                                if let Ok(port_str) = String::from_utf8(port.stdout) {
                                    let port = port_str.trim();
                                    if port != "0" && !port.is_empty() {
                                        return Some(format!("http://{}:{}", host, port));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let v2ray_socks_ports = ["10808", "10809", "1080"];
    let v2ray_http_ports = ["8080", "7890"];

    let listening_ports = if let Ok(output) = Command::new("ss").args(["-tln"]).output() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else if let Ok(output) = Command::new("netstat").args(["-tln"]).output() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::new()
    };

    for port in &v2ray_http_ports {
        if listening_ports.contains(&format!("127.0.0.1:{}", port))
            || listening_ports.contains(&format!("localhost:{}", port))
            || listening_ports.contains(&format!("0.0.0.0:{}", port))
            || listening_ports.contains(&format!(":::{}", port))
        {
            return Some(format!("http://127.0.0.1:{}", port));
        }
    }

    for port in &v2ray_socks_ports {
        if listening_ports.contains(&format!("127.0.0.1:{}", port))
            || listening_ports.contains(&format!("localhost:{}", port))
            || listening_ports.contains(&format!("0.0.0.0:{}", port))
            || listening_ports.contains(&format!(":::{}", port))
        {
            return Some(format!("socks5://127.0.0.1:{}", port));
        }
    }

    None
}

fn apply_proxy(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    let mut builder = builder;

    if let Some(proxy_url) = detect_system_proxy() {
        let proxy_result = if proxy_url.starts_with("socks5://") {
            Proxy::all(&proxy_url)
        } else if proxy_url.starts_with("http://") || proxy_url.starts_with("https://") {
            Proxy::all(&proxy_url)
        } else {
            Proxy::all(&format!("http://{}", proxy_url))
        };

        if let Ok(proxy) = proxy_result {
            builder = builder.proxy(proxy);
            eprintln!("🔗 Используется системный прокси: {}", proxy_url);
            return builder;
        }
    }

    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY") {
        if let Ok(proxy) = Proxy::https(&proxy_url) {
            builder = builder.proxy(proxy);
            eprintln!("🔗 Используется HTTPS прокси: {}", proxy_url);
            return builder;
        }
    } else if let Ok(proxy_url) = std::env::var("ALL_PROXY") {
        if let Ok(proxy) = Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
            eprintln!("🔗 Используется прокси (ALL_PROXY): {}", proxy_url);
            return builder;
        }
    } else if let Ok(proxy_url) = std::env::var("HTTP_PROXY") {
        if let Ok(proxy) = Proxy::http(&proxy_url) {
            builder = builder.proxy(proxy);
            eprintln!("🔗 Используется HTTP прокси: {}", proxy_url);
            return builder;
        }
    }

    eprintln!("ℹ️  Прокси не обнаружен, запросы идут напрямую");
    builder
}

pub fn create_client_with_proxy() -> Result<Client, reqwest::Error> {
    let builder = Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(300));

    let builder = apply_proxy(builder);
    builder.build()
}

pub fn create_client_with_long_timeout() -> Result<Client, reqwest::Error> {
    let builder = Client::builder()
        .connect_timeout(Duration::from_secs(120))
        .timeout(Duration::from_secs(300));

    let builder = apply_proxy(builder);
    builder.build()
}

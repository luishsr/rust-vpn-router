use clap::{Arg, Command};
use reqwest::Client;
use std::fs::{self};
use std::process::Command as ShellCommand;
use std::net::Ipv4Addr;
use std::process::Command as ProcessCommand;
use tokio::runtime::Runtime;

#[tokio::main]
async fn main() {
    let matches = Command::new("VPN Router")
        .version("1.1")
        .author("Your Name <youremail@example.com>")
        .about("Configures VPN split tunneling for specific domains")
        .arg(
            Arg::new("vpn-config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets the OpenVPN config file")
                .required(true),
        )
        .arg(
            Arg::new("add-domains")
                .short('a')
                .long("add")
                .value_name("DOMAINS")
                .help("Comma-separated list of domains to route through the VPN"),
        )
        .arg(
            Arg::new("remove-domains")
                .short('r')
                .long("remove")
                .value_name("DOMAINS")
                .help("Comma-separated list of domains to remove from VPN routing"),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .help("List the currently routed domains")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let vpn_config = matches.get_one::<String>("vpn-config").unwrap();

    if matches.get_flag("list") {
        match list_routed_domains(vpn_config) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to list routed domains: {}", e),
        }
    } else if let Some(domains) = matches.get_one::<String>("add-domains") {
        let domain_list: Vec<&str> = domains.split(',').collect();
        match setup_vpn_routing(vpn_config, domain_list).await {
            Ok(_) => println!("VPN routing setup successfully!"),
            Err(e) => eprintln!("Failed to set up VPN routing: {}", e),
        }
    } else if let Some(domains) = matches.get_one::<String>("remove-domains") {
        let domain_list: Vec<&str> = domains.split(',').collect();
        match remove_vpn_routing(vpn_config, domain_list) {
            Ok(_) => println!("VPN routing removed successfully!"),
            Err(e) => eprintln!("Failed to remove VPN routing: {}", e),
        }
    }
}

async fn setup_vpn_routing(vpn_config: &str, domains: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut ip_addresses = Vec::new();

    for domain in &domains {
        let ips = resolve_domain_ips(&client, domain).await?;
        ip_addresses.extend(ips);
    }

    configure_vpn(vpn_config, ip_addresses)?;

    Ok(())
}

async fn resolve_domain_ips(client: &Client, domain: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let response = client.get(&format!("https://dns.google/resolve?name={}", domain)).send().await?;
    let json: serde_json::Value = response.json().await?;

    let ips = json["Answer"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|entry| entry["data"].as_str())
        .filter(|ip| ip.parse::<Ipv4Addr>().is_ok())
        .map(String::from)
        .collect::<Vec<_>>();

    Ok(ips)
}

fn configure_vpn(vpn_config: &str, ip_addresses: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_content = fs::read_to_string(vpn_config)?;

    // Disable default route pull if not already done
    if !config_content.contains("route-nopull") {
        config_content.push_str("route-nopull\n");
    }

    // Add routes for each IP address
    for ip in ip_addresses {
        let route_entry = format!("route {} 255.255.255.255 net_gateway\n", ip);
        if !config_content.contains(&route_entry) {
            add_route_to_interface(&ip, "tun0")?;  // Assuming tun0 is the interface
        }
    }

    // Restart OpenVPN with the new configuration
    ShellCommand::new("sudo")
        .arg("openvpn")
        .arg("--config")
        .arg(vpn_config)
        .spawn()?;

    Ok(())
}

fn add_route_to_interface(ip: &str, interface: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Add route via ip command
    let status = ProcessCommand::new("sudo")
        .arg("ip")
        .arg("route")
        .arg("add")
        .arg(format!("{}/32", ip))
        .arg("dev")
        .arg(interface)
        .status()?;

    if !status.success() {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to add route for IP {}", ip),
        )))
    } else {
        Ok(())
    }
}

fn list_routed_domains(vpn_config: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(vpn_config)?;
    let routes: Vec<&str> = config_content
        .lines()
        .filter(|line| line.starts_with("route"))
        .collect();

    println!("Currently routed domains/IPs:");
    for route in routes {
        println!("{}", route);
    }

    Ok(())
}

fn remove_vpn_routing(vpn_config: &str, domains: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut ips_to_remove = Vec::new();

    // Resolve the domains to IPs
    for domain in &domains {
        let ips = Runtime::new().unwrap().block_on(resolve_domain_ips(&client, domain))?;
        ips_to_remove.extend(ips);
    }

    let config_content = fs::read_to_string(vpn_config)?;
    let mut new_content = String::new();

    for line in config_content.lines() {
        let is_route = line.starts_with("route");
        let should_remove = ips_to_remove.iter().any(|ip| line.contains(ip));

        if is_route && should_remove {
            println!("Removing route: {}", line);
            continue; // Skip adding this line to new content
        }

        new_content.push_str(line);
        new_content.push('\n');
    }

    fs::write(vpn_config, new_content)?;

    Ok(())
}

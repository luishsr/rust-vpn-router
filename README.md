# VPN Router

VPN Router is a Rust-based command-line application that allows you to easily manage VPN split tunneling for specific domains. With this tool, you can route traffic for selected domains through your VPN connection while keeping other traffic, including SSH, outside the VPN. The tool also allows you to list currently routed domains and remove them as needed.

## Features

- **Split Tunneling:** Route only traffic to specified domains through the VPN.
- **Domain Management:** Add, list, and remove routed domains dynamically.
- **Easy Configuration:** Works with your existing OpenVPN configuration file.

## Installation

### Prerequisites

- **Rust**: You'll need Rust and Cargo installed on your machine. You can install Rust using `rustup`:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  
- **OpenVPN**: Make sure you have OpenVPN installed and configured on your system.

### Build from Source

- Clone the repository:

   ```bash
   git clone https://github.com/luishsr/rust-vpn-router.git
   cd rust-vpn-router

- Build the project:

    ```bash
    cargo build --release

The compiled binary will be available in the target/release directory.

### Usage

- Basic Usage
  ```bash
  vpn-router --config /path/to/your-config.ovpn --add solana.com,api.solana.com

- Options
-c, --config <FILE>: Specifies the OpenVPN configuration file. (Required)
-a, --add <DOMAINS>: Comma-separated list of domains to route through the VPN.
-r, --remove <DOMAINS>: Comma-separated list of domains to remove from VPN routing.
-l, --list: List the currently routed domains.

### Examples
- Route traffic for specific domains:

  ```bash
  vpn-router --config /etc/openvpn/client.ovpn --add solana.com,api.solana.com

- List currently routed domains:

  ```bash
  vpn-router --config /etc/openvpn/client.ovpn --list

- Remove domains from VPN routing:

  ```bash
   vpn-router --config /etc/openvpn/client.ovpn --remove solana.com

### How It Works
VPN Router modifies your OpenVPN configuration file to enable split tunneling. It dynamically adds or removes routes for the specified domains by resolving their IP addresses and updating the OpenVPN config file.

- Add Domains: The tool resolves the IP addresses of the provided domains and appends route entries to the OpenVPN configuration.
- Remove Domains: The tool removes route entries corresponding to the provided domains from the OpenVPN configuration.
- List Domains: The tool lists all current route entries in the OpenVPN configuration.

### Contributing
Contributions are welcome! Please feel free to submit a pull request or open an issue.

### License
This project is licensed under the MIT License. See the LICENSE file for details.

### Acknowledgments
Inspired by the need to manage VPN routing more effectively on cloud servers.
Thanks to the Rust community for their excellent tooling and libraries.

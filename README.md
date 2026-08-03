<p align="center">
  <img src="./docs/assets/logo.png" alt="Logo" width="100">
</p>

<p align="center">
    <img alt="Built with Rust" src="https://img.shields.io/badge/Built%20with-Rust-red?logo=rust&logoColor=white"/>
    <a href="https://ko-fi.com/1TheCrazy">
        <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support on Ko-fi"/>
    </a>
    <img alt="MIT license" src="https://img.shields.io/badge/licence-MIT-blue.svg"/>
</p>

# Tunnel
Tunnel is a lightweight WireGuard-based private VPN platform for managing and connecting to a network of distributed proxy nodes.

- [Features](#features)
- [Installation Guide](#installation-guide)
- [CLI Guide](#cli)
- [How it works](#how-it-works)
- [Troubleshooting](#troubleshooting)
- [Work In Progress](#wip)
- [Contributing](#contributing)

## Features
> Tunnel is a lightweight private VPN platform for managing and connecting to a network of distributed proxy nodes.

What now?<br>
If this doesn't mean anything to you, you can imagine Tunnel as a framework for managing **your private VPN network**.<br>
You can now easily set up your own nodes, connect to your home network via Tunnel, or use your VPS or dedicated server as a VPN proxy.<br>
If you encoded this software and its description as a list of features, it might look like this:

- Add your own VPN nodes
- Manage your nodes via a centralized server
- Easily connect to a VPN proxy node via the CLI or GUI
- DynDNS handling

> [!WARNING]
> iOS is not supported as a Node/Client environment. A Server could technically run on iOS.

> [!WARNING]
> It's recommended that you use a separate device (e.g. a small Raspberry Pi) for each node/client.
> A Node may not run on the same device as a Client concurrently.

## Installation Guide
Tunnel is designed to be easily installed. However, you'll still have to do some things manually if you want to use Tunnel.

### Prerequisites
You won't need any software prerequisites, but this section covers everything that the installer does not handle for you.

#### Networking and Firewall
In order for Tunnel to work, you'll have to open a special port on your server so that clients can make requests to the software and nodes can register themselves with the network.

The following ports have to be open:

| Port | Protocol |Required on Server | Required on Node|
|---|---|---|---|
|VPN Port|UDP|No|Yes|
|42069|TCP|Yes|Yes|

The VPN Port is customizable but defaults to `51820`.<br>
The port `42069` is used by the Server and the Node to communicate and therefore has to be open on both the Node and the Server.<br>
As you can see, the Node requires both the Tunnel Service Port and the VPN Port to be open for incoming requests.<br>
Mind the protocols for which the ports have to be opened.

It will not cover *how* you would open these ports, as this could vary widely from deployment to deployment. <br>
For example, you might have to configure `ufw` or `iptables`; configure your router if you plan to use your home network as a Node; or configure your VPS' firewall through your cloud provider.


#### Configuration
Using configuration files (`node.toml` and `server.toml`) allows you to configure settings, including security settings.<br>
You need to at least configure the basics of the server and node in order for your Tunnel network to work.<br>

A Node can have the following configuration in `node.toml`:

|Name|What is this?| Default |
|---|---|---|
|**server_host**|The host of the server managing this Tunnel network|localhost:42069|
|**password**|The password of the server|None (no authentication needed)|
|**vpn_port**|The port on which the VPN traffic between Node and Client should happen|51820|
|**update_period**|The period on which the node should report back to the server|10min|

A typical Node configuration therefore might look like this:
```
server_host = "123.123.123.123:42069" # or yourdomain.com:42069
                                      # explicitly include port 42069
password = "supersecretpassword"
vpn_port = 12345
update_period = "24h 10min"
```

A Server can have the following configuration in `server.toml`:

|Name|What is this?| Default |
|---|---|---|
|**password**|The password of the server|None (no authentication needed)|

A typical Server configuration therefore might look like this:
```
password = "supersecretpassword" # identical between node and server
```

### Installing
> [!IMPORTANT]
> Since the software (Node/Server/Client) runs as binaries, the most common architectures are provided in GitHub releases.
> These are: `x86_64`, `aarch64` (ARM 64-bit), and `armv7` (ARM 32-bit).
> If you require more exotic architectures, you may need to compile them yourself by cloning this repo and compiling them locally.

> [!NOTE]
> To install the installer-based client GUI, refer to the GitHub Releases page, where you'll find the installer for your system.

The installation of a Node/Server/CLI boils down to the following steps:

1. Choose a target directory, e.g. `/home/user/tunnel` or `C:\Users\User\Desktop\Tunnel`
2. Create a configuration file (if installing Node or Server), e.g. `node.toml` or `server.toml`
3. Run the installer script:
```
curl -fsSL https://raw.githubusercontent.com/1TheCrazy/Tunnel/main/install/install.sh | sudo sh -s -- --node # or --server or --cli
```
Or on Windows (make sure you run this shell as an administrator):
```
& ([scriptblock]::Create((Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/1TheCrazy/Tunnel/main/install/install.ps1').Content)) --node # or --server or --cli
```
4. Run the downloaded file from a shell: `./node`, `./server` / `./node.exe`, or `./server.exe`<br>
To run the CLI, open a new shell and type `tunnel --help` to get started.

## CLI
> [!NOTE]
> Tunnel also provides a GUI-based client.
> See [the Installation Guide](#installing) for information.

To see information about the CLI and each subcommand, use `tunnel <command> --help`.

The flow for using the CLI is:

1. Add a server: `tunnel server add <NAME> <HOST> -p <PASSWORD>`
2. Set a server as active: `tunnel server set <NAME>`
3. List the nodes available on the active server: `tunnel list-nodes`
4. Connect to a node: `tunnel connect <ID>`
5. Disconnect from the node: `tunnel disconnect`

A typical chain of commands may therefore look like this:
```
tunnel server add "Creative Name" "vpn.mydomain.com:42069" # no --password: no authentication required
tunnel server set "Creative Name"
tunnel -r list-nodes
tunnel -r connect PcSlIATNqUyiPUhwf_LoXQ # ID obtained from `tunnel list-nodes`
```

It's recommended to use the `-r` (`--refresh`) flag with every `tunnel connect`, since node IPs may have been updated.

The GUI provides the same functionality as the CLI, wrapped in a clean and modern *Tauri*-based application.

## How it works

Below is a visual diagram explaining the abstract flow of the framework:
![Framework Flow](docs/assets/image.png)

This architecture allows for an easily expandable node network.<br>
This, however, comes at the cost that the Node has to host a server itself so it can receive messages from the server when a client wants to establish a new connection.

Therefore, the server is nothing but a simple HTTP server handling Client-Node communication and serving as a Node registry.<br>
The server exposes appropriate endpoints to the client.

The Node hosts an HTTP server itself so that it may receive an HTTP POST request when a client wants to establish a *fresh* connection. It can then assign the client an IP address inside the VPN network and register its key.

For the VPN part, the client and node just wrap the WireGuard CLI.<br>
This means that the VPN connection is securely managed by WireGuard, while Tunnel mainly acts as the communication and registration framework.<br>

## Troubleshooting
If you run into any errors that actually seem like bugs while using Tunnel, it's recommended to clear any saved data and configurations and reset WireGuard.<br>
First, stop the process gracefully (node/server/`tunnel disconnect`) so that the active WireGuard service is stopped.<br>

Then execute the following commands:

***Windows***
```
# Delete every installed WireGuard service
Get-Service 'WireGuardTunnel$*' |
    Select-Object -ExpandProperty Name |
    ForEach-Object {
        sc.exe delete "$_"
    }

# Remove Tunnel state
Remove-Item "C:\Users\USER\AppData\Roaming\1thecrazy\tunnel\save" -Recurse -Force
Remove-Item "C:\Users\USER\AppData\Roaming\1thecrazy\tunnel\wg" -Recurse -Force
```
***Linux***
```
# Delete every WireGuard service
sudo rm -rf /etc/wireguard/*

# Remove Tunnel state
sudo rm -rf /home/USER/.config/1thecrazy/tunnel/save
sudo rm -rf /home/USER/.config/1thecrazy/tunnel/
```

Then restart the application (node/server/client).<br>
This usually fixes WireGuard issues.<br>
Also, keep an eye on the logs of each service; they usually contain a clue about what went wrong.

Most of the time, errors will stem from invalid command usage, an invalid configuration, or missing network configuration (ports, firewall, ...).


## W.I.P
This project is in a state that is best described with "stable beta".<br>
The main functionality is done, and it's not only usable but also enjoyable; however, not every desired feature is implemented yet.<br>

Things that are not implemented yet, but probably will be in the future, are:

- Security (HTTPS, SSL)
- IPv6

Just keep in mind that this project is not fully finished if you find yourself thinking, "Why isn't this implemented yet?".

## Contributing
This project is open-source, so any contributions are welcome!<br>
You can create a [pull request](https://github.com/1TheCrazy/Tunnel/pulls) at any time.

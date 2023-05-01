# quicssh-rs

> :smile: **quicssh-rs** is a QUIC proxy that allows to use QUIC to connect to an SSH server without needing to patch the client or the server. 

`quicssh-rs` is [quicssh](https://github.com/moul/quicssh) rust implementation. It is based on [quinn](https://github.com/quinn-rs/quinn) and [tokio](https://github.com/tokio-rs/tokio)

Why use QUIC? Because SSH is vulnerable in TCP connection environments, and most SSH packets are actually small, so it is only necessary to maintain the SSH connection to use it in any network environment. QUIC is a good choice because it has good weak network optimization and an important feature called connection migration. This means that I can switch Wi-Fi networks freely when remote, ensuring a stable SSH connection.

## Demo
https://user-images.githubusercontent.com/39181969/235409750-234de94a-1189-4288-93c2-45f62a9dfc48.mp4

## Why not mosh?
Because the architecture of mosh requires the opening of many ports to support control and data connections, which is not very user-friendly in many environments. In addition, vscode remote development does not support mosh.

## Architecture

Standard SSH connection

```
┌───────────────────────────────────────┐             ┌───────────────────────┐
│                  bob                  │             │         wopr          │
│ ┌───────────────────────────────────┐ │             │ ┌───────────────────┐ │
│ │           ssh user@wopr           │─┼────tcp──────┼▶│       sshd        │ │
│ └───────────────────────────────────┘ │             │ └───────────────────┘ │
└───────────────────────────────────────┘             └───────────────────────┘
```



---

SSH Connection proxified with QUIC

```
┌───────────────────────────────────────┐             ┌───────────────────────┐
│                  bob                  │             │         wopr          │
│ ┌───────────────────────────────────┐ │             │ ┌───────────────────┐ │
│ │ssh -o ProxyCommand "quicssh-rs    │ │             │ │       sshd        │ │
│ │ client quic://%h:4433             │ │             │ └───────────────────┘ │
│ │       user@wopr                   │ │             │           ▲           │
│ └───────────────────────────────────┘ │             │           │           │
│                   │                   │             │           │           │
│                process                │             │  tcp to localhost:22  │
│                   │                   │             │           │           │
│                   ▼                   │             │           │           │
│ ┌───────────────────────────────────┐ │             │┌─────────────────────┐│
│ │  quicssh-rs client wopr:4433      │─┼─quic (udp)──▶│   quicssh-rs server ││
│ └───────────────────────────────────┘ │             │└─────────────────────┘│
└───────────────────────────────────────┘             └───────────────────────┘
```

## Usage

```console
$ quicssh-rs -h
A simple ssh server based on quic protocol

Usage: quicssh-rs [OPTIONS] <COMMAND>

Commands:
  server  Server
  client  Client
  help    Print this message or the help of the given subcommand(s)

Options:
      --log <LOG_FILE>  Location of log, Defalt if
  -h, --help            Print help
  -V, --version         Print version
   ```

### Client

```console
$ quicssh-rs client -h
Client

Usage: quicssh-rs client <URL>

Arguments:
  <URL>  Sewrver address

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Server

```console
$ quicssh-rs server -h
Server

Usage: quicssh-rs server [OPTIONS]

Options:
  -l, --listen <LISTEN>  Address to listen on [default: 0.0.0.0:4433]
  -h, --help             Print help
  -V, --version          Print version
```

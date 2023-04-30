# quicssh-rs

> :smile: **quicssh-rs** is a QUIC proxy that allows to use QUIC to connect to an SSH server without needing to patch the client or the server. 

`quicssh-rs` is [quicssh](https://github.com/moul/quicssh) rust implementation. It is based on [quinn](https://github.com/quinn-rs/quinn) and [tokio](https://github.com/tokio-rs/tokio)

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
│ │  quicssh-rs client wopr:4433      │─┼─quic (udp)──▶│   quicssh-rs server    ││
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
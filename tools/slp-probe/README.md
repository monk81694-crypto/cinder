# slp-probe

Minimal Minecraft Java Server List Ping probe written in Rust with std only and zero dependencies.

## What it does

Connects to a Minecraft Java server with TCP, sends a handshake packet with protocol version 767 and next state 1, sends a status request, reads the varint prefixed JSON response, extracts players online, max players, version name and MOTD, then sends a ping payload and measures the pong round trip.

Output is a single JSON line on stdout for easy consumption by the Node Electron launcher.

## Build

```
cargo build --release
```

The binary will be at target/release/slp-probe.exe on Windows and target/release/slp-probe on Linux and macOS.

## Install

Windows copy to launcher bin folder, run from tools/slp-probe:

```
copy target\release\slp-probe.exe ..\..\bin\
```

Linux and macOS:

```
cp target/release/slp-probe ../../bin/
```

## Protocol

Usage:

```
slp-probe <host> [port=25565] [timeout_ms=5000]
```

Rules:

- Print exactly one JSON line to stdout
- Exit 0 on success or failure, exit 2 on usage error
- Success:

```
{"ok":true,"players":123,"max":500,"motd":"A server","pingMs":42,"version":"1.21.4"}
```

- Failure:

```
{"ok":false,"error":"connection refused"}
```

- Usage error:

```
{"ok":false,"error":"usage"}
```

Example:

```
slp-probe play.example.com 25565 5000
slp-probe 127.0.0.1
slp-probe localhost 25565 2000
```

Details:

- Handshake uses protocol version 767, server address from host arg, port from port arg, next state 1
- Status request follows, response is a varint length prefixed packet with JSON
- JSON parsing extracts players online and max, version name, description as string or object with text plus extra array texts
- MOTD handling: flatten description text plus extra array texts, strip section sign formatting codes, truncate to 120 chars, JSON escape
- Ping: send u64 timestamp in ms, measure pong, if ping fails then pingMs is -1 but result is still ok true when status succeeded
- Timeouts use TcpStream connect timeout plus set_read_timeout, total runtime never exceeds timeout_ms plus 2s

## Fallback

The launcher falls back to a JS implementation if the binary is absent. The binary is optional and only used as a fast path. If tools/slp-probe output is missing or fails to run, the launcher uses the built in JS prober with the same JSON protocol.

## Notes

- Edition 2021, no external crates
- MOTD is truncated to 120 chars
- pingMs is round trip in ms or -1 when ping fails


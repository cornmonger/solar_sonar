# CHANGES: solar_sonar

## 0.4.0 (draft)
- Support for different kinds of chat logs:
  - Alliance, Corp, Fleet, Local, and Group
- Support for Game logs
  - Combat parsing
  - Location parsing based on jump notifications
- Identifies chat messages directed at your character's name
- Identifies danger call-outs in chat such as "neut"
- Custom modes:
  - Audible pings behave differently based on: Mode, Log, and Character.
## 0.3.0
- Networking support added:
  - Configuration:
    - Server: `~/.config/solar_sonar/server.toml`
    - Client: `~/.config/solar_sonar/client.toml`
  - TLS certificates generated to: `~/.config/solar_sonar/certs`
  - Host a server with `--serve <server profile>`
  - Connect to a server with `--connect <connect profile>`
  - Tips:
    - LAN: Use your LAN ip
    - Internet: Use the external IP
    - Copy the `authority_<server profile>.pem` to your client's certs dir.
## 0.2.0
- New command-line features:
  - Perform `--replay` of a specific log file
  - Disable output of `--stdio` and `--audio`
- Basic library support via `solar_sonar::start`
## 0.1.1
First release.

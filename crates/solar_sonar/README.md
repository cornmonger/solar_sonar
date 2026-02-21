# solar_sonar
[![Crate Badge]][Crate] [![Docs Badge]][Docs] [![Changes Badge]][Changes] [![License Badge]][License] 

*StarMap alerts for EVE Online*

**solar_sonar** is a small command-line tool that monitors intel chat channels
in [EVE Online](https://www.eveonline.com) and sounds a text-to-speech alert
when systems within range are mentioned.


## Usage

### Installation via Cargo (Rust)

#### 1. Install Rust

Instructions: [rustup.rs](https://rustup.rs)

- Windows: Download the installer at [rustup.rs](https://rustup.rs)
- Linux / MacOS:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

Take note of the post-install instructions regarding Cargo and your
environment `$PATH`.

Restart your terminal if necessary.

#### 2. Install solar_sonar via Cargo (Rust)

```bash
cargo install solar_sonar
```

### Configuration

Within EVE:
- Enable logging: Settings / Chat / Log Chat to File
- Enable timestamps: Right-click on channel / Show Timestamp

`~/.config/solar_sonar`
- `characters.toml` Characters and their channels
- `settings.toml` Game path and various options

The first run of *solar_sonar* will install configuration files and
let you edit them (if `$EDITOR` is set in your environment). It will also
download the voice model assets (80mb) for text-to-speech.


### Command Line Interface

Refer to `solar_sonar --help`.

Example: *Watch system **UALX-3** for character aliased **corn** within **two** jumps*:  
```bash
solar_sonar corn UALX-3
```

On startup, the text-to-speech system will play a random phrase to verify
that alerts are sounding off. If you don't hear it, something is wrong.


## Project
If you run into problems with the app or instructions:
1. Search [GitHub Discussions](https://github.com/cornmonger/solar_sonar/discussions)
and ask about it if you don't find anything
2. Search [GitHub Issues](https://github.com/cornmonger/solar_sonar/issues) and
report it if you don't find anything

Feedback is welcome.

Contributions are usually welcome. Ask first in a **issue** before making a
pull request and we'll discuss it.


## License (AGPL3)
solar_sonar: StarMap alerts for EVE Online  
Copyright (C) YC128 [cornmonger](https://cornmonger.fish)  

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a [copy](./LICENSE-AGPL-3.txt) of the
GNU Affero General Public License along with this program.
If not, see https://www.gnu.org/licenses/.


## Trademarks
[EVE Online](https://eveonline.com) is a trademark of
[CCP Games](https://www.ccpgames.com).


[Crate]: https://crates.io/crates/solar_sonar
[Crate Badge]: https://img.shields.io/crates/v/solar_sonar.svg
[Docs]: https://docs.rs/solar_sonar
[Docs Badge]: https://img.shields.io/badge/docs-blue
[Changes]: ../../docs/CHANGES.md
[Changes Badge]: https://img.shields.io/badge/changes-blue
[License]: #License-AGPL3
[License Badge]: https://img.shields.io/badge/license-AGPL3-blue.svg

# Podzol

[![Crates.io](https://img.shields.io/crates/v/podzol)](https://crates.io/crates/podzol)
[![AUR version](https://img.shields.io/aur/version/podzol)](https://aur.archlinux.org/packages/podzol)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Podzol is a modern package manager for Minecraft modpacks that simplifies
creation and management through a clean TOML-based manifest format and direct
integration with the Modrinth API

[Website](https://podzol.morrone.dev) •
[Documentation](https://podzol.morrone.dev/docs) •
[Crates.io](https://crates.io/crates/podzol)

## Features

- Simple TOML-based manifest format
- Direct integration with Modrinth API
- Automatic version management
- Client/server-side awareness
- Support for multiple mod loaders (Fabric, Forge, Quilt, NeoForge)
- Built-in resource pack and shader management
- File override system for custom configurations

## Installation

### From Crates.io

```bash
cargo install podzol
```

### From AUR (Arch Linux)

```bash
paru -S podzol
```

## Quick Start

1. Create a new modpack:

```bash
podzol init --name "My Cool Pack" --version 1.21.1 --loader fabric
```

2. Add mods to your pack:

```bash
podzol add iris sodium modmenu
```

3. Export your modpack:

```bash
podzol export
```

## Configuration

Podzol uses a clean TOML format for modpack configuration:

```toml
[pack]
name = "Cool pack"
version = "0.1.0"
description = "A very cool minecraft modpack"

[enviroment]
minecraft = "1.21.1"
fabric = "0.16.10"

[mods]
iris = { version = "1.8.1+1.21.1-fabric", side = "client" }
sodium = { version = "mc1.21.1-0.6.5-fabric", side = "client" }
```

### Manifest Structure

- **Pack Information**: Basic metadata about your modpack
- **Environment**: Minecraft and mod loader versions
- **Mods**: Mod definitions with automatic version management
- **Resource Packs**: Optional resource pack configurations
- **Shaders**: Shader pack configurations
- **File Overrides**: Custom file management for client/server

## Commands

```bash
podzol init    # Create a new project
podzol add     # Add components to your modpack
podzol remove  # Remove components
podzol export  # Create a distributable package
```

## Roadmap

- Publishing capabilities
- Package extension support
- Additional mod platform integrations
- Advanced configuration options

## Comparison with Other Tools

| Feature              | Podzol      | Packwiz      | Manual Management |
| -------------------- | ----------- | ------------ | ----------------- |
| Configuration Format | Simple TOML | Complex TOML | N/A               |
| Version Management   | Automatic   | Manual       | Manual            |
| Modrinth Integration | Direct API  | Limited      | None              |
| Setup Complexity     | Minimal     | Complex      | None              |
| Learning Curve       | Gentle      | Steep        | N/A               |

## License

This project is licensed under the
[Apache-2.0 License](http://www.apache.org/licenses/LICENSE-2.0). For more
information, please see the [LICENSE](LICENSE) file.

# Rotary Generic Command

[OpenDeck](https://github.com/ninjadev64/OpenDeck) plugin that maps rotary encoder actions to custom shell commands.

| Action | Trigger |
|---|---|
| Clockwise rotation | CW command |
| Counter-clockwise rotation | CCW command |
| Short press | Press command |
| Long press (> 750 ms, fires on threshold) | Long press command |

All four commands are configured per-instance via the property inspector.

## Install

### From release

Download `info.degois.damien.opendeck.plugins.rotary-command.zip` from the [releases page](../../releases) and install it through OpenDeck.

### From source (Linux)

Requires the Rust toolchain. The supplied Makefile only knows the Linux XDG
plugin path; macOS and Windows users should install from the release zip.

```sh
make install
```

This builds and copies the plugin to `~/.config/opendeck/plugins/`.

## Build

```sh
make build            # build for current target
make package          # build + create installable .zip
make clean            # remove build artifacts
```

Cross-compile with `TARGET=aarch64-unknown-linux-gnu make build`.

> **Note on logging.** Shell commands are echoed to the log stream at debug
> level (`RUST_LOG=debug`). Avoid embedding secrets directly in the command —
> read them from a file or environment variable instead.

## Example

Volume control via PipeWire:

| Field | Command |
|---|---|
| CW | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+` |
| CCW | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-` |
| Press | `wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle` |
| Long press | `pavucontrol` |

## Acknowledgments

CI workflow originally inspired by [OpenActionPlugins/mpris](https://github.com/OpenActionPlugins/mpris); since extended for cross-platform builds and a macOS universal binary.

## License

MIT

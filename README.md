# Rotary Generic Command

[OpenDeck](https://github.com/ninjadev64/OpenDeck) plugin that maps rotary encoder actions to custom shell commands.

| Action | Trigger |
|---|---|
| Clockwise rotation | CW command |
| Counter-clockwise rotation | CCW command |
| Short press | Press command |
| Long press (> 500 ms) | Long press command |

All four commands are configured per-instance via the property inspector.

## Install

### From release

Download `info.degois.damien.opendeck.plugins.rotary-command.zip` from the [releases page](../../releases) and install it through OpenDeck.

### From source

Requires Rust toolchain.

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

## Example

Volume control via PipeWire:

| Field | Command |
|---|---|
| CW | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+` |
| CCW | `wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-` |
| Press | `wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle` |
| Long press | `pavucontrol` |

## Acknowledgments

CI workflow inspired by [OpenActionPlugins/mpris](https://github.com/OpenActionPlugins/mpris).

## License

MIT

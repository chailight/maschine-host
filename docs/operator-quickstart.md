# Operator quickstart

The host is a headless performance router for eight groups and encoder pages. On macOS it can read the full MK2 HID interface. Group, pad, and button lamps are written only after the check in `docs/led-lamps.md` matches. LCD panels are written only after the check in `docs/lcd-panels.md` matches. See `docs/hid-input.md`.

## Run the mock acceptance check

```sh
cargo test
cargo run -- --self-check
```

`self-check` loads `docs/example-config.yaml`, selects group B, plays a pad, changes group, releases the original note, moves an encoder, and disconnects. The Bitbox file sends the controllers listed in `docs/bitbox-control-pages.md`. `docs/encoder-panel-example.yaml` keeps empty controller numbers, so those encoders update the display and send nothing.

## Start the process

```sh
cargo run -- --config docs/example-config.yaml
```

Invalid YAML or a missing sample file exits with the path of the problem and does not activate a partial route. A missing MIDI port is printed and the process keeps running. SIGINT or SIGTERM sends note-offs and stops local voices.

`docs/example-config.yaml` opens the existing CoreMIDI destination `iConnectMIDI4+ DIN 4` for the `bitbox` route. `virtual_port: false` selects that port by its exact system name. The eight buttons above the displays select the control categories. See `docs/bitbox-control-pages.md`.

`docs/local-sampler.example.yaml` plays `samples/click.wav` on group A and a chromatic zone on group B. Set `audio.device` only after you have chosen an output. The value is recorded for the operator; the process does not open that device unless you build the `cpal-output` probe.

```sh
cargo run --features cpal-output -- --probe-audio
```

## Encoder pages

Page left and page right move inside the current category when the file declares the eight categories. Without categories they wrap the whole page list. Group buttons do not change the encoder page. The buttons above the displays are the category selectors.

Displayed numbers are the last value this host commanded. They are not the instrument's current value.

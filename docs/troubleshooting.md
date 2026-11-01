# Troubleshooting

## The MK2 does nothing

This build does not read or write the controller. `UnavailableAdapter` returns "transport is not implemented" for every LED or LCD submit, and note routing still runs. Confirm with `cargo run -- --self-check`. A hardware pass is `docs/hardware-smoke-checklist.md`.

## MIDI port is unavailable

The port id in a group must exist under `midi_outputs`, and the system name must match a live ALSA or CoreMIDI port exactly. The sample name `CONFIGURE_PORT_NAME` will not match. A failed note-on is discarded so it is not held. A later release does not invent a note-off for it. Panic still sends all-notes-off for notes that were confirmed.

On this development Mac an iConnectMIDI4+ is present. It is not opened unless the config names it. Do not treat it as a Bitbox.

## Null CC

`cc: null` is a placeholder. The encoder still stores and displays a 0-127 value. No CC is sent. Put a number in only after you have checked that firmware's documentation. A macro with a missing controller number is rejected at load.

## Stuck notes

Release uses the destination, channel, and note captured at note-on, including after a group change. Disconnect, panic, and shutdown send those note-offs, then CC 123 on each channel that had a held note, and they stop local voices. One-shot local playback ignores release on purpose. Panic still stops those voices.

Reload is refused while a pad is held or a one-shot voice is still counted.

## Audio

Only 16-bit integer PCM WAV is loaded. A float or 24-bit file fails preload and the configuration is not activated. If the output sink returns an error, MIDI continues. The latency printed at startup is buffer time, not a measured delay.

## Cold start leaves the PERFORM lamp off

Observed 2026-11-01 on a Maschine Controller MK2 (`17cc:1140`) after the hardware was powered on and the host was started for the first time with `docs/example-config.yaml`. Group lamps and pad lamps light. The first page button navigates. The PERFORM button above the displays stays dark, and pressing PERFORM does not light it. Quit the host and start it again without power-cycling the controller, and PERFORM lights.

The host already marks PERFORM on before the first write. Repeating the opening lamp and screen writes every 200 ms for the first 1.6 seconds does not clear it. Writing the silk-button report after the pads, groups, and screens, then once more on the next poll, does not clear it either. Not treated as blocking.

## Config errors

The loader prints `path: message` and activates nothing. Eight groups A-H, 16 notes, channels 1-16, and eight encoders per page are required. Schema version must be 1.

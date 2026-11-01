## Why

The host already reads pads and groups from the attached MK2 and builds a semantic lamp frame, then discards that frame. The selected group and the held pads never appear on the hardware. The input change deliberately sent no HID output, and no lamp position has been confirmed on this device.

## What Changes

- Confirm one pad lamp and one group lamp on the live MK2 before the map is stored. Leave `NIHardwareAgent` running unless it paints over that check. A mismatch, or any other lamp changing, stores nothing.
- Light the selected group and each held pad in one active color. Light the other groups and the idle pads in one resting color. Turn those lamps off when the controller is disconnected.
- Send group and pad LED output reports only after that check matches. A failed write is recoverable and does not block note routing.
- Keep the mock adapter for tests with no hardware. The semantic label model stays rate-limited and is still treated as commanded, not read back from an instrument.
- Allow those confirmed group and pad LED output reports. Feature reports, the class-compliant MIDI interface, the audio-control interface, and the DFU interface stay unused. The labeled input capture still sends nothing.
- Leave both LCD panels unwritten. Button lamps wait for a later change. Do not link Encdr. The repository is GPL-3.0-or-later, so a confirmed lamp map adapted from Encdr 0.7.8 may be stored with Encdr's copyright notice.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `display-feedback`: Group and pad lamps on the full MK2 show the selected group and held pads once a one-lamp check has confirmed those positions. LCD pixels stay unwritten, and the existing label model stays commanded rather than read back.
- `controller-input`: The ban on HID output reports is narrowed so the confirmed group and pad LED reports from `display-feedback` may be sent. Feature reports, the other MK2 interfaces, and output during the labeled input capture stay forbidden.

## Impact

- The live HID adapter gains a writer for confirmed group and pad lamps. The controller core, MIDI backend, and sampler keep their current contracts.
- `NIHardwareAgent` already holds a HID client on the development machine and may repaint the panel during the check.
- Linux hidraw, Raspberry Pi checks, button lamps, and both LCD panels are outside this change.
- The `display-feedback` Purpose line is updated when this change is applied so it no longer says lamp writes are outside that capability. Other capabilities keep that sentence, because they still do not fulfill lamp writes.

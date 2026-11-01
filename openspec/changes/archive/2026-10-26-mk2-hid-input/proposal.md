## Why

The host can route groups, pads, and encoders only from a mock source. A full-size Maschine MK2 is attached on this Mac, and the 2026-10-26 enumeration identified its HID interface, but the process never opens that interface. The other capabilities already specify what those events should do, and they stay unexercised by the controller until the events exist.

## What Changes

- Open the full MK2 HID interface (`17cc:1140`, interface 2, vendor usage page `0xFF01`) and turn confirmed input reports into the existing hardware-independent events.
- Keep the mock source when that interface is absent, and fail recoverably when it is present but cannot be read.
- Ignore the class-compliant MIDI interface, the audio-control interface, and the DFU interface. Do not send HID output or feature reports.
- Drive the existing group, pad, encoder, MIDI, and sampler behavior from those events. Their requirements stay as they are.
- Record a second labeled survey for the remaining silk-screen controls: the eight buttons above the displays, Control, Step, Browse, Sampling, the Left and Right buttons beside the displays, All, Auto Write, Volume, Swing, Tempo, the master Left, Right, and Enter buttons, Note Repeat, the data encoder, Restart, the transport Left and Right buttons, Grid, Play, Rec, Erase, Shift, Scene, Pattern, Pad Mode, Navigate, Duplicate, Select, Solo, and Mute. Knobs are turned one detent in each direction. None of these controls emits an event until its bytes are assigned, and none is guessed onto page, panic, or octave.
- Replace the placeholder Purpose text on every current main spec so each capability states why it exists.
- Leave physical LCD and LED updates unmet. The spike recorded report sizes only and did not decode a panel or lamp layout.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `controller-input`: The host reads the full MK2 HID interface when it is present, emits events only for controls confirmed by a labeled capture, and still supplies a mock source when the interface is absent. A further labeled survey names the remaining silk-screen controls and keeps each gesture with that name until an assignment maps it.
- `runtime`: A macOS process with the full MK2 attached uses that HID interface for controller operation. Mock tests on Linux and macOS stay available. The existing headless Linux requirement is unchanged.

## Impact

- The hardware adapter gains a macOS HID reader. The controller core, MIDI backend, and sampler keep their current contracts.
- `NIHardwareAgent` and Logitech Options+ already hold HID clients on the development machine. A refused open must not be treated as a successful connection.
- Linux hidraw, Raspberry Pi checks, Bitbox CC assignments, and panel or LED writes are outside this change.
- Main spec Purpose lines are documentation. They do not by themselves add behavior.

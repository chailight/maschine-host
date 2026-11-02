## Why

Every Bitbox channel starts hard left, and a hard hit on one pad also starts the pads beside and below it. Both showed up in hardware use. They are separate from the Maschine project converter and from pad-mode configuration.

## What Changes

- The eight pan encoders start at 64. Pan in the preset stays saved full left, so 64 remains the centre of the learned travel.
- When the controller connects, the host sends controller 38 at 64 on channels 1–8. Until that message, the module stays at the saved full-left position, because a stored default is not heard until it is sent.
- A pad press requires pressure above a floor on both copies of that pad. Coupling from a neighbor, such as pad 16 also starting pads 15, 12, and sometimes 8, does not become a note. The 36-report release hold stays as it is: it only delays the end of a pad that has already gone down.
- The floor is taken from a capture of one hard hit and the quiet pads around it. It is high enough to reject that coupling and low enough that a deliberate light hit still plays.

## Capabilities

### New Capabilities

### Modified Capabilities

- `mixer-controls`: Each slot's pan starts at 64 and that value is sent on connect.
- `controller-input`: A pad press requires both pressure copies to clear a measured floor.

## Impact

- `docs/example-config.yaml` pan defaults, and the connect path that today sends no controller messages.
- Pad pressure reading in the HID assignment path. The velocity curve still runs after a press is accepted, and the report layout stays the same.
- Tests for the pan messages and for a below-floor neighbor that must not press.
- The host template's saved pan, the Maschine-to-Bitbox converter, and `pad-mode-config` stay unchanged.

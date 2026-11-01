## MODIFIED Requirements

### Requirement: MK2 inputs primary behavior
The system SHALL normalize a pad, Group button, encoder, or page control into a hardware-independent event with control ID, action, value, and timestamp. When a full-size Maschine MK2 HID interface is present, the system SHALL read that interface. The system SHALL emit an event only for a control confirmed by a labeled capture. The system SHALL NOT use the MK2 class-compliant MIDI interface, audio-control interface, or DFU interface as that input, and SHALL NOT send HID output or feature reports.

#### Scenario: Primary operation
- **WHEN** a pad, Group button, encoder, or page control is operated
- **THEN** the system normalizes the event into a hardware-independent event with control ID, action, value, and timestamp

#### Scenario: Full MK2 HID interface
- **WHEN** a device identifies itself as vendor `17cc`, product `1140`, and exposes the MK2 HID interface
- **THEN** the system reads input reports from that HID interface and emits a normalized event for each confirmed control

#### Scenario: Unconfirmed input
- **WHEN** an input report contains a control that the labeled capture has not confirmed
- **THEN** the system emits no control event for that control

#### Scenario: Other MK2 interfaces
- **WHEN** the attached MK2 also exposes a class-compliant MIDI interface, an audio-control interface, or a DFU interface
- **THEN** the system does not read those interfaces as pad, group, encoder, or page input and does not send HID output or feature reports

### Requirement: Safety and secondary behavior
The system SHALL support a mock input source for automated tests when hardware is absent. When the full MK2 HID interface is present but cannot be read, the system SHALL report a recoverable error and SHALL NOT present the controller as connected.

#### Scenario: Secondary operation
- **WHEN** hardware is absent
- **THEN** the system supports a mock input source for automated tests

#### Scenario: HID open refused
- **WHEN** the full MK2 HID interface is present and the system cannot read it
- **THEN** the system reports a recoverable error and does not present the controller as connected

## ADDED Requirements

### Requirement: Remaining silk-screen survey
The system SHALL record a labeled input survey of the remaining MK2 silk-screen controls: the eight buttons above the displays from left to right, Control, Step, Browse, Sampling, the Left and Right buttons beside the displays, All, Auto Write, the Volume, Swing, and Tempo knobs in both directions, the master Left, Right, and Enter buttons, Note Repeat, the data encoder in both directions, Restart, the transport Left and Right buttons, Grid, Play, Rec, Erase, Shift, Scene, Pattern, Pad Mode, Navigate, Duplicate, Select, Solo, and Mute. The capture SHALL use the same timed input log as the pad, group, and screen-knob session. The operator performs these controls in that agreed order, with a pause between them, and each distinct gesture is associated with that control afterwards. The system SHALL NOT emit a control event for one of these controls until an assignment maps it, and SHALL NOT send HID output or feature reports during the capture.

#### Scenario: Named gesture is recorded
- **WHEN** the operator performs these controls in the agreed order during a timed HID capture
- **THEN** the input reports are recorded in that order so each distinct gesture can be associated with that control's name

#### Scenario: Unmapped silk-screen control
- **WHEN** one of these controls is operated and the assignment has no field for it
- **THEN** the system emits no control event

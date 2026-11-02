# MK2 inputs

## Purpose

Normalizes pad, group, encoder, and page controls into hardware-independent events. When the full MK2 HID interface is present, the host reads that interface and emits events only for controls confirmed by a labeled capture. A matched lamp map may send a group, pad, or button LED output report. Feature reports stay unused.

## Requirements

### Requirement: MK2 inputs primary behavior
The system SHALL normalize a pad, Group button, encoder, or page control into a hardware-independent event with control ID, action, value, and timestamp. When a full-size Maschine MK2 HID interface is present, the system SHALL read that interface. The system SHALL emit an event only for a control confirmed by a labeled capture. The system SHALL NOT use the MK2 class-compliant MIDI interface, audio-control interface, or DFU interface as that input. The system SHALL NOT send HID feature reports. The system SHALL NOT send a HID output report except a group, pad, or button LED report from a lamp map that a one-lamp check has matched on the attached MK2, or an LCD panel report from a panel map that a one-panel check has matched on the attached MK2.

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
- **THEN** the system does not read those interfaces as pad, group, encoder, or page input, does not send HID feature reports, and does not send a HID output report except a group, pad, or button LED report from a matched lamp map or an LCD panel report from a matched panel map

#### Scenario: Confirmed lamp output
- **WHEN** the one-lamp check has matched the stored lamp map and a group is selected
- **THEN** the system sends that group's LED output report and does not send a HID feature report

#### Scenario: Confirmed panel output
- **WHEN** the one-panel check has matched the stored panel map and an encoder page is selected
- **THEN** the system sends that page's LCD panel reports and does not send a HID feature report

### Requirement: Safety and secondary behavior
The system SHALL support a mock input source for automated tests when hardware is absent. When the full MK2 HID interface is present but cannot be read, the system SHALL report a recoverable error and SHALL NOT present the controller as connected.

#### Scenario: Secondary operation
- **WHEN** hardware is absent
- **THEN** the system supports a mock input source for automated tests

#### Scenario: HID open refused
- **WHEN** the full MK2 HID interface is present and the system cannot read it
- **THEN** the system reports a recoverable error and does not present the controller as connected

### Requirement: Remaining silk-screen survey
The system SHALL record a labeled input survey of the remaining MK2 silk-screen controls: the eight buttons above the displays from left to right, Control, Step, Browse, Sampling, the Left and Right buttons beside the displays, All, Auto Write, the Volume, Swing, and Tempo knobs in both directions, the master Left, Right, and Enter buttons, Note Repeat, the data encoder in both directions, Restart, the transport Left and Right buttons, Grid, Play, Rec, Erase, Shift, Scene, Pattern, Pad Mode, Navigate, Duplicate, Select, Solo, and Mute. The capture SHALL use the same timed input log as the pad, group, and screen-knob session. The operator performs these controls in that agreed order, with a pause between them, and each distinct gesture is associated with that control afterwards. The system SHALL NOT emit a control event for one of these controls until an assignment maps it, and SHALL NOT send HID output or feature reports during the capture.

#### Scenario: Named gesture is recorded
- **WHEN** the operator performs these controls in the agreed order during a timed HID capture
- **THEN** the input reports are recorded in that order so each distinct gesture can be associated with that control's name

#### Scenario: Unmapped silk-screen control
- **WHEN** one of these controls is operated and the assignment has no field for it
- **THEN** the system emits no control event

### Requirement: Encoder page buttons
The Left and Right buttons beside the displays SHALL select the encoder page. When the loaded pages do not declare categories, a press of the Right button SHALL select the next page and a press of the Left button SHALL select the previous page, wrapping the full list, and each of those buttons SHALL light only while it is held. When the pages declare categories, Left and Right SHALL wrap inside the active category and the host SHALL remember that category's page. The Left lamp SHALL be on when the active category has an earlier subpage, and the Right lamp SHALL be on when it has a later subpage. A category with one subpage SHALL light neither arrow. Releasing either button SHALL leave those lamps as the page position requires. The eight buttons above the displays SHALL select the eight categories when categories are declared, as one radio, and SHALL NOT change the selected group.

#### Scenario: Next page
- **WHEN** the Right button beside the displays is pressed, a later encoder page exists, and the pages do not declare categories
- **THEN** the next encoder page is selected

#### Scenario: Previous page wraps
- **WHEN** the first encoder page is selected, the pages do not declare categories, and the Left button beside the displays is pressed
- **THEN** the last encoder page is selected

#### Scenario: Page button wraps inside the category
- **WHEN** the Bitbox configuration is loaded and the Right button is pressed on the last MIX subpage
- **THEN** MIX shows its first subpage and the selected group is unchanged

#### Scenario: Page button lamp
- **WHEN** the Left or Right button beside the displays is held, and then released, and the pages do not declare categories
- **THEN** that button's lamp is on while it is held and off after release

#### Scenario: Category arrow shows another page
- **WHEN** the Bitbox configuration is loaded and MIX is on its first subpage
- **THEN** the Right lamp is on and the Left lamp is off, and releasing Right leaves the Right lamp on

### Requirement: Pad press clears a pressure floor
A pad press SHALL be emitted only when both copies of that pad's pressure are strictly above a recorded floor. Pressure at or below the floor SHALL NOT emit a press, including when the other pads in the same report are above the floor. The floor SHALL be the smallest whole count that sits above every coupled reading in one captured hard hit and at or below a deliberate light hit on the struck pad. The capture SHALL include the struck pad and the pads that also sounded, which for a hit on pad 16 are pad 15, pad 12, and pad 8. The 36-report wait before a pad release SHALL stay in place for a pad that has already gone down. A rise during that wait SHALL remain the same hit. Velocity scaling and the velocity curve SHALL run only after a press is accepted. The pad report layout SHALL stay unchanged.

#### Scenario: A neighbor below the floor does not press
- **WHEN** one pad's pressure is above the floor on both copies and a neighboring pad's pressure on both copies is above zero and at or below the floor
- **THEN** the system emits a press for the struck pad only

#### Scenario: Pressure at the floor is not a hit
- **WHEN** both copies of a pad read exactly the floor and the previous report read zero
- **THEN** the system emits no press for that pad

#### Scenario: One copy above the floor is not a hit
- **WHEN** one copy of a pad is above the floor and the other copy is zero
- **THEN** the system emits no press for that pad

#### Scenario: A dip while the finger is down is the same hit
- **WHEN** a pad has pressed and its pressure returns to zero for fewer reports than the release wait, then rises above the floor again
- **THEN** the system emits no second press and no release for that pad

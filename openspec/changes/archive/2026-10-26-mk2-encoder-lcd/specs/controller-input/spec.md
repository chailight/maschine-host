## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: Encoder page buttons
The Left and Right buttons beside the displays SHALL select the encoder page. A press of the Right button SHALL select the next page. A press of the Left button SHALL select the previous page. The selection SHALL wrap at either end. Each of those buttons SHALL light only while it is held.

#### Scenario: Next page
- **WHEN** the Right button beside the displays is pressed and a later encoder page exists
- **THEN** the next encoder page is selected

#### Scenario: Previous page wraps
- **WHEN** the first encoder page is selected and the Left button beside the displays is pressed
- **THEN** the last encoder page is selected

#### Scenario: Page button lamp
- **WHEN** the Left or Right button beside the displays is held, and then released
- **THEN** that button's lamp is on while it is held and off after release

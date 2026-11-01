## MODIFIED Requirements

### Requirement: LCD and LED feedback primary behavior
The system SHALL keep readable labels and active selections in the host display model when the group or encoder page changes. The system SHALL NOT write those labels to the MK2 LCD panels.

#### Scenario: Primary operation
- **WHEN** the group or encoder page changes
- **THEN** the system keeps readable labels and active selections in the host display model and does not write the LCD panels

## ADDED Requirements

### Requirement: Group and pad lamps
The system SHALL light each group in that group's own color after a one-lamp check on the attached full-size MK2 has matched one pad and one group to the stored lamp map. The selected group and each held pad SHALL use the selected group's full color. Each other group SHALL use a dimmer shade of its own color, and each idle pad SHALL use a dimmer shade of the selected group's color. The system SHALL light a silk-screen button while its gesture says it is on: one pad-mode button at a time, one of Volume, Swing, Tempo, or Note Repeat at a time, each display button as a toggle, and every other labeled button only while it is held. Until that check matches, the system SHALL NOT send a group, pad, or button LED output report. The system SHALL turn the group, pad, and button lamps off when the controller is not connected. The system SHALL NOT write either LCD panel. A failed lamp write SHALL be reported as a recoverable error and SHALL NOT block note routing.

#### Scenario: Selected group and held pad
- **WHEN** a group is selected and a pad is held, and the one-lamp check has matched the stored lamp map
- **THEN** the selected group and the held pad show that group's full color, idle pads show the dimmer shade of that color, and each other group shows the dimmer shade of its own color

#### Scenario: Check matches one pad and one group
- **WHEN** one commanded pad lamp and one commanded group lamp match the stored map, and no other lamp changes
- **THEN** the system uses that map for every group and pad lamp

#### Scenario: Check does not match
- **WHEN** the commanded pad or group lights a different control, or another lamp changes
- **THEN** the system does not store that map and sends no further group or pad LED output report

#### Scenario: Map not yet matched
- **WHEN** the one-lamp check has not matched the stored lamp map
- **THEN** the system sends no group or pad LED output report

#### Scenario: Controller disconnected
- **WHEN** the controller is not connected
- **THEN** the group and pad lamps are off

#### Scenario: Lamp write fails
- **WHEN** a confirmed lamp write fails
- **THEN** the system reports a recoverable error and note routing continues

#### Scenario: Panels stay unchanged
- **WHEN** the group, pad, and button lamps are updated
- **THEN** both LCD panels stay unwritten

#### Scenario: Silk-screen button gesture
- **WHEN** a labeled silk-screen button is pressed and the lamp map is matched
- **THEN** a pad-mode press lights only that mode button, a Volume, Swing, Tempo, or Note Repeat press lights only that button in its set, a display button toggles, and a momentary button lights only while held

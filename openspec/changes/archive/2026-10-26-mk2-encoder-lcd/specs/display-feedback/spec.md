## MODIFIED Requirements

### Requirement: LCD and LED feedback primary behavior
The system SHALL keep readable labels and active selections in the host display model when the group or encoder page changes. After a one-panel check has matched the stored panel map, the system SHALL write the active encoder page to the MK2 LCD panels. Until that check matches, the system SHALL NOT write the LCD panels.

#### Scenario: Primary operation
- **WHEN** the group or encoder page changes and the panel map is matched
- **THEN** the system keeps readable labels and active selections in the host display model and writes the active encoder page to the LCD panels

#### Scenario: Panel map not matched
- **WHEN** the group or encoder page changes and the one-panel check has not matched
- **THEN** the system keeps the labels in the host display model and does not write the LCD panels

### Requirement: Group and pad lamps
The system SHALL light each group in that group's own color after a one-lamp check on the attached full-size MK2 has matched one pad and one group to the stored lamp map. The selected group and each held pad SHALL use the selected group's full color. Each other group SHALL use a dimmer shade of its own color, and each idle pad SHALL use a dimmer shade of the selected group's color. The system SHALL light a silk-screen button while its gesture says it is on: one pad-mode button at a time, one of Volume, Swing, Tempo, or Note Repeat at a time, each display button as a toggle, and every other labeled button only while it is held. Until that check matches, the system SHALL NOT send a group, pad, or button LED output report. The system SHALL turn the group, pad, and button lamps off when the controller is not connected. A lamp report SHALL NOT contain LCD panel bytes. A failed lamp write SHALL be reported as a recoverable error and SHALL NOT block note routing.

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
- **THEN** the lamp report contains no LCD panel bytes

#### Scenario: Silk-screen button gesture
- **WHEN** a labeled silk-screen button is pressed and the lamp map is matched
- **THEN** a pad-mode press lights only that mode button, a Volume, Swing, Tempo, or Note Repeat press lights only that button in its set, a display button toggles, and a momentary button lights only while held

## ADDED Requirements

### Requirement: Encoder page panels
The system SHALL write both MK2 LCD panels from the active encoder page after a one-panel check on the attached full-size MK2 has matched one panel to the stored panel map. The left panel SHALL show encoders 1 through 4 in four columns, and the right panel SHALL show encoders 5 through 8 in four columns. Each column SHALL sit above that encoder and SHALL place its configured label above the value stored for that encoder on the active page, with the value on the bottom line. The active page name SHALL be visible across the top of the panels. Turning an encoder SHALL change the stored value for that encoder on the active page only, and the panels SHALL show the new value. Selecting another page SHALL show the values stored for that page and SHALL leave every other page's stored values unchanged. The system SHALL clear both panels when the controller is not connected. Until the check matches, the system SHALL NOT send an LCD panel report. A failed panel write SHALL be reported as a recoverable error and SHALL NOT block note routing. The Bitbox configuration SHALL keep the delay, reverb, and filter pages and SHALL assign no encoder action in this change. The proof-of-concept example SHALL load two encoder pages, label each encoder `enc 1` through `enc 8`, and assign no encoder action.

#### Scenario: Active page on the panels
- **WHEN** the panel map is matched and an encoder page is selected
- **THEN** the left panel shows encoders 1 through 4 in four columns, the right panel shows encoders 5 through 8 in four columns, each column places that encoder's label above its stored value, and the page name is visible across the top

#### Scenario: Knob changes the active page only
- **WHEN** an encoder is turned on the active page and another page has a stored value for that encoder
- **THEN** the active page's stored value changes, the other page's stored value is unchanged, and the panels show the active page's new value

#### Scenario: Page change shows the other page's values
- **WHEN** another encoder page is selected
- **THEN** the panels show that page's stored encoder values and the page that was left keeps its stored values

#### Scenario: Bitbox pages stay
- **WHEN** the Bitbox configuration is loaded
- **THEN** the delay, reverb, and filter pages are present, no encoder action is assigned, and the panels show those page names and their configured encoder labels

#### Scenario: Two generic pages
- **WHEN** the proof-of-concept example is loaded
- **THEN** two encoder pages are available, each encoder is labeled `enc 1` through `enc 8`, and no encoder action is assigned

#### Scenario: Check matches one panel
- **WHEN** one commanded panel matches the stored map and the other panel and the lamps stay unchanged
- **THEN** the system uses that map for both panels

#### Scenario: Check does not match
- **WHEN** the commanded panel does not match, or another panel or a lamp changes
- **THEN** the system does not store that map and sends no further LCD panel report

#### Scenario: Map not yet matched
- **WHEN** the one-panel check has not matched the stored panel map
- **THEN** the system sends no LCD panel report

#### Scenario: Controller disconnected
- **WHEN** the controller is not connected
- **THEN** both LCD panels are clear

#### Scenario: Panel write fails
- **WHEN** a confirmed panel write fails
- **THEN** the system reports a recoverable error and note routing continues

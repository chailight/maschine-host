## MODIFIED Requirements

### Requirement: Group and pad lamps
The system SHALL light each group in that group's own color after a one-lamp check on the attached full-size MK2 has matched one pad and one group to the stored lamp map. The selected group and each held pad SHALL use the selected group's full color. Each other group SHALL use a dimmer shade of its own color, and each idle pad SHALL use a dimmer shade of the selected group's color. The system SHALL light a silk-screen button while its gesture says it is on: the Pad Mode lamp while the setup screen is engaged, one of Scene, Pattern, Navigate, Duplicate, Select, Solo, or Mute at a time, one of Volume, Swing, Tempo, or Note Repeat at a time, each display button as a toggle, and every other labeled button only while it is held. Until that check matches, the system SHALL NOT send a group, pad, or button LED output report. The system SHALL turn the group, pad, and button lamps off when the controller is not connected. A lamp report SHALL NOT contain LCD panel bytes. A failed lamp write SHALL be reported as a recoverable error and SHALL NOT block note routing.

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
- **THEN** a Pad Mode press toggles only the Pad Mode lamp, a Scene or Pattern press lights only that button among Scene, Pattern, Navigate, Duplicate, Select, Solo, and Mute and leaves the Pad Mode lamp as it was, a Volume, Swing, Tempo, or Note Repeat press lights only that button in its set, a display button toggles, and a momentary button lights only while held

### Requirement: Encoder page panels
The system SHALL write both MK2 LCD panels from the active encoder page after a one-panel check on the attached full-size MK2 has matched one panel to the stored panel map. The left panel SHALL show encoders 1 through 4 in four columns, and the right panel SHALL show encoders 5 through 8 in four columns. Each column SHALL sit above that encoder and SHALL place its configured label above the value stored for that encoder on the active page, with the value on the bottom line. An on/off value SHALL show On at 64 and above and Off below 64. When the configuration declares eight categories and the Pad Mode setup screen is off, each column SHALL show that category's name on its top line, under the button above it, and SHALL NOT draw the subpage name. The lit category button identifies the active page. Without categories, the active page name SHALL be visible across the top of the panels. While the Pad Mode setup screen is off, turning an encoder SHALL change the stored value for that encoder. A value that is not shared SHALL change on the active page only. A shared parameter SHALL show the same value on every page that uses it. Selecting another page SHALL show the values stored for that page and SHALL leave an unshared value on the page that was left unchanged. The system SHALL clear both panels when the controller is not connected. Until the check matches, the system SHALL NOT send an LCD panel report. A failed panel write SHALL be reported as a recoverable error and SHALL NOT block note routing. The displayed value is the last command from the host. The proof-of-concept example SHALL load two encoder pages, label each encoder `enc 1` through `enc 8`, and assign no encoder action.

#### Scenario: Active page on the panels
- **WHEN** the panel map is matched, the pages do not declare categories, an encoder page is selected, and the Pad Mode setup screen is off
- **THEN** the left panel shows encoders 1 through 4 in four columns, the right panel shows encoders 5 through 8 in four columns, each column places that encoder's label above its stored value, and the page name is visible across the top

#### Scenario: Category names under the buttons
- **WHEN** the Bitbox configuration is loaded, MIX is selected, and the Pad Mode setup screen is off
- **THEN** the second column shows MIX on its top line

#### Scenario: On and off text
- **WHEN** an on/off encoder on the Bitbox configuration is at 64 and the Pad Mode setup screen is off
- **THEN** that column shows On

#### Scenario: Knob changes the active page only
- **WHEN** an encoder is turned on the active page, the Pad Mode setup screen is off, the encoder does not share its parameter, and another page has a stored value for that encoder
- **THEN** the active page's stored value changes, the other page's stored value is unchanged, and the panels show the active page's new value

#### Scenario: Page change shows the other page's values
- **WHEN** another encoder page is selected and the Pad Mode setup screen is off
- **THEN** the panels show that page's stored encoder values and an unshared value on the page that was left is unchanged

#### Scenario: Bitbox pages stay
- **WHEN** the Bitbox configuration is loaded and the Pad Mode setup screen is off
- **THEN** the eight control categories are present, each column shows its category name, and the encoders send the controllers configured for those pages

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

## ADDED Requirements

### Requirement: Pad Mode setup screen
While the Pad Mode setup screen is engaged and the panel map is matched, both LCDs SHALL be titled PAD MODE and SHALL show the selected group's letter and label. The left LCD SHALL show four columns labeled Vel, Scale, Root, and Oct. The right LCD SHALL show Type in its first column. Vel SHALL show that group's velocity curve. Scale SHALL show `notes` when the group uses its note list, and otherwise the mode label `ionian`, `dorian`, `phrygian`, `lydian`, `mixolydian`, `aeolian`, `locrian`, `pent maj`, `pent min`, or `chromatic`. Root SHALL show the root as a note name, with C4 at MIDI note 60. Oct SHALL show the octave offset as `0` or a signed integer for every group. Type SHALL show `sample`, `granular`, `clip`, `slicer`, or `none`. Category names and encoder-page values SHALL NOT be drawn. Encoders 6–8 SHALL send nothing. Leaving the screen SHALL draw the active encoder page again.

#### Scenario: Fields for the selected group
- **WHEN** the setup screen is engaged on Group C, whose label is Scale map, whose curve is 0.5, whose scale is `natural-minor` with root 48, whose octave offset is 0, and whose pad type was omitted
- **THEN** both panels are titled PAD MODE and name Group C, the left columns show 0.5, `aeolian`, C3, and 0, and the right panel's first column shows `sample`

#### Scenario: One-shot columns
- **WHEN** the setup screen is engaged on a one-shot group whose first note is 36, with no named scale and an omitted pad type
- **THEN** the Scale column shows `notes`, the Root column shows C2, and the Oct column shows 0

#### Scenario: Category page returns
- **WHEN** MIX is the active category and the user leaves the setup screen
- **THEN** the panels show the MIX page, including MIX on the second column

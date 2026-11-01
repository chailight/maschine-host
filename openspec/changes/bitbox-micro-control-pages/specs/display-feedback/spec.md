## MODIFIED Requirements

### Requirement: Encoder page panels
The system SHALL write both MK2 LCD panels from the active encoder page after a one-panel check on the attached full-size MK2 has matched one panel to the stored panel map. The left panel SHALL show encoders 1 through 4 in four columns, and the right panel SHALL show encoders 5 through 8 in four columns. Each column SHALL sit above that encoder and SHALL place its configured label above the value stored for that encoder on the active page, with the value on the bottom line. An on/off value SHALL show On at 64 and above and Off below 64. When the configuration declares eight categories, each column SHALL show that category's name on its top line, under the button above it, and SHALL NOT draw the subpage name. The lit category button identifies the active page. Without categories, the active page name SHALL be visible across the top of the panels. Turning an encoder SHALL change the stored value for that encoder. A value that is not shared SHALL change on the active page only. A shared parameter SHALL show the same value on every page that uses it. Selecting another page SHALL show the values stored for that page and SHALL leave an unshared value on the page that was left unchanged. The system SHALL clear both panels when the controller is not connected. Until the check matches, the system SHALL NOT send an LCD panel report. A failed panel write SHALL be reported as a recoverable error and SHALL NOT block note routing. The displayed value is the last command from the host. The proof-of-concept example SHALL load two encoder pages, label each encoder `enc 1` through `enc 8`, and assign no encoder action.

#### Scenario: Active page on the panels
- **WHEN** the panel map is matched, the pages do not declare categories, and an encoder page is selected
- **THEN** the left panel shows encoders 1 through 4 in four columns, the right panel shows encoders 5 through 8 in four columns, each column places that encoder's label above its stored value, and the page name is visible across the top

#### Scenario: Category names under the buttons
- **WHEN** the Bitbox configuration is loaded and MIX is selected
- **THEN** the second column shows MIX on its top line

#### Scenario: On and off text
- **WHEN** an on/off encoder on the Bitbox configuration is at 64
- **THEN** that column shows On

#### Scenario: Knob changes the active page only
- **WHEN** an encoder is turned on the active page, the encoder does not share its parameter, and another page has a stored value for that encoder
- **THEN** the active page's stored value changes, the other page's stored value is unchanged, and the panels show the active page's new value

#### Scenario: Page change shows the other page's values
- **WHEN** another encoder page is selected
- **THEN** the panels show that page's stored encoder values and an unshared value on the page that was left is unchanged

#### Scenario: Bitbox pages stay
- **WHEN** the Bitbox configuration is loaded
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

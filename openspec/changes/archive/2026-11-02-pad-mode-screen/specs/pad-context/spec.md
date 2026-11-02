## MODIFIED Requirements

### Requirement: Contextual pad sound design
The host SHALL use the selected group's current pad type. An omitted pad type SHALL load as sample. A configured type is replaced for the session when the Pad Mode setup screen sets another one. The host SHALL NOT discover the type from the module. PAD subpages SHALL include only controllers the registry allows for that type. A group whose current pad type is unset SHALL show the PAD category unavailable and SHALL send nothing from it. The setup screen SHALL NOT start a clip-hold layer.

#### Scenario: Switch from granular to Clip
- **WHEN** the user selects a group configured as clip while PAD is showing a granular subpage
- **THEN** PAD shows clip subpages, granular controllers send nothing, and the pads send the clip group's notes

#### Scenario: Type chosen on the setup screen
- **WHEN** the setup screen sets the selected group's pad type to `granular` and the user leaves the screen for PAD
- **THEN** PAD shows granular subpages

#### Scenario: Cleared type hides the pad pages
- **WHEN** the setup screen sets the selected group's pad type to `none` and the user opens PAD
- **THEN** PAD is unavailable and its encoders send nothing

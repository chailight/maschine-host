# Pad Context Specification

## ADDED Requirements

### Requirement: Contextual pad sound design
The host SHALL use the pad type configured on the group. It SHALL NOT discover the type from the module. PAD subpages SHALL include only controllers the registry allows for that type. A group with no pad type SHALL show the PAD category unavailable and SHALL send nothing from it.

#### Scenario: Switch from granular to Clip
- **WHEN** the user selects a group configured as clip while PAD is showing a granular subpage
- **THEN** PAD shows clip subpages, granular controllers send nothing, and the pads send the clip group's notes

### Requirement: Safe momentary actions
A clip hold SHALL send 64 or above while held and below 64 on release. Release SHALL also be sent when the user leaves the subpage, changes group, disconnects, or shuts down. The 16 pads SHALL keep sending notes. A pad SHALL send a clip hold only while an explicit clip-hold layer is active. Opening PAD or pressing Pad Mode SHALL NOT start that layer. Controllers 120 and 123 SHALL NOT be used as holds.

#### Scenario: Exit held roll
- **WHEN** roll is held and the user leaves the clip-hold layer
- **THEN** the host sends the roll controller below 64 and the pads return to sending notes

#### Scenario: Pads stay notes
- **WHEN** PAD is selected and no clip-hold layer is active
- **THEN** a pad press sends the selected group's note and sends no clip-hold controller

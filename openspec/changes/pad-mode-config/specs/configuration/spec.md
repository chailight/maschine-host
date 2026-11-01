## ADDED Requirements

### Requirement: Per-group pad mode fields
The configuration SHALL accept an optional velocity curve on each group and an optional root and scale name on a scale-mode group. An omitted curve SHALL mean 1. A curve that is not a positive number SHALL be rejected. A scale name outside `major`, `natural-minor`, `major-pentatonic`, `minor-pentatonic`, and `chromatic` SHALL be rejected. A root outside 0 through 127 SHALL be rejected. A scale set on a one-shot or chromatic group SHALL be rejected. An invalid field SHALL be reported and the configuration SHALL NOT become active. The Bitbox file SHALL set velocity curve 0.5 on every group and SHALL omit the scale, leaving notes 36 through 51.

#### Scenario: Omitted curve is straight
- **WHEN** a group omits its velocity curve
- **THEN** the loaded curve is 1

#### Scenario: Invalid curve
- **WHEN** a group sets a velocity curve of 0 or below
- **THEN** validation fails and the configuration does not become active

#### Scenario: Scale on a drum group
- **WHEN** a one-shot group sets a root and scale name
- **THEN** validation fails and the configuration does not become active

#### Scenario: Bitbox curve
- **WHEN** the Bitbox configuration is loaded
- **THEN** every group uses velocity curve 0.5, omits a named scale, and keeps notes 36 through 51

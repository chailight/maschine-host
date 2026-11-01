# LCD and LED feedback

## ADDED Requirements

### Requirement: LCD and LED feedback primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** group or encoder page changes
- **THEN** WHEN group or encoder page changes, THE SYSTEM SHALL display readable labels and active selections on the MK2.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN an encoder changes, THE SYSTEM SHALL update its displayed value without assuming external-device feedback; LCD rendering SHALL be rate-limited.

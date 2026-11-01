# Independent encoder pages

## ADDED Requirements

### Requirement: Independent encoder pages primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** an encoder page is selected
- **THEN** WHEN an encoder page is selected, THE SYSTEM SHALL load eight independent mappings and update the display.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN an encoder changes, THE SYSTEM SHALL emit only the configured action or explicit macro targets, preserving a bounded application-side value.

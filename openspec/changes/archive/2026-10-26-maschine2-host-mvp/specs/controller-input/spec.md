# MK2 inputs

## ADDED Requirements

### Requirement: MK2 inputs primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** a pad, Group button, encoder or page control is operated
- **THEN** WHEN a pad, Group button, encoder or page control is operated, THE SYSTEM SHALL normalize the event into a hardware-independent event with control ID, action, value and timestamp.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN hardware is absent, THE SYSTEM SHALL support a mock input source for automated tests.

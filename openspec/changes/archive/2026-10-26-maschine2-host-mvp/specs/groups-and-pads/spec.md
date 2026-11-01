# Group and pad performance

## ADDED Requirements

### Requirement: Group and pad performance primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** a Group A-H button is pressed
- **THEN** WHEN a Group A-H button is pressed, THE SYSTEM SHALL select that group without changing the encoder page.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN a pad is pressed, THE SYSTEM SHALL route its configured note/sample with velocity to the selected group destination. WHEN a held pad is released after switching groups, THE SYSTEM SHALL send note-off to the original destination and note.

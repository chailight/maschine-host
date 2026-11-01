# Portable headless runtime

## ADDED Requirements

### Requirement: Portable headless runtime primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** started without a graphical desktop
- **THEN** WHEN started without a graphical desktop, THE SYSTEM SHALL provide MIDI/controller operation on supported Linux hardware.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN started with a mock hardware adapter, THE SYSTEM SHALL run state and routing tests on Linux and macOS without an attached MK2.

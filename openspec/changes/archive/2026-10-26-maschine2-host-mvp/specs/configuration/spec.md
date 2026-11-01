# Declarative configuration

## ADDED Requirements

### Requirement: Declarative configuration primary behavior
The system SHALL implement the behavior specified below.

#### Scenario: Primary operation
- **WHEN** a YAML configuration is loaded
- **THEN** WHEN a YAML configuration is loaded, THE SYSTEM SHALL validate groups, pad maps, destinations, channels and encoder mappings before activation.

### Requirement: Safety and secondary behavior
The system SHALL implement the following additional behavior.

#### Scenario: Secondary operation
- **WHEN** the relevant condition occurs
- **THEN** WHEN configuration is invalid, THE SYSTEM SHALL report actionable validation errors and avoid partially activating unsafe routes.

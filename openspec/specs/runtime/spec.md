# Portable headless runtime

## Purpose

Runs MIDI and controller operation without a graphical desktop on supported Linux hardware. On macOS, controller input comes from the full MK2 HID interface when it is present. Mock tests still run on Linux and macOS with no controller attached. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: Portable headless runtime primary behavior
The system SHALL provide MIDI and controller operation on supported Linux hardware when started without a graphical desktop. On macOS, when a full-size Maschine MK2 HID interface is present, the system SHALL use that interface for controller operation.

#### Scenario: Primary operation
- **WHEN** the system is started without a graphical desktop
- **THEN** the system provides MIDI and controller operation on supported Linux hardware

#### Scenario: macOS with the full MK2 attached
- **WHEN** the system is started on macOS and a full-size MK2 HID interface is present
- **THEN** the system uses that interface for controller operation

### Requirement: Safety and secondary behavior
The system SHALL run state and routing tests on Linux and macOS without an attached MK2 when started with a mock hardware adapter.

#### Scenario: Secondary operation
- **WHEN** the system is started with a mock hardware adapter
- **THEN** the system runs state and routing tests on Linux and macOS without an attached MK2

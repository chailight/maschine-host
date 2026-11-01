# Configuration And Testing Specification

## ADDED Requirements

### Requirement: Data-driven pages
The system SHALL load categories, subpages, stock-learn assignments, and each group's pad type from configuration. A missing pad type or stock-learn assignment SHALL fail closed as unavailable or UNMAPPED. The Bitbox file SHALL keep Groups A–H on channels 1–8, notes 36–51, and the current velocity curves. The panel proof file SHALL stay without controller numbers.

#### Scenario: Override performance profile
- **WHEN** a granular group assigns encoder 8 to pan random, controller 83
- **THEN** the host sends controller 83 for that encoder and leaves the other granular encoders unchanged

### Requirement: Test without Bitbox hardware
Tests SHALL drive controller events and capture MIDI and the display model. They SHALL check category labels, text values, shared parameter values, unmapped stock controls, and note routing.

#### Scenario: Regression of page selection
- **WHEN** a test plays a note, changes category, then releases the pad
- **THEN** the note-off uses the original note-on destination and the selected group is otherwise unchanged

#### Scenario: Button label and on/off text
- **WHEN** a test selects REVERB and moves an on/off encoder across 64
- **THEN** the display model shows REVERB on that button's column and shows On or Off for the encoder value

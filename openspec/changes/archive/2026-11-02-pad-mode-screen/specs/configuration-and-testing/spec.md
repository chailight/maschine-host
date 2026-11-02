## MODIFIED Requirements

### Requirement: Data-driven pages
The system SHALL load categories, subpages, stock-learn assignments, and each group's pad type from configuration. An omitted pad type SHALL load as sample. An unknown pad type SHALL stay unavailable. A missing stock-learn assignment SHALL fail closed as unavailable or UNMAPPED. The Bitbox file SHALL keep Groups A–H on channels 1–8, notes 36–51, and the current velocity curves. The panel proof file SHALL stay without controller numbers.

#### Scenario: Override performance profile
- **WHEN** a granular group assigns encoder 8 to pan random, controller 83
- **THEN** the host sends controller 83 for that encoder and leaves the other granular encoders unchanged

#### Scenario: Omitted pad type
- **WHEN** the Bitbox configuration is loaded and a group omits `pad_type`
- **THEN** that group's pad type is sample

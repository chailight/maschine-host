## ADDED Requirements

### Requirement: Pad press clears a pressure floor
A pad press SHALL be emitted only when both copies of that pad's pressure are strictly above a recorded floor. Pressure at or below the floor SHALL NOT emit a press, including when the other pads in the same report are above the floor. The floor SHALL be the smallest whole count that sits above every coupled reading in one captured hard hit and at or below a deliberate light hit on the struck pad. The capture SHALL include the struck pad and the pads that also sounded, which for a hit on pad 16 are pad 15, pad 12, and pad 8. The 36-report wait before a pad release SHALL stay in place for a pad that has already gone down. A rise during that wait SHALL remain the same hit. Velocity scaling and the velocity curve SHALL run only after a press is accepted. The pad report layout SHALL stay unchanged.

#### Scenario: A neighbor below the floor does not press
- **WHEN** one pad's pressure is above the floor on both copies and a neighboring pad's pressure on both copies is above zero and at or below the floor
- **THEN** the system emits a press for the struck pad only

#### Scenario: Pressure at the floor is not a hit
- **WHEN** both copies of a pad read exactly the floor and the previous report read zero
- **THEN** the system emits no press for that pad

#### Scenario: One copy above the floor is not a hit
- **WHEN** one copy of a pad is above the floor and the other copy is zero
- **THEN** the system emits no press for that pad

#### Scenario: A dip while the finger is down is the same hit
- **WHEN** a pad has pressed and its pressure returns to zero for fewer reports than the release wait, then rises above the floor again
- **THEN** the system emits no second press and no release for that pad

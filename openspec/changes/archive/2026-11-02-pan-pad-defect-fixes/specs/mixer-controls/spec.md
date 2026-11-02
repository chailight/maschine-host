## ADDED Requirements

### Requirement: Pan starts at centre
Each of the eight pan encoders SHALL start at 64. When the controller connects, the host SHALL send controller 38 with value 64 on channels 1 through 8. That connect SHALL NOT send level, filter, or pitch controllers. The preset template SHALL keep pan saved full left, so 64 remains the centre of the learned travel. A later turn of one pan encoder SHALL send controller 38 for that encoder's channel only, starting from 64.

#### Scenario: Connect centres every channel
- **WHEN** the controller connects and no pan encoder has been turned
- **THEN** channels 1 through 8 each receive controller 38 with value 64, and channels 1 through 8 receive no level, filter, or pitch controller from that connect

#### Scenario: The pan page shows centre before a turn
- **WHEN** the pan page is shown and no pan encoder has been turned
- **THEN** each of the eight pan values is 64

#### Scenario: One step leaves centre on that channel
- **WHEN** pan encoder 3 turns one step upward from its starting value
- **THEN** channel 3 receives controller 38 with value 65 and no other channel receives a pan controller

# Effects Controls Specification

## ADDED Requirements

### Requirement: Global effects pages
DELAY, REVERB, CHAR, DYNAM, and EQ SHALL use the verified controllers in the parameter registry. DELAY's fixed row SHALL be beat-sync, ping-pong, filter enable, and filter width on channel 1. Its delay time, delay feedback, and delay cutoff encoders SHALL use controllers 31, 30, and 32 on channel 1, the same time and feedback parameters as PERFORM. REVERB decay, damping, and pre-delay SHALL use controllers 33, 34, and 35 on channel 1. CHAR amount and tone SHALL be controllers 14–21 on channel 1. The algorithm subpage SHALL be the four pick controllers 22–25, not a second set of amounts. Each pick value SHALL show that bank's effect name in four bands of 32: 0–31, 32–63, 64–95, and 96–127. The line above each Amount and Type encoder SHALL name its fixed slot: Destruct, Overdrive, Colour, or Saturation. Amount labels SHALL read Amount and pick labels SHALL read Type. Each tone encoder SHALL use those two rows for the selected effect's tone description. Two lines fill both rows. One line uses only the upper row. The tone value stays on the bottom line. Destruction shows Pockey, DeRez, Ring Mod, and Fracture. Overdrive shows Tube, Drive, Mackity, and Huge. Colour shows Channel9, IronOxide, ToTape, and Coils. Saturation shows Purest, Spiral, Density, and Mojo. A pick value of 0 SHALL show the first effect. Amount at 0 is what removes the slot.

#### Scenario: Delay fixed row
- **WHEN** DELAY is active and the first encoder turns
- **THEN** the host sends controller 108 on channel 1

#### Scenario: Reverb decay
- **WHEN** REVERB is active and the decay encoder turns
- **THEN** the host sends controller 33 on channel 1

#### Scenario: Destruction pick names
- **WHEN** the destruction pick value is 0, and then 32, 64, and 96
- **THEN** the column shows Pockey, then DeRez, Ring Mod, and Fracture

### Requirement: Character tone reminder
The Type subpage's right panel SHALL keep the category names on its top line. Each column SHALL show the current effect on the same row as that slot's name on the left panel. The tone description SHALL sit at the bottom of the column. Two lines SHALL use the value row and the row just above it, spaced like the slot name and its parameter name. One line SHALL use only that upper row. Coils SHALL describe its tone as Quality. The Amount subpage SHALL keep the tone encoders on its right panel.

#### Scenario: DeRez tone reminder
- **WHEN** the Type subpage is shown and Destruction is set to DeRez
- **THEN** the right panel's first column shows DeRez on the slot-name row and harshness on the upper tone row, and the category name remains on the top line

#### Scenario: Coils tone reminder
- **WHEN** the Type subpage is shown and Colour is set to Coils
- **THEN** the column shows Coils and Quality

#### Scenario: Amount page labels the tone control
- **WHEN** the Amount subpage is shown and Destruction is Pockey
- **THEN** the right panel's first column shows sample and rate above the tone value

#### Scenario: One-line tone label
- **WHEN** the Amount subpage is shown and Saturation is Purest
- **THEN** the tone column shows drive on the upper row and the value on the bottom line

### Requirement: Channel9 console names
While Colour is set to Channel9, its tone value SHALL name the console instead of showing the controller number. The bands are 0–25 Neve, 26–50 API, 51–76 SSL, 77–101 Teac, and 102–127 Mackie. Any other Colour effect SHALL keep the tone number.

#### Scenario: Default Channel9 tone
- **WHEN** Colour is Channel9 and its tone is 64
- **THEN** the tone column shows SSL

#### Scenario: Another colour effect
- **WHEN** Colour is changed to IronOxide
- **THEN** the tone column shows the stored number

### Requirement: Mode-dependent compressor and EQ labels
DYNAM SHALL label controllers 41–45 from the host-commanded compressor type. Stock uses threshold, ratio, attack, release, and makeup. Logical4 uses threshold, ratio, speed, dry/wet, and makeup. VariMu uses intensity, no second control, speed, dry/wet, and output. Thunder uses thunder and output, and the three encoders between them are disabled. Disabled encoders SHALL send nothing. EQ mode, controller 50, SHALL show EQ and Type above the mode name. The ten names are DJ Filter, Dual Cut, Band Pass, Mixer, Tone+Filt, Tilt, Notch, Peak, Phaser, and Formant, in bands of 13 starting at 0. Play encoders 51–53 SHALL take the names of that mode. Tilt's fourth encoder SHALL show N/A and send nothing. Pushes 54–56 SHALL be encoders 5–7 on the same page, named for that mode's push, and SHALL send a hold at 64 or above. A push named Reset SHALL also store the matching play controller, 51, 52, or 53, at its centre value of 64 and send that controller. Its value row SHALL stay blank while the encoder is at rest, and SHALL show a filled square for a quarter of a second after a turn in either direction. Kill, Open, and Slam SHALL leave the play controller where it is. Tilt's third push SHALL show N/A and send nothing. Bypass 57 SHALL be encoder 8. Slope 59 SHALL be the first encoder of the next EQ subpage. That subpage SHALL also offer a host-generated LFO on encoders 2–5: speed in hertz from stopped at 0 through 0.1–10 Hz, depth as the peak swing of the destination controller, shape as Sine, Triangle, Square, or Saw, and a destination. The destination SHALL default to None. Its other choices SHALL be the current mode's three play controls, by the names shown for those controls. A choice with no control, such as Tilt's fourth encoder, SHALL show N/A and send nothing. While a destination is selected, the host SHALL move that play controller around its stored value and SHALL NOT change the stored value. Selecting None SHALL send the stored value once. The LFO SHALL NOT send controllers 91–93 and SHALL NOT follow a tempo.

#### Scenario: Compressor type changes
- **WHEN** the host-commanded compressor type changes from Stock to Thunder
- **THEN** the first parameter encoder is labelled Thunder, the output encoder stays active, and the three encoders between them send nothing

#### Scenario: DJ filter to mixer EQ
- **WHEN** the EQ mode changes from its starting value 6 to 45
- **THEN** the mode column shows Mixer, the three play encoders show high, mid, and low, and they stay on controllers 51, 52, and 53

#### Scenario: Tilt has no fourth control
- **WHEN** the EQ mode is Tilt
- **THEN** the fourth encoder and its push show N/A and send nothing

#### Scenario: Reset push centres the play control
- **WHEN** DJ Filter's filter is away from 64 and its reset push is turned
- **THEN** the host stores 64 for controller 51 and sends controller 51 with value 64, and the reset column's value row shows a filled square

#### Scenario: Reset column rests blank
- **WHEN** the reset encoder has not been turned for a quarter of a second
- **THEN** its value row is blank

#### Scenario: Host LFO follows the current EQ names
- **WHEN** the EQ mode is Phaser and the LFO destination is moved from None to the first play control
- **THEN** the destination column shows Sweep, and the host moves controller 51 around its stored value without changing that stored value

#### Scenario: Kill push leaves the play control
- **WHEN** Mixer EQ's high control is away from 64 and its kill push is turned
- **THEN** controller 51 stays at that value

### Requirement: Delay time sync mode
The host SHALL use one delay-time controller, 31 on channel 1, shared with PERFORM. Beat-sync, controller 108, SHALL NOT select a different time controller.

#### Scenario: One time control
- **WHEN** beat-sync is On and the delay-time encoder turns
- **THEN** the host sends controller 31 on channel 1

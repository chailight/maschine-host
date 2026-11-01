# Parameter Registry Specification

## ADDED Requirements

### Requirement: Typed parameter registry
The system SHALL define each controllable parameter once, with an ID, label, source, scope, address, range, supported pad types, value semantics, and display format. Scope SHALL be global, selected-slot, or all-slot. Source SHALL be patch-fixed or stock-learn. Two pages that use the same parameter SHALL share one host-commanded value. The host SHALL address that value by the parameter and, for a slot scope, the slot. It SHALL NOT keep a separate value per page position.

#### Scenario: Shared parameter on two pages
- **WHEN** the selected slot's delay send is changed on PERFORM
- **THEN** MIX delay send for that same slot shows the same host-commanded value

### Requirement: Verified MIDI mappings
The system SHALL send a patch-fixed controller only when this requirement names it. For a Bitbox parameter that is not patch-fixed, the host MAY propose a controller number and channel. The Bitbox learns that number; the host does not perform MIDI Learn. A proposed controller SHALL NOT reuse a controller already used on that same channel by a patch-fixed control or by another proposed control. Patch-fixed controllers SHALL be 7-bit. The host SHALL NOT assign controllers 120 or 123 to an encoder.

Published reference: the community Bitbox Micro patch for stock firmware 2.3.4, [CC reference](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/CC-REFERENCE.md). The installed module was not read.

Patch-fixed controllers, per slot, on that slot's MIDI channel:

- 84 delay send, 85 reverb send, 87 sustain, and 89 loop crossfade are continuous.
- 79 density, 80 grain size, 81 window, 82 scatter, and 83 pan random are continuous and granular only.
- 86 reverse and 90 play through are on at 64 and above. Reverse is sample, granular, and multi only. Play through is slicer only.
- 88 loop mode is three named bands and is not for clip or slicer.
- 91 LFO wave, 92 LFO beat sync, and 93 LFO retrigger follow the patch. Sync and retrigger are on at 64 and above.
- 114 manual tempo is continuous. 115 jungle cycle is continuous. 117 Live Flip is 25 bands. These are clip only.
- 78 baby scratch, 102 brake, 104 freeze, 105 pull up, 107 power down, 118 roll, and 119 pitch roll are clip holds: 64 and above is held, and below 64 is released.

Patch-fixed controllers on channel 1:

- Character amount 14–17, tone 18–21, and pick 22–25. Tone starts at 64. Amount starts at 0, and 0 removes the slot. Pick is four bands of 32, and 0 is the first effect. Destruction is Pockey, DeRez, Ring Mod, and Fracture. Overdrive is Tube, Drive, Mackity, and Huge. Colour is Channel9, IronOxide, ToTape, and Coils. Saturation is Purest, Spiral, Density, and Mojo. Channel9's tone names the console: 0–25 Neve, 26–50 API, 51–76 SSL, 77–101 Teac, 102–127 Mackie. Other tones stay numbers.
- Compressor enable 40, parameters 41–45, and type 47. Type is four bands: Stock, Logical4, VariMu, Thunder. Controllers 41–45 change meaning with the host-commanded type.
- EQ mode 50, play encoders 51–53, pushes 54–56, bypass 57, and slope 59. Mode starts at 6. Play encoders start at 64. Bypass, slope, and pushes are on at 64 and above. Mode is ten bands of 13: DJ Filter, Dual Cut, Band Pass, Mixer, Tone+Filt, Tilt, Notch, Peak, Phaser, and Formant. Tilt has no fourth encoder. Pushes 54–56 share the mode page. Slope is the next subpage, which also runs a host LFO in hertz against one of controllers 51–53. That LFO is not controllers 91–93.
- Delay beat-sync 108, ping-pong 109, filter enable 110, and filter width 111.

The host proposes these controllers for parameters the patch does not number. Global proposals use channel 1. A per-slot proposal uses the same controller on each slot channel. None of these numbers is used by a patch-fixed controller on channel 1.

- 30 delay feedback, 31 delay time, and 32 delay cutoff.
- 33 reverb decay, 34 reverb damping, and 35 reverb pre-delay.
- 36 level, 37 filter, 38 pan, and 39 pitch, one controller shared by slots 1–8 on channels 1–8.

Delay time is one controller. Beat-sync does not select a second time controller.

#### Scenario: Proposed feedback controller
- **WHEN** the user turns the global delay-feedback encoder
- **THEN** the host sends controller 30 on channel 1

#### Scenario: Master effect stays on channel 1
- **WHEN** the user turns the second Character amount encoder
- **THEN** the host sends controller 15 on channel 1

### Requirement: Text value format
The system SHALL show a parameter's display text on the bottom line of its LCD column when a format is configured. An on/off parameter SHALL show On at 64 and above and Off below 64. A parameter with no format SHALL show the stored number. The host SHALL NOT invent band edges. A named list SHALL show a name only when the manifest records the value range for that name.

#### Scenario: On and off
- **WHEN** a reverse encoder is at 64, and then moves below 64
- **THEN** the column shows On and then Off

### Requirement: State honesty
The system SHALL treat the displayed value as the last value the host commanded. The host SHALL NOT describe that value as the module's current value. The module does not return parameter values.

#### Scenario: External Bitbox adjustment
- **WHEN** a control is changed on the module and the host has not commanded it since
- **THEN** the host still shows its last commanded value and does not mark that value as read from the module

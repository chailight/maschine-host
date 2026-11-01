# Independent encoder pages

## Purpose

Loads eight encoder mappings for the selected page and keeps each value inside its configured range. Controller input comes from the full MK2 HID interface when it is present. Panel and LED writes are outside the behavior this capability fulfills.

## Requirements

### Requirement: Independent encoder pages primary behavior
The system SHALL load eight independent mappings and update the display when an encoder page is selected.

#### Scenario: Primary operation
- **WHEN** an encoder page is selected
- **THEN** the system loads eight independent mappings and updates the display

### Requirement: Safety and secondary behavior
The system SHALL emit only the configured action or explicit macro targets when an encoder changes, and SHALL preserve a bounded application-side value.

#### Scenario: Secondary operation
- **WHEN** an encoder changes
- **THEN** the system emits only the configured action or explicit macro targets and preserves a bounded application-side value

### Requirement: Bitbox Micro patch pages
The Bitbox configuration SHALL provide these encoder pages, in this order: `delay-send` 84, `reverb-send` 85, `delay-setup`, `character`, `character-pick`, `eq`, `eq-push`, `compressor`, `compressor-next`, `reverse` 86, `sustain` 87, `loop-mode` 88, `loop-crossfade` 89, `play-through` 90, `grain-density` 79, `grain-size` 80, `grain-window` 81, `grain-scatter` 82, `grain-pan` 83, `lfo-wave` 91, `lfo-sync` 92, `lfo-retrigger` 93, `baby-scratch` 78, `brake` 102, `freeze` 104, `pull-up` 105, `power-down` 107, `clip-method` 112, `clip-clock` 113, `clip-tempo` 114, `jungle-cycle` 115, `jungle-flavour` 116, `live-flip` 117, `roll` 118, and `pitch-roll` 119. A numbered page is per pad: it SHALL set that controller on encoders 1 through 8 and SHALL send encoder 1 on channel 1 through encoder 8 on channel 8. A master page SHALL send each of its controllers on channel 1. An unused encoder SHALL assign no controller. The panel proof SHALL NOT assign these controller numbers. The Bitbox file SHALL NOT assign controllers 120 or 123.

The master pages SHALL assign:

- `delay-setup`: controllers 108, 109, 110, and 111, then four unused encoders.
- `character`: controllers 14, 15, 16, 17, 18, 19, 20, and 21.
- `character-pick`: controllers 22, 23, 24, 25, 26, 27, 28, and 29.
- `eq`: controllers 50, 51, 52, 53, 57, and 59, then two unused encoders.
- `eq-push`: controllers 54, 55, and 56, then five unused encoders.
- `compressor`: controllers 40, 41, 42, 43, 44, 45, 46, and 47.
- `compressor-next`: controller 48, then seven unused encoders.

Starting values SHALL be 0 except where a centre or a glue setting is required. Character controllers 18 through 21 and EQ controllers 51 through 53 SHALL start at 64. EQ controller 50 SHALL start at 6. Compressor controller 41 SHALL start at 90, 42 at 7, 43 at 38, 44 at 37, and 45 at 64.

#### Scenario: Delay send
- **WHEN** the Bitbox configuration is loaded and the first encoder is turned on `delay-send`
- **THEN** a controller 84 message is sent on channel 1

#### Scenario: Reverb send follows the pad
- **WHEN** the Bitbox configuration is loaded and the second encoder is turned on `reverb-send`
- **THEN** a controller 85 message is sent on channel 2

#### Scenario: Character amount stays on channel 1
- **WHEN** the Bitbox configuration is loaded and the second encoder is turned on `character`
- **THEN** a controller 15 message is sent on channel 1

#### Scenario: Compressor glue default
- **WHEN** the Bitbox configuration is loaded
- **THEN** the compressor threshold starts at 90 and the compressor switch starts at 0

#### Scenario: Page order
- **WHEN** the Bitbox configuration is loaded
- **THEN** the pages begin `delay-send`, `reverb-send`, `delay-setup`, `character`, `character-pick`, `eq`, `eq-push`, `compressor`, `compressor-next`, and `reverse`

#### Scenario: Stop is not an encoder page
- **WHEN** the Bitbox configuration is loaded
- **THEN** no encoder is assigned controller 120 or controller 123

#### Scenario: Panel proof stays unassigned
- **WHEN** the panel proof configuration is loaded
- **THEN** its encoders assign no controller number

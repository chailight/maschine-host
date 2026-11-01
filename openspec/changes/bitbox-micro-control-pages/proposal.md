# Change: Bitbox Micro performance and sound-design control pages

## Why
The existing Maschine MK2 host already uses Group A-H to select the playable instrument. Add a Bitbox Micro control surface that makes a selected slot's delay and reverb sends available without page switching, and exposes the patched firmware's fixed controllers without pretending stock MIDI Learn numbers are known.

## Scope
- Eight direct-access top-level page categories via buttons above the displays: PERFORM, MIX, DELAY, REVERB, CHAR, DYNAM, EQ, PAD.
- Contextual subpages using existing page navigation; eight encoder mappings per page; dynamically labelled LCDs and meaningful LEDs.
- Stable selected instrument, explicit global/selected-slot/all-slot parameter scopes.
- A parameter registry distinguishing fixed patched CCs, preset-dependent stock MIDI Learn, and discrete actions.
- Bitbox-specific profiles for sample, granular, clip and slicer modes; safe momentary gestures.
- Configurable MIDI channel routing and mock-backend tests.

## Non-goals
Changing Group A-H instrument selection; implementing a sampler; machine listening; touchscreen UI; automatic reading of Bitbox's actual parameter values; promising control of stock parameters without MIDI Learn setup.

## Outcomes
The performer can play a selected group while controlling that slot's delay and reverb sends, switch to an eight-slot mixer, or enter sound-design pages without losing instrument selection. Global delay feedback appears only after its stock-learn controller is configured.

## Prerequisites / validation
The published community patch for stock 2.3.4 is the source for patch-fixed controllers: https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/CC-REFERENCE.md. The installed module was not read. Delay time, delay feedback, reverb parameters, and per-slot level, filter, pan, and pitch are stock MIDI Learn, not fixed controllers. The host stores the last command it sent and does not read the module.

## Conflicts with the current host
- Groups A–H stay the playable instruments. Category buttons must not select a group, and Pad Mode must not open the PAD category.
- The eight buttons above the displays currently toggle lamps and select nothing. They become one category radio.
- Left and Right currently wrap the flat 35-page list. They wrap the active category's subpages instead. The main encoder-pages, controller-input, and display-feedback specs still describe the flat list, numeric values, and the old page-name line. Those three need delta specs before implementation.
- Encoder state is stored per page position. A parameter shown on two pages needs one shared value.
- The LCD draws a number. On/off values need the words On and Off, and each column needs its category name under the button.

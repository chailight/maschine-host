## Why

The Bitbox file still sends placeholder controllers 20, 21, and 22. The community Bitbox Micro patch already publishes a fixed controller for every pad control and for the master Character chain, EQ, and compressor. The host can reach those by giving each one an encoder page.

## What Changes

- **BREAKING:** The Bitbox delay, reverb, and filter pages no longer send controllers 20, 21, and 22. Those numbers belong to the Character chain. Delay send becomes controller 84 and reverb send becomes controller 85, one encoder per pad channel.
- The Bitbox file gains a page for every other per-pad control in the patch: reverse, sustain, loop, play through, the five granular controls, the three LFO controls, and the Clip-pad modes and hold buttons.
- The file gains master pages, all on channel 1: the shared delay setup, the four Character slots, the EQ and its pushes, and the compressor.
- Groups, pad notes, velocity curves, and the MIDI port stay as they are. The panel proof file stays without controller numbers.
- Pad stop, controllers 120 and 123, is not given a page. The host already sends all-notes-off from panic.

## Capabilities

### New Capabilities

### Modified Capabilities

- `encoder-pages`: The Bitbox configuration's encoder pages follow the community patch controller map instead of the placeholder delay, reverb, and filter controllers.

## Impact

- `docs/example-config.yaml` encoder pages, and the test that checks those pages.
- No new MIDI message format. Each encoder still sends the controller and channel stored for that page.
- Source for the numbers: the [Bitbox Micro firmware patch](https://github.com/modularpatcher/bitbox-micro-fw-patch), in particular its [CC reference](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/CC-REFERENCE.md), [pad controls](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/PAD-CONTROLS.md), [Clip modes](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/CLIP-MODES.md), [devices](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/DEVICES.md), [Character chain](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/CHARACTER.md), [EQ and filter](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/EQ-FILTER.md), and [compressor](https://github.com/modularpatcher/bitbox-micro-fw-patch/blob/main/COMPRESSOR.md).

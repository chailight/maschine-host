# Hardware smoke checklist

Identity was confirmed on this Mac on 2026-10-26: the attached controller is `17cc:1140`. The remaining items were not run. There is still no Raspberry Pi, and no pad, encoder, LED, or LCD report was transferred. Do not mark those from the mock tests.

- [ ] Pi ARM64: read all 16 pads with velocity, groups A-H, and the eight screen encoders.
- [ ] Pi ARM64, no desktop: light the 16 RGB pads, the eight group LEDs, and both 256x64 panels.
- [x] Confirm the USB product is `1140` and not the Mikro id `1200`. Confirmed 2026-10-26 on macOS: Maschine Controller MK2, serial `4AF076D5`.
- [ ] macOS, with NIHardwareAgent and NIHostIntegrationAgent still running, then with them quit.
- [ ] Page-left and page-right silkscreen match the logical page controls.
- [ ] Disconnect and replug returns LEDs and both LCDs without stuck notes.
- [ ] External MIDI note-off survives rapid group changes on a real port.
- [ ] Local WAV one-shot and chromatic playback on the chosen Pi and Mac outputs: polyphony, clipping, and device unplug.
- [ ] Read the Bitbox enhanced-firmware CC map before replacing null controllers.

## Context

See proposal.md for why. Pan encoders in `docs/example-config.yaml` store `default: 0`, and `Controller::new` copies that into the shared slot value. `connect` returns no messages, and the live run never calls it: a successful HID open leaves the controller in the connected state it was constructed with, so controller 38 is not sent until a pan knob moves. Pad pressure in report `0x20` is a 12-bit count, repeated 32 bytes later. A press is emitted when both copies are above zero, and any positive count scales to a velocity of at least 1. The 36-report release wait applies only after that pad has gone down.

## Goals / Non-Goals

**Goals:**

- The eight pan values and the eight connect messages come from the same stored default.
- A count at or below the floor is treated as the pad being up, for both the press and the release wait.
- The floor number is taken from one capture, not chosen in advance.

**Non-Goals:**

- Sending level, filter, pitch, or any other default on connect.
- Changing `panpos` in the host template, the velocity curve, or the pad byte map.
- Folding this into `pad-mode-config` or the Maschine project converter.

## Decisions

1. **The config default is the centre value.** Set the eight `mix-pan` encoders to `default: 64`. Each is `parameter: pan` with slot scope, so the shared value for that slot is 64 and the pan page shows 64 before a turn. One step upward sends 65 on that encoder's channel. Alternative: hardcode 64 in the connect send and leave the config at 0. Rejected because the page and the first message would start from different values.

2. **Connect sends only those pan controllers, and the live run has to call it.** `connect` returns controller 38 at the stored pan value on each configured pan encoder's own channel, channels 1–8 in the example file, and no other controllers. `voiced` leaves 38 unchanged because the LFO target is an EQ play controller. The run loop applies that list once MIDI is open and the HID open succeeded. A refused HID read still disconnects and sends no pan. Alternative: send every encoder default on startup. Rejected because that would move level, filter, and the effect defaults the module already has. Alternative: send from `Controller::new`. Rejected because construction has no MIDI port.

3. **The floor gates both copies before a pad is down.** Replace "either copy at zero means up" with "either copy at or below the floor means up". Velocity uses the primary count only after both copies are above the floor, so a hit just above the floor keeps its real velocity. The release wait then sees the same gated pressure: a dip to the floor starts the wait, and a rise above the floor during the wait is still the same hit. Alternative: drop the extra note after it has been emitted. Rejected because the neighbor would already have sounded. Alternative: raise the floor after scaling to 1–127. Rejected because a count of 1 and a count of 32 both become velocity 1, so the floor would no longer see the coupling.

4. **The capture picks the constant.** One `--capture-hid` take records a hard hit on the pad the operator calls pad 16, and a separate light hit on that same pad. The assignment was pressed from the bottom-left, so host `pad:0` is the bottom-left and host `pad:15` is the top-right. Pad 16, 15, 12, and 8 in the report are host pads 15, 14, 11, and 7. Read the 12-bit count on both copies of each. The constant is the smallest whole number above every coupled reading and at or below the light hit. Record that number and the readings beside `PAD_RELEASE_REPORTS`, and note them in `docs/hid-input.md`. If the light hit is not above the coupled readings, stop without guessing a floor.

## Risks / Trade-offs

- [A reconnect recentres pan the operator had moved] → Accepted. Each connect is a return to centre. Rollback is to restore `default: 0` and the empty connect.
- [The floor sits above a wanted light hit, or below the coupling] → The capture rule forbids shipping a number that fails either side. The tests use a stand-in floor so they do not depend on the hardware count.
- [The corner the operator means is not host pad 15] → The capture is what names the indexes. The constant applies to every pad, so a wrong guess about which corner was hit does not change the gate.

## Migration Plan

No stored data moves. Deploy by running the host with the updated example config. Rollback is reverting the pan defaults, the connect messages, and the floor constant.

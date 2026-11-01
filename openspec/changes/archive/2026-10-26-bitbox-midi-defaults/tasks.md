## 1. Omitted encoder channel

- [x] 1.1 When an encoder MIDI CC mapping omits its channel, use the channel of the group at the same index. An explicit channel replaces that default.
- [x] 1.2 Test that an omitted channel follows the group and an explicit channel does not.

## 2. Bitbox defaults

- [x] 2.1 In `docs/example-config.yaml`, give every group notes 36 through 51 and keep Groups A through H on channels 1 through 8.
- [x] 2.2 In that file, set delay to controller 20, reverb to controller 21, and filter to controller 22. Set each slot's channel to its group's channel.
- [x] 2.3 Leave `docs/encoder-panel-example.yaml` without controller numbers.

## 3. Sent messages

- [x] 3.1 Test that a Bitbox pad press sends that pad's shared note on the selected group's channel.
- [x] 3.2 Test that turning a Bitbox encoder sends the page's controller number on that slot's channel.
- [x] 3.3 Test that the published Bitbox file and the panel proof validate, and that the proof assigns no controller number.

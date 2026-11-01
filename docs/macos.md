# macOS permissions and Native Instruments services

Observed 2026-10-26 on macOS 15.7.7 with a Maschine Controller MK2 connected (`17cc:1140`, serial `4AF076D5`):

- `NIHardwareAgent` has an `IOHIDLibUserClient` on the MK2 HID interface.
- `logioptionsplus_agent` also has an `IOHIDLibUserClient` on that same HID interface. The interface's exclusive owner is Apple's `AppleUserUSBHostHIDDevice`.
- `NIHostIntegrationAgent` and `NTKDaemon` were running and did not own an MK2 interface.
- `MIDIServer` owns the class-compliant MIDI interface through `com.apple.AppleMIDIUSBDriver`.

Encdr's usage notes say the NI services can hold the USB interfaces on macOS, and that they may have to be quit before another program can claim the device. This host does not claim the device. Quitting the services is not required to run the mock or to send MIDI to a named port.

The live CoreMIDI name "Maschine Controller MK2" is the class-compliant USB MIDI port. An offline device of the same name can remain after an earlier connection. Virtual ports named "Maschine MK2 Virtual Input" and "Maschine MK2 Virtual Output" are separate and have no USB driver owner. None of those ports is the pad path, and this host does not send LED feedback through them.

When a later adapter does open the MK2 HID interface, expect a conflict while `NIHardwareAgent` holds its client. Unplug and replug after that agent exits, and confirm the device is `17cc:1140` before trusting it. Product `1200` is the Mikro MK2 in the Encdr descriptor set. The DFU interface on the full MK2 is present and was not entered.

CoreMIDI permission prompts, if the OS shows one, belong to the terminal or app that opened the port. This repository does not ship an entitlement or a signed bundle.

The development machine also has an iConnectMIDI4+. Listing ports is safe. Sending notes is not, until the config names the intended output.

# LCD panels

The running host sends LCD reports because `docs/panel-map.yaml` records a matched check from 2026-10-26. Lamp reports do not contain panel bytes. No HID feature report is sent. `NIHardwareAgent` is left running.

Each panel is 256×64, divided into four columns of 64 pixels so each encoder sits above its knob. The page name is one line across the top. In each column the configured label sits above the value stored for the active page, and the value is on the bottom line. The left panel is encoders 1–4 and the right panel is encoders 5–8. The host draws those words in the 5×7 face. Turning a knob changes only that page. The Left and Right buttons beside the displays move one page and wrap, and each lamp is on only while the button is held.

`docs/example-config.yaml` is the Bitbox config. It keeps the delay, reverb, and filter pages, and it assigns no encoder action. `docs/encoder-panel-example.yaml` is the panel proof and the long-term example: two pages labeled `enc 1` through `enc 8`, also with no encoder action. Real encoder actions wait for a later change.

## Check

The first check sent one 2049-byte report. macOS accepted the write and the left panel did not change. The next check sent eight strips whose width field was 256. That reached the left panel and cut blank lines through the picture. A captured frame for this controller uses a row width of 32 bytes and eight rows of pixels. The check command still paints those strips on the left panel only, waits, then clears that panel. It does not write the right panel or the lamps, and it does not change the stored map.

```sh
cargo run -- --panel-check
```

That left panel matched on 2026-10-26: the page name ran across the top, and enc 1 through enc 4 sat in the four columns above the knobs. The right panel and the lamps were left unchanged, and `docs/panel-map.yaml` records the match. The right panel uses the same strips with the other report id. A normal host run is what shows it.

`--capture-hid` and `--listen` do not send output or feature reports. `--self-check` uses the mock adapter and does not open the HID interface.

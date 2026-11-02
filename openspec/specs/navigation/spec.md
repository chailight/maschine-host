# Navigation Specification

## Purpose

Keeps Groups A through H as the playable instruments while the buttons above the displays choose a control category.

## Requirements

### Requirement: Group selection remains instrument selection
The system SHALL keep Groups A–H as the playable instrument selection. Selecting a group SHALL change the notes the 16 pads send and SHALL leave the active control category and its subpage unchanged. A pad press SHALL send that group's note. The PAD category and the silk-screen Pad Mode button SHALL remain different controls. Pad Mode SHALL NOT open the PAD category, and the PAD category SHALL NOT change pad mode, velocity curve, note scale, or pad type. Pressing Pad Mode SHALL toggle the setup screen. The first press engages it and the next press leaves it. Engaging or leaving it SHALL leave the active category and its subpage unchanged. Selecting a group while the setup screen is engaged SHALL leave that screen engaged and SHALL show the newly selected group's velocity curve, note scale, root note, octave offset, and pad type.

#### Scenario: Change group while editing effects
- **WHEN** a user selects Group B while DELAY is active and the setup screen is off
- **THEN** the playable instrument changes to B, DELAY remains the active category, and a pad press sends Group B's note

#### Scenario: Pad Mode is not the PAD category
- **WHEN** the user presses the silk-screen Pad Mode button
- **THEN** the setup screen becomes engaged, the active control category is unchanged, the PAD category is not selected, and the pads still send the selected group's notes

#### Scenario: Pad Mode press leaves the setup screen
- **WHEN** the setup screen is engaged and the user presses Pad Mode again
- **THEN** the setup screen is off and the panels show the category and subpage that were selected before the first press

#### Scenario: Change group on the setup screen
- **WHEN** the setup screen is engaged and the user selects Group B
- **THEN** the setup screen stays engaged, it shows Group B's velocity curve, note scale, root note, octave offset, and pad type, and the active category is unchanged

### Requirement: Category buttons and their labels
The eight buttons above the displays, from left to right, SHALL select PERFORM, MIX, DELAY, REVERB, CHAR, DYNAM, EQ, and PAD. These eight buttons SHALL act as one radio: the selected category's button is lit and the other seven are not. They SHALL NOT join the Volume/Swing/Tempo/Note Repeat radio or the Scene/Pattern radio. While the Pad Mode setup screen is off, each LCD column SHALL show that button's category name on the top line, under the button. While the setup screen is engaged, those category names SHALL NOT be drawn. Selecting a category SHALL NOT emit a note and SHALL NOT change the selected group. Selecting a category while the setup screen is engaged SHALL remember that category and SHALL leave the setup screen engaged.

#### Scenario: Category label under the button
- **WHEN** MIX is selected and the Pad Mode setup screen is off
- **THEN** the second column shows MIX on its top line, the MIX button is lit, the other seven category buttons are not lit, and the selected group is unchanged

#### Scenario: Navigate without interrupting performance
- **WHEN** the user selects MIX and a pad is held
- **THEN** no new MIDI note is emitted and the held pad still belongs to the group that started the note

#### Scenario: Category press keeps the setup screen
- **WHEN** the setup screen is engaged and the user selects MIX
- **THEN** the setup screen stays engaged, MIX is the remembered category, and the category names are not drawn

### Requirement: Subpages stay inside the category
The Left and Right buttons beside the displays SHALL move among the active category's subpages only and SHALL wrap inside that category. The host SHALL remember the last subpage of each category. When the active category has more than one subpage, the Left lamp SHALL be on unless the display is on that category's first subpage, and the Right lamp SHALL be on unless the display is on that category's last subpage. A category with one subpage SHALL light neither arrow. Releasing either button SHALL leave those lamps as the page position requires. While the Pad Mode setup screen is engaged, Left and Right SHALL NOT move the subpage, and the arrow lamps SHALL stay as the remembered subpage requires. The panel proof configuration SHALL keep its existing unlabeled pages and SHALL NOT gain these categories. Pages without categories SHALL keep lighting Left and Right only while held.

#### Scenario: Return to the last mixer subpage
- **WHEN** the user leaves MIX on REVERB SEND, selects DELAY, then selects MIX again
- **THEN** MIX shows REVERB SEND and the selected group is unchanged

#### Scenario: First mixer subpage lights only Right
- **WHEN** MIX is showing its first subpage
- **THEN** the Left lamp is off and the Right lamp is on

#### Scenario: Middle mixer subpage lights both arrows
- **WHEN** MIX is showing a middle subpage
- **THEN** both arrow lamps are on

#### Scenario: Last mixer subpage lights only Left
- **WHEN** MIX is showing its last subpage
- **THEN** the Left lamp is on and the Right lamp is off

#### Scenario: Single subpage lights neither arrow
- **WHEN** DELAY, which has one subpage, is selected
- **THEN** both arrow lamps are off

#### Scenario: Arrows do not page the setup screen
- **WHEN** MIX is on a middle subpage, the setup screen is engaged, and the user presses Right
- **THEN** the remembered MIX subpage is unchanged and both arrow lamps stay on

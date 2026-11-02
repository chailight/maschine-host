## MODIFIED Requirements

### Requirement: Encoder page buttons
The Left and Right buttons beside the displays SHALL select the encoder page. When the loaded pages do not declare categories, a press of the Right button SHALL select the next page and a press of the Left button SHALL select the previous page, wrapping the full list, and each of those buttons SHALL light only while it is held. When the pages declare categories, Left and Right SHALL wrap inside the active category and the host SHALL remember that category's page. The Left lamp SHALL be on when the active category has an earlier subpage, and the Right lamp SHALL be on when it has a later subpage. A category with one subpage SHALL light neither arrow. Releasing either button SHALL leave those lamps as the page position requires. The eight buttons above the displays SHALL select the eight categories when categories are declared, as one radio, and SHALL NOT change the selected group.

#### Scenario: Next page
- **WHEN** the Right button beside the displays is pressed, a later encoder page exists, and the pages do not declare categories
- **THEN** the next encoder page is selected

#### Scenario: Previous page wraps
- **WHEN** the first encoder page is selected, the pages do not declare categories, and the Left button beside the displays is pressed
- **THEN** the last encoder page is selected

#### Scenario: Page button wraps inside the category
- **WHEN** the Bitbox configuration is loaded and the Right button is pressed on the last MIX subpage
- **THEN** MIX shows its first subpage and the selected group is unchanged

#### Scenario: Page button lamp
- **WHEN** the Left or Right button beside the displays is held, and then released, and the pages do not declare categories
- **THEN** that button's lamp is on while it is held and off after release

#### Scenario: Category arrow shows another page
- **WHEN** the Bitbox configuration is loaded and MIX is on its first subpage
- **THEN** the Right lamp is on and the Left lamp is off, and releasing Right leaves the Right lamp on

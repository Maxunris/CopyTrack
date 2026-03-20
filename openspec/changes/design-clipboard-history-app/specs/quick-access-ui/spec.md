## ADDED Requirements

### Requirement: System provides a global quick-access popup
The system SHALL expose a quick-access popup that can be opened through a global keyboard shortcut without focusing the main application window first.

#### Scenario: User presses the global shortcut
- **WHEN** the configured global shortcut is triggered
- **THEN** the system opens the quick-access popup above other windows with the most relevant recent history items

### Requirement: Quick popup supports keyboard-first interaction
The system SHALL allow users to navigate, search, and re-copy entries from the quick-access popup using keyboard controls alone.

#### Scenario: User navigates popup results with keyboard
- **WHEN** the quick-access popup is open and the user presses arrow keys and Enter
- **THEN** the system changes selection and re-copies the selected entry without requiring a mouse

### Requirement: System provides tray-based entry points
The system SHALL provide a system tray or menu bar presence with actions to open history, pause capture, access settings, and quit the application where the target platform supports it.

#### Scenario: User opens tray menu
- **WHEN** the user clicks the tray or menu bar icon
- **THEN** the system shows core actions including history, pause or resume capture, settings, and exit

### Requirement: System reflects capture state in quick surfaces
The system SHALL visibly indicate whether clipboard capture is active or paused in the popup and tray-accessible controls.

#### Scenario: Capture is paused from tray
- **WHEN** the user pauses capture from the tray menu
- **THEN** the tray and popup surfaces both show that the application is not recording new clipboard items

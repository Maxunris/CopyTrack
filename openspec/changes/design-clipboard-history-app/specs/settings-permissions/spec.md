## ADDED Requirements

### Requirement: User can configure retention and cleanup behavior
The system SHALL allow users to configure retention limits based on age, item count, and storage footprint, while protecting pinned items from automatic cleanup.

#### Scenario: Age-based retention expires old entries
- **WHEN** the configured retention period elapses for a non-pinned item
- **THEN** the system removes that entry during cleanup

### Requirement: User can configure startup and theme behavior
The system SHALL allow users to configure launch-at-login behavior, theme mode, quick-access shortcut, and default window behavior.

#### Scenario: User changes theme preference
- **WHEN** the user switches between light, dark, or system theme
- **THEN** the application updates its appearance to match the saved preference

### Requirement: System communicates required permissions
The system SHALL provide platform-specific guidance when clipboard access, accessibility access, notifications, or login-item permissions are needed for expected behavior.

#### Scenario: Required OS permission is missing
- **WHEN** the application detects that an operating system permission needed for clipboard capture or hotkeys is unavailable
- **THEN** the system explains the missing permission and shows the user how to enable it

### Requirement: User can define privacy controls
The system SHALL allow users to pause capture, exclude applications, redact previews for sensitive entries where supported, and clear stored history.

#### Scenario: User excludes a password manager
- **WHEN** the user adds a password manager application to the exclusion list
- **THEN** the system stops recording clipboard events originating from that application

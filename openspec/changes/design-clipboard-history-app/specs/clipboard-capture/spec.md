## ADDED Requirements

### Requirement: System captures supported clipboard content
The system SHALL monitor clipboard changes and create a history entry whenever supported content is copied, including plain text, rich text previews where available, URLs, code snippets treated as text, images, and file references.

#### Scenario: Text content is copied
- **WHEN** the user copies plain text in another application
- **THEN** the system creates a new history entry with the copied text, timestamp, detected type, and preview

#### Scenario: Image content is copied
- **WHEN** the user copies an image from another application
- **THEN** the system stores an image history entry with preview metadata and a retrievable image payload reference

#### Scenario: File reference is copied
- **WHEN** the user copies one or more files from the operating system file manager
- **THEN** the system stores a history entry representing the copied file references with file names and path metadata where permitted

### Requirement: System avoids duplicate noise
The system SHALL detect immediate duplicate clipboard entries and avoid creating redundant history items when the copied payload and type have not changed.

#### Scenario: Same text is copied twice without change
- **WHEN** the user copies identical text consecutively
- **THEN** the system does not create a second redundant history entry

### Requirement: System records clipboard metadata
The system SHALL store metadata for each history entry including creation time, content type, size estimate, source application if available, and security classification flags.

#### Scenario: Source application is available
- **WHEN** the operating system exposes the source application for a clipboard event
- **THEN** the system stores the source application name with the created history entry

### Requirement: System supports capture controls
The system SHALL allow clipboard monitoring to be paused and resumed without deleting existing history.

#### Scenario: Capture is paused
- **WHEN** the user disables clipboard capture
- **THEN** the system stops creating new history entries until capture is resumed

### Requirement: System respects excluded contexts
The system SHALL allow configured applications or contexts to be excluded from clipboard capture.

#### Scenario: Copy happens in an excluded application
- **WHEN** the user copies content from an excluded application
- **THEN** the system does not store that clipboard event in history

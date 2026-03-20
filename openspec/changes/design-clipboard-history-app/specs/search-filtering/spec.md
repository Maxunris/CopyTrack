## ADDED Requirements

### Requirement: User can search clipboard history
The system SHALL provide instant search across stored clipboard entries using normalized textual content and metadata.

#### Scenario: User searches for a remembered phrase
- **WHEN** the user enters a search query in the main window or quick popup
- **THEN** the system filters results to entries whose content or metadata match the query

### Requirement: User can filter by content attributes
The system SHALL allow filtering by content type, pinned state, favorite state, tag, date range, and source application where metadata exists.

#### Scenario: User filters only images
- **WHEN** the user applies the image content filter
- **THEN** the system shows only history entries stored as images

### Requirement: System supports sortable history views
The system SHALL provide sorting options for recency, manual importance markers, and other supported metadata-derived ordering modes.

#### Scenario: User sorts by pinned then recent
- **WHEN** the user changes the sort mode to prioritize pinned entries
- **THEN** the system updates the visible history ordering to match the selected rule

### Requirement: Search remains usable at larger history sizes
The system SHALL keep search and filtering responsive for the retention limits configured for the first supported release tier.

#### Scenario: User searches a large local history
- **WHEN** the retained history contains many stored entries within supported limits
- **THEN** the system returns filtered results without blocking the interface

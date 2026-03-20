## ADDED Requirements

### Requirement: System definition includes a stack-selection questionnaire
The product definition SHALL include a questionnaire that gathers the information needed to recommend frontend, backend, storage, packaging, and sync technologies.

#### Scenario: Technical planning starts
- **WHEN** the product definition phase is complete
- **THEN** the team has a structured questionnaire covering platform targets, UI shell, offline mode, data retention, sync, and expected scale

### Requirement: Questionnaire covers product and audience constraints
The questionnaire SHALL ask about primary users, supported platforms, release priority, monetization expectations, and privacy sensitivity.

#### Scenario: Product owner evaluates audience fit
- **WHEN** the questionnaire is presented
- **THEN** it includes prompts that clarify whether the product targets general users, developers, business teams, or another niche

### Requirement: Questionnaire covers technical architecture constraints
The questionnaire SHALL ask about desktop versus web delivery, local database durability, optional backend needs, import or export expectations, and sync requirements.

#### Scenario: Architecture decisions are deferred
- **WHEN** the final stack has not yet been chosen
- **THEN** the questionnaire captures the architectural constraints needed to choose an appropriate stack afterward

### Requirement: Questionnaire output informs the next design step
The system SHALL treat the completed questionnaire as the input for proposing a technology stack, explaining trade-offs, and drafting project structure.

#### Scenario: User answers the questionnaire
- **WHEN** the product owner provides answers
- **THEN** the next planning step produces a technology recommendation with rationale, pros and cons, and a proposed project structure

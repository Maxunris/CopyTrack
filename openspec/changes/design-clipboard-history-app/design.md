## Context

The product is a clipboard history application aimed at users who copy and reuse content frequently during work, study, and research. The current problem space includes accidental clipboard loss, poor visibility into previously copied data, slow reuse of older items, and a lack of organization for mixed content such as plain text, links, code snippets, images, and file references.

This change is intentionally product-led before implementation-led. User answers now confirm a macOS-first desktop utility with future cross-platform expansion, local-only storage in version one, editable global shortcut support, launch-at-login behavior, menu bar presence, and a broad audience that requires an intuitive interface.

Primary stakeholders:
- End users who want instant clipboard recall with minimal friction
- Product owner deciding scope and target audience
- Future implementation team choosing desktop shell, data layer, and OS integrations

Constraints:
- Clipboard access is platform-specific and permission-sensitive, especially on macOS and mobile operating systems
- The first release should remain local-first and responsive even with large histories
- Sensitive data handling must be visible and configurable
- The interface should feel like a polished desktop utility, not a generic admin panel
- Version one should include retention presets of 50, 100, 500, 1000, and 10000 items

## Goals / Non-Goals

**Goals:**
- Define a desktop-first clipboard product with clear core value: capture, find, preview, and reuse copied content quickly
- Describe a UI structure that supports both always-available quick access and richer long-form history management
- Model the system around multiple clipboard content types and metadata-rich records
- Establish requirements for search, tagging, retention, privacy, permissions, and future sync
- Produce an implementation-ready artifact set while preserving flexibility on the final stack
- Prepare a discovery questionnaire that will drive the later technology recommendation
- Optimize the first release for macOS utility ergonomics, including translucent surfaces, top-bar presence, and keyboard-first recall

**Non-Goals:**
- Defining cloud sync as mandatory for version one
- Solving mobile-first constraints in the initial release
- Designing a browser-based SaaS product as the primary experience

## Decisions

### Decision: The product is macOS-first, desktop-only, and local-first

The initial product direction targets macOS first because clipboard monitoring, global shortcuts, menu bar behavior, translucent window styling, and overall polish matter more than immediate multi-platform delivery. The product remains desktop-only, while future Windows and Linux support should be preserved in the architecture. A local-first model keeps the app fast, private by default, and usable offline.

Alternatives considered:
- Web-first application: rejected because web apps cannot provide reliable global clipboard history capture or system tray workflows
- Mobile-first application: rejected for the first phase because OS restrictions make passive clipboard history unreliable or impossible

### Decision: Split the experience into two surfaces

The UX should use a quick-access popup for speed and a full main window for deep management. The popup supports rapid recall through a global shortcut and recent-item list. The main window supports search, filtering, pinning, preview, and settings.

Alternatives considered:
- Single main window only: rejected because it slows down the primary recall flow
- Tray-only utility: rejected because richer history management and settings become cramped

### Decision: Use a unified clipboard item model with type-specific previews

All clipboard entries should share a common data model: unique ID, content type, created timestamp, source app if available, preview text, raw payload reference, size, favorite state, pinned state, tags, and security flags. Rendering and actions then branch by content type.

Alternatives considered:
- Separate storage flow per content type: rejected because it complicates search, filters, and future sync
- Text-only MVP model: rejected because the requested concept explicitly includes broader content support

### Decision: Privacy controls are first-class UX, not hidden settings

Users must be able to pause capture, exclude apps, configure retention, delete entries, and mark content as sensitive. The product should visibly communicate when capture is active and when content is intentionally not stored.

Alternatives considered:
- Silent capture with minimal controls: rejected because it creates trust and compliance risks
- Hard-coded exclusions only: rejected because user workflows vary too much

### Decision: Search and organization are core, not premium extras

Search, filters, tags, favorites, and pinning should be defined as baseline capabilities. Clipboard history becomes noisy quickly; without organization, capture alone has limited value.

Alternatives considered:
- Recent-items-only product: rejected because it does not solve long-tail recall
- Search without structure: rejected because tags and filters improve recall for large histories

### Decision: Sync is optional and layered behind a transport boundary

Import/export should exist in the requirements now, while multi-device sync remains optional and deferred. The architecture should treat sync as an extension of the local data model rather than a prerequisite for the core product.

Alternatives considered:
- Mandatory account-based sync from day one: rejected because it increases complexity, security burden, and time to first release
- No portability path at all: rejected because export and future sync affect the data model today

### Decision: Visual direction should feel like a premium macOS utility

The interface should borrow from native desktop idioms: compact command surfaces, high information density, calm neutral palette, translucent layered surfaces, and crisp typography. The app should feel closer to Raycast, CleanShot, or a well-made menu bar utility than to a dashboard.

Alternatives considered:
- Pure native OS clone: rejected because a consistent cross-platform brand may matter later
- Generic settings-heavy utility UI: rejected because it weakens product differentiation

## Risks / Trade-offs

- [Clipboard APIs differ significantly by OS] -> Mitigation: isolate clipboard watchers, pasteboard adapters, and permission checks behind platform services
- [Image and file payloads may inflate local storage] -> Mitigation: use retention limits, lazy previews, and configurable size caps
- [Sensitive data may be copied unintentionally] -> Mitigation: provide pause capture, exclusion rules, redaction options, and explicit privacy messaging
- [Search performance may degrade on large histories] -> Mitigation: store normalized searchable fields and plan for indexed local queries
- [Too many UI actions can overwhelm the quick popup] -> Mitigation: keep popup focused on recent recall and move advanced organization to the main window
- [Future sync can distort the local-first model] -> Mitigation: define sync as optional and additive, with conflict strategy designed after core local behavior is stable

## Migration Plan

This change is a greenfield product definition, so there is no production migration. The rollout path for implementation should be:

1. Confirm questionnaire answers and lock platform priorities
2. Select stack and packaging strategy
3. Implement local clipboard capture and storage
4. Ship quick popup plus main history window
5. Add search, filters, tags, and privacy controls
6. Add import/export
7. Evaluate optional sync after the local product is stable

Rollback strategy:
- If advanced features create too much scope, retain the local capture plus search-focused core and defer sync, tags, or image/file depth

## Open Questions

- Should version one persist full binary payloads for copied files where possible, or store file references plus metadata first and expand later?
- What exact default global shortcut should ship first on macOS before the user customizes it?
- Should the first public release include basic import or export, or fully defer portability until after the core experience is stable?

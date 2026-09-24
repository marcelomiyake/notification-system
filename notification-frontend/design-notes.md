# Operator console design notes

## Contents

- [Prototype reference](#prototype-reference)
- [Visual language](#visual-language)
- [Accessibility and interaction states](#accessibility-and-interaction-states)
- [Prototype interactions reviewed](#prototype-interactions-reviewed)
- [Product boundary](#product-boundary)

## Prototype reference

> Project documentation index: [Documentation index](../docs/README.md)

The operator console uses the dashboard/prototype workflow from [OpenDesign](https://github.com/nexu-io/open-design). The reviewed [OpenDesign prototype](http://127.0.0.1:17758/projects/93372b76-ddcc-44d0-aa3e-5bb6ddfed83f/conversations/f12440a0-b769-4a95-bd1e-cd2d835ad499/files/index.html) is a separate, self-contained design artifact (project `93372b76-ddcc-44d0-aa3e-5bb6ddfed83f`). The shipped interface is implemented in Vue/CSS; OpenDesign is not a runtime dependency.

The prototype uses a compact navigation rail, summary cards that pair each figure with context, a task-focused composer, and a separate recipient-control panel. Its nine activity records, counts, and delivery outcomes are synthetic fixtures; the channel breakdown is explicitly not queue depth. `sent` means simulated recording-adapter acceptance, not recipient delivery. The artifact makes no capacity claim and does not call the API or an external provider.

## Visual language

- Cool, low-saturation workspace canvas with white cards and one indigo action color.
- Manrope for headings and DM Sans for interface labels/body; system fallbacks keep the page legible without external fonts.
- Four channel glyph/color treatments are always paired with text labels; colors do not communicate channel or status alone.
- Dense operational details remain secondary to the primary compose action; environment status and synthetic-data reminder stay visible.
- Responsive layout collapses navigation and stacks panels for tablet/mobile widths.

## Accessibility and interaction states

- Semantic landmarks, headings, native controls, explicit labels, visible keyboard focus, and accessible action names.
- Preference switches expose the channel name and checked state; scheduling and templates are separate labelled controls.
- Loading, empty, error, success, disabled, validation, and submitting states have text or ARIA status semantics.
- The reviewed prototype provides a 3px `:focus-visible` ring and a `prefers-reduced-motion` rule that removes smooth scrolling and reduces animations and transitions. Native form labels, error descriptions, live feedback, and a semantic detail dialog with a delivery timeline are present.
- The OpenDesign preview was checked at desktop and mobile (390 × 844) sizes. Contrast was selected for labels and action text on their backgrounds; this is a design review, not a formal WCAG audit.
- Dates use local input and are converted to UTC RFC 3339 before submission.

## Prototype interactions reviewed

- Filter the local activity list by channel and status, open a notification detail timeline, or reset the synthetic fixtures.
- Compose iOS push, Android push, SMS, or email using a synthetic recipient and versioned template; preview interpolated variables and the matching request shape.
- Switch between immediate and scheduled requests, toggle a recipient preference, and demonstrate acceptance followed by preference suppression before any simulated adapter attempt.
- Demonstrate identical-key idempotent replay and changed-payload `409 Conflict`, plus successful recording acceptance, a transient error with bounded retry, and a permanent error routed to the channel dead-letter queue.

These interactions are local browser simulations. The Vue console's integration and Kind end-to-end behavior is tracked separately in [`docs/verification/`](../docs/verification/README.md).

## Product boundary

This is an operator-facing local reference UI, not an end-user subscription portal. API authorization remains authoritative. The browser carries only the local demo API key in the private Kind environment; do not reuse this pattern for production.

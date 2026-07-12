---
name: xberg-web-ui
description: WASM-powered document ingestion UI — local extract→OCR→NER→redact→embed, sync on demand
colors:
  primary: "#0f172a"
  primary-hover: "#1e293b"
  destructive: "#dc2626"
  destructive-hover: "#b91c1c"
  destructive-bg: "#fef2f2"
  destructive-text: "#b91c1c"
  warning: "#f59e0b"
  warning-bg: "#fffbeb"
  warning-text: "#b45309"
  success: "#16a34a"
  success-bg: "#f0fdf4"
  success-text: "#15803d"
  neutral-bg: "#ffffff"
  neutral-surface: "#f8fafc"
  neutral-border: "#e2e8f0"
  neutral-border-strong: "#cbd5e1"
  neutral-text: "#0f172a"
  neutral-text-muted: "#64748b"
  focus-ring: "#94a3b8"
typography:
  display:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "clamp(1.5rem, 3vw, 2rem)"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.02em"
  headline:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 600
    lineHeight: 1.3
  title:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "1rem"
    fontWeight: 500
    lineHeight: 1.4
  body:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 500
    lineHeight: 1.5
    letterSpacing: "0.02em"
    textTransform: "uppercase"
rounded:
  sm: "6px"
  md: "8px"
  lg: "12px"
  full: "9999px"
spacing:
  sm: "8px"
  md: "16px"
  lg: "24px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.neutral-bg}"
    rounded: "{rounded.md}"
    padding: "10px 20px"
    height: "36px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-primary-active:
    backgroundColor: "{colors.primary}"
    transform: "scale(0.98)"
  button-destructive:
    backgroundColor: "{colors.destructive}"
    textColor: "{colors.neutral-bg}"
    rounded: "{rounded.md}"
    padding: "10px 20px"
    height: "36px"
  button-destructive-hover:
    backgroundColor: "{colors.destructive-hover}"
  button-outline:
    backgroundColor: "transparent"
    textColor: "{colors.primary}"
    borderColor: "{colors.neutral-border-strong}"
    borderWidth: "1px"
    rounded: "{rounded.md}"
    padding: "10px 20px"
    height: "36px"
  button-outline-hover:
    backgroundColor: "{colors.neutral-surface}"
  input-field:
    backgroundColor: "{colors.neutral-bg}"
    textColor: "{colors.neutral-text}"
    borderColor: "{colors.neutral-border-strong}"
    borderWidth: "1px"
    rounded: "{rounded.md}"
    padding: "0 12px"
    height: "36px"
  input-field-focus:
    borderColor: "{colors.focus-ring}"
    boxShadow: "0 0 0 2px {colors.focus-ring}"
  badge-neutral:
    backgroundColor: "{colors.neutral-surface}"
    textColor: "{colors.neutral-text-muted}"
    rounded: "{rounded.full}"
    padding: "2px 8px"
  badge-warning:
    backgroundColor: "{colors.warning-bg}"
    textColor: "{colors.warning-text}"
    rounded: "{rounded.full}"
    padding: "2px 8px"
  badge-error:
    backgroundColor: "{colors.destructive-bg}"
    textColor: "{colors.destructive-text}"
    rounded: "{rounded.full}"
    padding: "2px 8px"
  badge-success:
    backgroundColor: "{colors.success-bg}"
    textColor: "{colors.success-text}"
    rounded: "{rounded.full}"
    padding: "2px 8px"
  card:
    backgroundColor: "{colors.neutral-bg}"
    borderColor: "{colors.neutral-border}"
    borderWidth: "1px"
    rounded: "{rounded.lg}"
    boxShadow: "0 1px 2px 0 rgb(0 0 0 / 0.05)"
  table-header:
    backgroundColor: "transparent"
    textColor: "{colors.neutral-text-muted}"
    fontWeight: "500"
  table-row:
    borderColor: "{colors.neutral-border}"
    borderWidth: "1px"
  table-row-hover:
    backgroundColor: "{colors.neutral-surface}"
  dialog-overlay:
    backgroundColor: "rgba(0, 0, 0, 0.4)"
  dialog-content:
    backgroundColor: "{colors.neutral-bg}"
    rounded: "{rounded.lg}"
    padding: "24px"
    boxShadow: "0 25px 50px -12px rgb(0 0 0 / 0.25)"
---

# Design System: xberg-web-ui

## 1. Overview

**Creative North Star: "The Technical Dossier"**

A design system for a power tool that runs a full RAG pipeline in the browser. The aesthetic is clinical, confident, and content-first — like a technical dossier declassified for the operator. Every pixel serves the task: create folder, drop documents, watch the engine work, sync when ready. No marketing fluff, no decorative motion, no mystery-meat states.

The system explicitly rejects **enterprise dashboard bloat**: dense tables with 20 columns, mystery-meat navigation, hidden background jobs, generic cream/slate SaaS palettes, tiny gray body text, loading spinners that never explain *what* is loading. Instead: visible engine state (SyncBar), explicit user actions (no invisible background uploads), semantic state colors that communicate instantly, and technical honesty over polish.

**Key Characteristics:**
- **Semantic state palette** — error/warning/success colors communicate at a glance; primary actions are slate-900 (near-black), not a brand hue
- **Flat by default, lift on state** — surfaces are flat at rest; shadows appear only as response to interaction (hover, focus, elevation)
- **Technical distinctive components** — shadcn primitives as foundation, but with custom variants for engine-specific states (SyncBar, DocumentTable, CreateFolderDialog)
- **System font stack** — no custom fonts; the tool disappears into the task
- **Keyboard-first, accessible by default** — WCAG 2.1 AA is the floor; focus rings, `role="alert"`, `prefers-reduced-motion` respected

## 2. Colors

A semantic state palette built on a slate neutral foundation. The primary action color is slate-900 (near-black) — authoritative, not decorative. State colors (destructive, warning, success) are high-contrast and used *only* for state communication, never decoration.

### Primary
- **Deep Slate** (`#0f172a`): Primary actions (Create, Sync), page titles, high-emphasis text. Used on ≤10% of any screen — its rarity is the signal.

### Destructive
- **Signal Red** (`#dc2626`): Destructive actions (Delete), critical errors in SyncBar, validation failures.
- **Red Hover** (`#b91c1c`): Hover state for destructive buttons.
- **Red Subtle** (`#fef2f2` / `#b91c1c`): Error badges, inline error text backgrounds.

### Warning
- **Amber** (`#f59e0b`): Pending/syncing state (SyncBar "Syncing N…"), non-blocking warnings.
- **Amber Subtle** (`#fffbeb` / `#b45309`): Warning badges, pending row highlights.

### Success
- **Emerald** (`#16a34a`): Completed sync, successful operations.
- **Success Subtle** (`#f0fdf4` / `#15803d`): Success badges, confirmation toasts.

### Neutral
- **White** (`#ffffff`): Page background, card surfaces, dialog content.
- **Slate 50** (`#f8fafc`): Hover surfaces, table row hover, input focus bg.
- **Slate 200** (`#e2e8f0`): Card borders, table row dividers, input borders at rest.
- **Slate 300** (`#cbd5e1`): Stronger borders (input focus fallback), disabled boundaries.
- **Slate 900** (`#0f172a`): Primary text, headings, primary buttons.
- **Slate 500** (`#64748b`): Muted text, table headers, placeholder text.
- **Slate 400** (`#94a3b8`): Focus rings, secondary focus indicators.

### Named Rules
**The One Voice Rule.** The primary accent (Deep Slate) appears on ≤10% of any given screen — primary buttons, page titles, active navigation. Its scarcity is the signal.

**The State-Only Rule.** Destructive, Warning, and Success colors are used *exclusively* for state communication (errors, pending, success). Never for decoration, never for primary actions, never as "accent" on inactive elements.

**The Contrast Floor Rule.** All text on neutral backgrounds hits ≥4.5:1 (body) or ≥3:1 (large). Slate-900 on White = 15.8:1. Slate-500 on White = 4.6:1 (passes). Red-600 on Red-50 = 6.2:1. Amber-600 on Amber-50 = 5.1:1. Emerald-600 on Emerald-50 = 5.8:1.

## 3. Typography

**Display Font:** system-ui stack (system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif)
**Body Font:** system-ui stack (same)
**Label/Mono Font:** system-ui stack (same), label uses uppercase + tracking

**Character:** The system font stack is deliberate — it's the native voice of the platform, invisible by design. No custom font to load, no flash of unstyled text, no brand tax. The tool disappears into the task.

### Hierarchy
- **Display** (600, clamp(1.5rem, 3vw, 2rem), 1.2, -0.02em): Page titles only (`Folders`, `{collection}`). One per screen.
- **Headline** (600, 1.25rem, 1.3): Section headers (`New folder` in dialog, `PII` column context).
- **Title** (500, 1rem, 1.4): Dialog titles, card headers, table cell links (document filenames).
- **Body** (400, 0.875rem, 1.5): All prose, helper text, error messages, empty states. Max line length 65–75ch in prose contexts.
- **Label** (500, 0.75rem, 1.5, 0.02em, uppercase): Form labels (`Folder name`, `Rehydration passphrase`), badge text, table headers.

### Named Rules
**The Single Stack Rule.** One font family everywhere. Display/body/label distinction comes from weight, size, case, and tracking — not font switching.

**The Clamp Ceiling Rule.** Display headings capped at 2rem (32px). Above that the page shouts, not designs.

**The Label Tracking Floor.** Labels use +0.02em letter-spacing and uppercase. Anything tighter cramps; anything looser loses scanability.

## 4. Elevation

Flat by default. Shadows appear *only* as a response to state: hover (cards lift), focus (rings), elevation (dialogs, dropdowns). No ambient shadows on resting surfaces.

### Shadow Vocabulary
- **Card Rest** (`0 1px 2px 0 rgb(0 0 0 / 0.05)`): Cards at rest — barely perceptible, tonal separation only.
- **Card Hover** (`0 4px 12px -2px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.06)`): Cards on hover — lift + diffuse.
- **Dialog** (`0 25px 50px -12px rgb(0 0 0 / 0.25)`): Modal overlay — high elevation, blocks background.
- **Focus Ring** (`0 0 0 2px #94a3b8`): Keyboard focus — visible, accessible, not decorative.

### Named Rules
**The Flat-By-Default Rule.** Surfaces are flat at rest. Shadows appear only as a response to state (hover, elevation, focus). No "card shadow" on every container.

**The Ring-Over-Glow Rule.** Focus uses a solid 2px ring (slate-400), not a diffuse glow. Rings are crisp, accessible, and work on any background.

## 5. Components

### Buttons
- **Shape:** Rounded corners (8px / `rounded-md`), 36px height, 20px horizontal padding.
- **Primary (Deep Slate):** `bg-slate-900 text-white hover:bg-slate-800`. Used for: Create folder, Sync, primary CTAs. One per screen max.
- **Destructive (Signal Red):** `bg-red-600 text-white hover:bg-red-700`. Used for: Delete document, Delete collection. Always paired with confirmation dialog.
- **Outline (Border):** `border border-slate-300 hover:bg-slate-100`. Used for: Cancel, secondary actions, "New folder" trigger (via DialogTrigger).
- **States:** Default → Hover (150ms) → Active (scale 0.98, 50ms) → Disabled (opacity-50, pointer-events-none). Focus: 2px slate-400 ring.

### Badges (Status Chips)
- **Shape:** Fully rounded (`rounded-full`), 2px vertical / 8px horizontal padding, 12px text (xs), medium weight.
- **Neutral (Slate 100/500):** Default state, "All synced", informational.
- **Warning (Amber 50/700):** Pending state — "Syncing N…", in-progress.
- **Error (Red 50/700):** Failed state — SyncBar errors, validation failures.
- **Success (Green 50/700):** Completed state — sync confirmed, ingestion done.
- **Usage:** Badges communicate *engine state only*. Never used as decorative tags.

### Inputs / Fields
- **Style:** 36px height, full width, 8px radius, 1px slate-300 border, 12px horizontal padding, 14px text.
- **Focus:** 2px slate-400 ring + border shift to slate-400, no outline.
- **Error:** Border shifts to red-500, ring to red-300, helper text in red-600.
- **Disabled:** Opacity-50, cursor-not-allowed, no ring on focus.
- **Label:** Separate `<label>` with `for` binding, 12px text, medium weight, slate-900.

### Cards / Containers
- **Corner Style:** 12px radius (`rounded-lg`).
- **Background:** White.
- **Border:** 1px slate-200.
- **Shadow:** `shadow-sm` at rest (1px diffuse), lifts to `shadow-md` on hover.
- **Internal Padding:** 16px (`p-4`) for CardContent, 16px + bottom border for CardHeader.

### Tables
- **Typography:** 14px text (`text-sm`), slate-900 body, slate-500 headers (medium weight).
- **Rows:** 1px slate-200 bottom border, hover → slate-50 background.
- **Header:** Left-aligned, 12px padding, no background.
- **Cell:** 8px padding, left-aligned.
- **Interactive cells:** Links use underline + slate-900, hover → slate-700.
- **Status column:** Badge component (semantic state colors).

### Dialogs
- **Overlay:** `fixed inset-0 bg-black/40` — 40% black, no blur.
- **Content:** White, 12px radius, 24px padding, max-width 512px (`max-w-lg`), centered.
- **Shadow:** High elevation (`shadow-lg` equivalent).
- **Header:** 16px bottom margin, title 18px semibold.
- **Footer:** 16px top margin, right-aligned, 8px gap.
- **Focus Trap:** Escape closes, click overlay closes, focus trapped inside.

### SyncBar (Signature Component)
- **Position:** Fixed bottom or sticky top of folder view.
- **Layout:** Right-aligned, 16px horizontal / 8px vertical padding, 1px top border (slate-200).
- **States:**
  - Error: Red badge (`role="alert"`) with error message.
  - Pending: Amber badge "Syncing N…" with count.
  - Synced: Muted slate-500 text "All synced" (no badge).
- **Philosophy:** The engine's heartbeat — always visible, never hidden, communicates *what* not just *that*.

### CreateFolderDialog (Signature Component)
- **Trigger:** Outline button "New folder" → opens Dialog.
- **Content:** Label + Input + Error (red) + Footer (Cancel outline, Create primary).
- **Validation:** Disables Create until name non-empty; sanitizes to alphanumeric/underscore/dash.
- **Keyboard:** Enter submits, Escape closes, focus traps to input on open.

### DocumentTable (Signature Component)
- **Columns:** Document (link to viewer), Status (badge), PII (comma-separated counts or "none").
- **Empty State:** "No documents yet." in slate-500, 14px.
- **Row Hover:** Slate-50 background, cursor-pointer on document link.
- **Virtualized:** TanStack Table handles large datasets.

## 6. Do's and Don'ts

### Do:
- **Do** use Deep Slate (`#0f172a`) for primary actions and page titles — and only there. One voice per screen.
- **Do** use state colors (Red/Amber/Emerald) exclusively for engine state communication — errors, pending, success.
- **Do** show the engine — SyncBar is always visible with current state; no invisible background work.
- **Do** make every network call user-initiated — Create Folder, Upload, Sync are explicit buttons.
- **Do** use 2px solid focus rings (slate-400) on all interactive elements — crisp, accessible, works on any bg.
- **Do** respect `prefers-reduced-motion` — transitions become instant crossfades (0ms or 50ms max).
- **Do** use semantic HTML — `<button>` for actions, `<label for>` for inputs, `<table>` for data, `role="alert"` for live errors.
- **Do** keep surfaces flat at rest — shadows only on hover, focus, or elevation (dialogs).
- **Do** use system font stack — no custom fonts to load, no FOUT, no brand tax.
- **Do** sanitize folder names client-side (alphanumeric/underscore/dash) before server call.

### Don't:
- **Don't** use border-left/right >1px as colored stripes on cards, rows, or alerts — enterprise dashboard bloat.
- **Don't** use gradient text (`background-clip: text`) — decorative, never meaningful. Use weight/size for emphasis.
- **Don't** use glassmorphism/backdrop-blur as default — rare and purposeful, or nothing.
- **Don't** use the hero-metric template (big number + small label + stats) — SaaS cliché.
- **Don't** use identical card grids with icon + heading + text repeated endlessly.
- **Don't** put tiny uppercase tracked eyebrows above every section — AI grammar, not voice.
- **Don't** use numbered section markers (01/02/03) as default scaffolding — numbers earn their place in real sequences.
- **Don't** let text overflow containers — test headings at every breakpoint; clamp max or rewrite copy.
- **Don't** hide background work — no silent uploads, no mystery spinners. SyncBar surfaces every pending/error.
- **Don't** use color-only state changes — always pair with text/icon/badge/text + color.
- **Don't** use cream/sand/beige warm-neutral body backgrounds — the saturated AI default of 2026. White or brand-tinted neutral only.
- **Don't** ship components without all states — default, hover, focus, active, disabled, loading, error.
- **Don't** use modals as first thought — exhaust inline/progressive alternatives first (CreateFolderDialog is justified: it's a creation flow with validation).
- **Don't** use display fonts in UI labels, buttons, or data — system stack everywhere.
- **Don't** reinvent standard affordances — custom scrollbars, weird form controls, non-standard modals are forbidden.
- **Don't** use heavy color or full-saturation accents on inactive states — state colors are for active communication only.
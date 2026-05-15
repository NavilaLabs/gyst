# Design System Specification: The Tactile Atelier

## 1. Overview & Creative North Star
**Creative North Star: The Curated Workspace**

This design system rejects the sterile, "software-as-a-service" aesthetic in favor of a high-end, editorial experience. It is designed to feel like a bespoke oak desk—meticulously organized, warm to the touch, and intellectually stimulating. We achieve this by moving away from standard grid-bound layouts and embracing a "layered paper" philosophy.

The system breaks the "template" look through **Intentional Asymmetry**. Large serif displays are offset against hyper-legible sans-serif data points. We prioritize breathing room over information density, ensuring that project-based time tracking feels like a rewarding ritual rather than a chore.

---

## 2. Colors & Surface Architecture
The palette is rooted in `surface` (#fafaf5) and `on_surface` (#1a1c19), creating a high-contrast environment for productivity that remains easy on the eyes.

### The "No-Line" Rule
**Explicit Instruction:** Designers are prohibited from using 1px solid borders to define sections. Layout boundaries must be defined solely through background color shifts or tonal transitions. Use `surface_container_low` (#f4f4ef) to set off a sidebar from a `surface` (#fafaf5) main stage.

### Surface Hierarchy & Nesting
Treat the UI as a physical stack of fine paper. 
- **Base Level:** `surface` (#fafaf5)
- **Secondary Sections:** `surface_container` (#eeeee9)
- **Nested Detail Cards:** `surface_container_lowest` (#ffffff) to create a subtle "pop" of brightness.
- **Floating Overlays:** Use `surface_bright` (#fafaf5) with a 60% opacity and a `20px` backdrop-blur to create a "glassmorphism" effect for navigation bars or floating action menus.

### Signature Textures
Main CTAs and progress indicators should use a subtle linear gradient: 
`linear-gradient(135deg, primary (#0d631b) 0%, primary_container (#2e7d32) 100%)`. This adds a "visual soul" and depth that a flat hex code cannot achieve.

---

## 3. Typography
Our typography is a dialogue between the artisanal and the analytical.

- **Display & Headlines (Noto Serif):** Used for project titles and high-level summaries. The serif adds a "literary" quality that feels premium and established. Use `display-lg` (3.5rem) for empty states and `headline-md` (1.75rem) for dashboard sections.
- **UI & Data (Inter):** Used for all functional elements, time logs, and labels. Inter provides the clinical precision required for time tracking. 
- **The Contrast Rule:** When displaying a "Time Elapsed" counter, use `title-lg` (1.375rem) in Inter, but pair it with a `label-sm` (0.6875rem) category name in all-caps with `0.05em` letter spacing to ensure the data feels "high-contrast" and authoritative.

---

## 4. Elevation & Depth

### The Layering Principle
Depth is achieved through "Tonal Stacking." To elevate a project card, do not reach for a shadow first; instead, place a `surface_container_lowest` (#ffffff) card on top of a `surface_container` (#eeeee9) background. The change in "warmth" creates natural separation.

### Ambient Shadows
When a component must float (e.g., a "Start Timer" FAB), use an extra-diffused shadow:
- **X: 0, Y: 8, Blur: 24, Spread: -4**
- **Color:** `on_surface` (#1a1c19) at **6% opacity**. 
This mimics natural, soft-box studio lighting rather than digital drop shadows.

### The "Ghost Border" Fallback
If contrast is required for accessibility (e.g., in a high-glare environment), use the `outline_variant` (#bfcaba) at **15% opacity**. It should be felt, not seen.

---

## 5. Components

### Buttons
- **Primary:** Gradient fill (Primary to Primary Container), `rounded-md` (1.5rem). Text should be `label-md` White.
- **Secondary:** `surface_container_high` (#e8e8e3) background with `on_surface` text. No border.
- **Tertiary:** No background. Underlined only on hover with a 2px stroke of `tertiary_fixed`.

### Input Fields
- **Style:** Background of `surface_container_low` (#f4f4ef), `rounded-DEFAULT` (1rem). 
- **States:** On focus, transition the background to `surface_container_lowest` (#ffffff) and add a "Ghost Border" of `primary` at 20% opacity.

### Cards & Time-Logs
- **Prohibition:** **No divider lines.**
- **Implementation:** Separate time entries using the `spacing-4` (1.4rem) scale. Group related logs within a `surface_container` capsule with `rounded-lg` (2rem) corners.

### Project Chips
- Use `secondary_container` (#cfe6f2) with `on_secondary_container` (#526772) text.
- Shape: `rounded-full` (9999px) for a soft, pebble-like feel.

### Time-Tracking Progress Bar
- **Track:** `surface_dim` (#dadad5).
- **Indicator:** A soft gradient of `tertiary` (#00598f) to `tertiary_fixed_dim` (#99cbff).

---

## 6. Do's and Don'ts

### Do
- **Do** use generous whitespace (Scale 8 or 10) between major project modules to prevent cognitive overload.
- **Do** use `notoSerif` for numbers in a decorative context (e.g., "Day 12 of Project"), but use `inter` for actionable data (e.g., "12:45:02").
- **Do** lean into the "Oatmeal" warmth. If a layout feels too "techy," increase the use of `surface_container`.

### Don't
- **Don't** use pure black (#000000) for text. Always use `on_surface` (#1a1c19) to maintain the "ink on paper" softness.
- **Don't** use sharp corners. Everything must have at least a `rounded-sm` (0.5rem) radius to maintain the "Cosy" promise.
- **Don't** use standard 12-column grids for everything. Try offsetting the main content container by `spacing-6` to create a more editorial, asymmetrical flow.
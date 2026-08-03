# Wira Digital Indonesia — Design System

> A warm, local, minimal-native design system for **Wira Digital Indonesia**, an
> independent Indonesian software studio that ships small, fast, no-nonsense desktop
> utilities — offered free. Its first product is **WinTick**.

This repository is both a **design guide** and a **component/asset library** consumed by
design agents to produce on-brand interfaces, marketing sites, decks and product mockups.

---

## 1. Company & product context

**Wira Digital Indonesia** ("wira" = champion / hero, from Sanskrit *vīra*) is a solo/small
Indonesian software studio. Positioning at this stage:

- Builds and gives away polished, tiny, performance-obsessed **desktop utilities**.
- **Brand model:** Wira Digital Indonesia is the umbrella studio brand. Each product gets
  its own product wordmark and its own one-page marketing site, plus a single company
  landing page for the studio itself.
- **First product — WinTick:** a Windows 11 background utility that brings the macOS
  `Cmd + \`` experience (cycle windows *within the same app*) to Windows, plus
  BetterSnapTool-style window snapping. Rust, native Win32, invisible by design (the core
  feature literally has zero UI). [redacted: unverified performance claims and distribution
  channel]

### Two visual worlds
This system deliberately holds **two type/color worlds** that must never blend:

1. **Brand world** — marketing sites, company page, store listing, decks, docs. Warm,
   local, confident. Uses the *Wira* brand tokens + **Plus Jakarta Sans**.
2. **Native world** — WinTick's actual in-product UI (settings, onboarding, tray). Mimics
   Windows 11 Fluent so it feels first-party. Uses the `.native` tokens + **Segoe UI**.
   Brand saga-red never appears in native chrome; the only injected color is the mandated
   tray-alert red `#E81123`.

### Sources used to build this system
- **The product repository** (Rust workspace plus its planning archive). No visual assets, logo, or design tokens existed in it — the
  product intentionally uses native OS chrome — so the *brand* identity here is authored
  fresh, grounded in the product's documented character (performance, honesty, calm,
  invisibility) and the studio's Indonesian identity.
  Key docs read: `prd.md`, `DESIGN.md`, `EXPERIENCE.md`, `brainstorm-intent.md`.
  → Explore the repository further to deepen product-accurate work.

There is **no logo** in the source. See *Iconography* below for how the brand mark is
handled (typographic wordmark + a generic function glyph — both placeholders you can
replace).

---

## 2. Content fundamentals — how Wira writes

Wira is **bilingual (Bahasa Indonesia + English)**. Marketing leads in Bahasa Indonesia
with English available; technical/product docs may lead in English. The two never mix
mid-sentence.

**Voice:** warm but precise. Proud-local without being loud. Honest to a fault — this is a
studio that will *show you a hung window rather than hide it* ("UX Honesty"), and it writes
the same way: no overclaiming, no dark patterns, no hype.

- **Tone:** calm, direct, quietly confident. Speaks *to* the user ("kamu"/"you"), refers to
  the studio as "kami"/"we". Never corporate-stiff, never bro-casual.
- **Casing:** Sentence case for headings and buttons ("Unduh sekarang", "Download now"), not
  Title Case. UPPERCASE only for short eyebrow labels with letter-spacing ("RINGAN • CEPAT").
- **Numbers as proof:** the brand treats specific numbers as earned headlines, set in mono.
  Any figure used this way MUST be a measured result, not a target — see `SECURITY.md` and
  the release gate for which claims are approved. [redacted: unverified performance claims]
- **Microcopy is solutive:** every warning offers the next step. Not "Hook failed" but
  "Keyboard hook blocked — try running as Administrator" / "Hook keyboard diblokir —
  coba jalankan sebagai Administrator."
- **No emoji** in product UI or marketing headlines. (Docs/READMEs internal to the studio
  may use them sparingly; brand-facing surfaces do not.)
- **Vibe examples:**
  - Hero: *"Pindah antar jendela seperti di Mac. Di Windows."* / *"Switch between an app's
    windows like on a Mac. On Windows."*
  - Value: *"Ringan, jujur, dan tak terlihat. Cara utilitas seharusnya bekerja."*
  - CTA: [redacted: distribution channel]
  - Empty/again: *"Tekan `Win + \`` untuk mencoba."*

---

## 3. Visual foundations

### Color
- **Primary — Saga red** (`--saga-500 #C63A28`): a warm, earthy brick red (merah saga).
  It is a deliberate *warming* of WinTick's harsh alert red into something with local,
  hand-dyed-textile warmth. Used for primary actions, brand fills, the wordmark accent.
- **Neutrals — Tanah** (warm stone, `--tanah-50` paper → `--tanah-900` ink): everything sits
  on warm paper `#FBF9F6`, never cold white/gray. Text is warm near-black `#1A1611`.
- **Accent — Emas** (gold `--emas-500 #D99A32`): sparing warmth — underlines, small marks,
  the occasional highlight. Never a second primary.
- **Functional signals held separate** from brand: danger `#E81123` (also the WinTick tray
  alert), warning `#E0A11B`, success `#2E9E5B`, info `#2D74C4`, each with a soft tint.
- **Imagery vibe:** warm, natural light, slightly desaturated; product screenshots shown on
  paper/stone backdrops, never on gradients. No duotone, no heavy grain.

### Typography
- **Plus Jakarta Sans** for all brand surfaces — chosen intentionally: it was commissioned
  as the visual identity typeface of **Jakarta**, so it carries the local-pride story. Warm,
  geometric-humanist, great from 12px captions to 76px display.
- **JetBrains Mono** for keys, shortcuts (`Win + \``), `config.toml`, code, and the proof
  numbers.
- **Segoe UI** (via `--font-native`) for WinTick's native app recreation only.
- Display is tight (`--tracking-tight`, `--leading-tight`); body is `--leading-relaxed`.
  Eyebrows are uppercase mono/sans with `--tracking-caps`.

### Spacing & layout
- 4px base grid (`--space-*`). Generous whitespace — minimal means *calm*, so let things
  breathe. Content columns cap at `--container-lg/xl`.
- Layout is quiet and grid-aligned; no busy backgrounds. Backgrounds are flat warm color —
  **no gradients, no patterns, no full-bleed hero photography by default** (a single product
  screenshot or a flat color block is the hero device).

### Radii, borders, cards
- Moderate rounding: `--radius-md 12px` for cards/buttons, `--radius-sm 8px` for inputs,
  `--radius-pill` for tags/eyebrows. Nothing sharp, nothing bubbly.
- Cards: white on paper, `1px --border-subtle` **or** a soft warm shadow (`--shadow-md`) —
  usually one, not both. No colored left-border accent cards. No glassmorphism on brand
  surfaces (blur/transparency belongs to the native Windows world only).

### Shadows
- Warm-tinted, low-spread, layered softly (`--shadow-xs → --shadow-xl`), always using warm
  ink `rgba(26,22,17,…)` — never pure black. Native surfaces use their own Fluent
  flyout/window shadows.

### Motion & states
- **Calm and native.** Fades + small (2–8px) travel. No bounce, no spring, no long
  durations. `--dur-fast/base/slow` with `--ease-standard`. The WinTick product itself is
  **zero-animation** by design; brand marketing may animate gently on scroll/hover only.
- **Hover:** brand fills darken one step (`--brand → --brand-hover`); ghost/quiet elements
  gain a subtle warm tint. **Press:** darken one more step (`--brand-active`) + optional 1px
  nudge; no scale-down on brand. Focus: `--ring-brand` (3px warm saga ring).
- **Transparency/blur:** only in the native Windows world (Mica/acrylic feel). Brand
  surfaces are opaque.

---

## 4. Iconography

- **No icon set ships in the WinTick source** (it uses Segoe Fluent Icons natively, a font
  that isn't web-distributable). For this system:
  - **Brand + marketing + kits use [Lucide](https://lucide.dev)** via CDN — clean, geometric,
    1.75px stroke line icons that pair well with Plus Jakarta Sans and read modern-but-calm.
    **This is a substitution** for a bespoke set; flagged here.
  - **WinTick native recreations** approximate Windows 11 **Segoe Fluent Icons** with Lucide
    equivalents (settings, chevrons, window, etc.). Also a substitution — a production build
    would use the real Segoe Fluent Icons font. Flagged.
- **No emoji** as iconography anywhere brand-facing. **No hand-rolled decorative SVGs.**
- Unicode is used only where it is genuinely typographic: the backtick key `` ` ``, arrow
  glyphs on keycaps (← → ↑ ↓), the `Win`/`Ctrl` key labels set in mono.
- **App/product glyph:** WinTick's mark is a generic *two-overlapping-rounded-rectangles*
  ("stacked windows") glyph representing the switch-between-windows function — this is
  functional iconography, **not** an invented company logo. It, and the typographic
  wordmarks, are placeholders. **→ ASK: replace with your real marks when ready.**

---

## 5. Index / manifest

### Root
- `styles.css` — the single global entry point (import lines only). Consumers link this.
- `tokens/` — `fonts.css`, `colors.css`, `typography.css`, `spacing.css`, `effects.css`,
  `native.css` (Windows-11 surface tokens), `semantic.css` (role aliases).
- `assets/` — `wintick-glyph.svg` (brand, 128), `wintick-glyph-mono.svg` (tray/mono).
- `guidelines/` — foundation specimen cards (Colors, Type, Spacing, Effects, Brand).
- `SKILL.md` — Agent-Skills-compatible entry for downloaded use.

### Components → `window.WiraDigitalDesignSystem_61a646` (via `_ds_bundle.js`)
Focused on WinTick's needs plus brand marketing:
- **controls/** — `Button` (brand + `native` variant), `Toggle` (brand + native switch),
  `Keycap` (keys & combos), `ShortcutInput` (listening-mode capturer, FR-18).
- **surfaces/** — `Card` (elevated / outline / sunken / ink), `Dialog` (native Windows
  window chrome + brand modal).
- **feedback/** — `Badge` (status & eyebrow), `TrayIcon` (3-tier health, FR-11).
- **navigation/** — `TrayMenu` (Windows-11 tray context flyout, FR-16).

Each component directory carries its `.jsx` + `.d.ts` + `.prompt.md` + a `@dsCard` demo HTML.

### UI kits → `ui_kits/`
- **wintick-app/** — native Windows-11 world: interactive desktop with first-run onboarding,
  tray menu, Settings window, and the 3-tier tray states + toast.
Two further kits exist in the private repository and are deliberately **not** published:
a marketing landing page with a store listing, and the umbrella studio landing page. Both
carry performance and availability claims that have not been approved, and the studio page
speaks on behalf of an entity that is still being established.

### Slides → `slides/`
Six brand slide templates (1280×720): title, agenda, feature, comparison, quote, closing.

### Sources
Built from the product repository, whose source and planning archive are published here.
Explore them further for product-accurate work.

---

## 6. Caveats & substitutions (please confirm)
- **Brand created from scratch.** WinTick's source has no visual identity by design (it uses
  native OS chrome), so all *brand* decisions here — name treatment, color, type — are
  proposals, not extracted facts.
- **No logo.** The wordmark, "W" monogram, and stacked-windows glyph are **placeholders**.
  Replace with real marks when ready.
- **Fonts:** Plus Jakarta Sans + JetBrains Mono load from Google Fonts CDN (not self-hosted
  binaries). `--font-native` targets **Segoe UI Variable / Segoe UI**, which are OS fonts —
  the platform stack renders them; no file is bundled (this is intentional for the native
  world, not a substitution to fix).
- **Icons:** [Lucide](https://lucide.dev) via CDN stands in for a bespoke set; WinTick's real
  native icons would be Segoe Fluent Icons. Flagged — swap if you have a preferred set.

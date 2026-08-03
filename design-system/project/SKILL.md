---
name: wira-digital-design
description: Use this skill to generate well-branded interfaces and assets for Wira Digital Indonesia (and its product WinTick), either for production or throwaway prototypes/mocks. Contains essential design guidelines, colors, type, fonts, assets, and UI-kit components for prototyping. Wira is a warm, local, minimal-native Indonesian software studio; WinTick is its Windows-11 window-switcher product with a deliberately native look.
user-invocable: true
---

Read the `readme.md` file within this skill, and explore the other available files.

If creating visual artifacts (slides, mocks, throwaway prototypes, etc), copy assets out and
create static HTML files for the user to view. If working on production code, you can copy
assets and read the rules here to become an expert in designing with this brand.

Key things to hold onto:
- **Two worlds, never mixed.** The *brand world* (marketing, company/product sites, decks,
  docs) uses Plus Jakarta Sans + warm saga red + the `--*` brand tokens. The *native world*
  (WinTick's in-product UI) mimics Windows 11 Fluent, uses Segoe UI + the `.native` tokens,
  and never shows brand saga — only the mandated tray-alert red `#E81123`.
- **Voice:** warm, precise, honest, bilingual (Bahasa Indonesia + English). Sentence case,
  no emoji, solutive microcopy, numbers-as-proof in mono. See readme §2.
- **Tokens** live in `tokens/*.css`, all reachable from `styles.css` — link that one file.
- **Components** compile to `window.WiraDigitalDesignSystem_61a646` via `_ds_bundle.js`
  (Button, Toggle, Keycap, ShortcutInput, Card, Dialog, Badge, TrayIcon, TrayMenu).
- **UI kits** in `ui_kits/` and **slides** in `slides/` show real compositions to copy.
- **No real logo exists** — a typographic wordmark + a generic "stacked windows" product
  glyph (`assets/wintick-glyph*.svg`) stand in. Treat them as placeholders.

If the user invokes this skill without any other guidance, ask them what they want to build
or design, ask some questions, and act as an expert designer who outputs HTML artifacts _or_
production code, depending on the need.

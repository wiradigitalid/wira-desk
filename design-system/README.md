# Design system

**Read this before anything below it.**

This is the design system for Wira Desk: brand foundations, design tokens, component
prototypes, and a recreation of the product's native Windows UI. It is published so design
work can continue alongside the code.

It is **not** production code. The prototypes are HTML/CSS/JS, while the product is a native
Win32 application in Rust that deliberately uses native OS chrome — its core feature has no UI
at all. Treat these files as the visual specification rather than as something to import.

Four things follow, and they matter before you rely on anything here:

**The brand marks are placeholders.** The wordmark, the "W" monogram, and the
stacked-windows glyph were authored as stand-ins and are labelled as such in
`project/readme.md`. There is no approved logo. Replace them rather than shipping them.

**The brand identity is a proposal, not an extracted fact.** The product has no visual
identity by design, so the colour, type, and name treatment here were authored fresh from the
product's documented character. Nothing in this directory went through a brand approval
process.

**Some content is redacted**, marked inline as `[redacted: <class>]`. Two classes appear here:
*unverified performance claims*, where a specific figure had been used as a headline without a
measurement behind it, and *distribution channel*, where copy named a store or release channel
that has not been committed to. The surrounding sentence is left intact so the design intent
stays readable. The planning archive under `_bmad-output/` uses the same convention with a
different set of classes, listed in its own README.

**The product was originally named WinTick.** That name appears throughout, including in
directory names such as `project/ui_kits/wintick-app/`. The rename is described in the root
`README.md`.

## What is here

| Path | Contents |
| --- | --- |
| `project/readme.md` | The design guide — voice, colour, type, spacing, motion, iconography. Start here. |
| `project/styles.css` | The single global entry point; imports everything under `tokens/`. |
| `project/tokens/` | Design tokens, split into the brand world and the `native` Windows-11 world. |
| `project/components/` | Component prototypes, each with a `.jsx`, a `.d.ts`, a prompt file, and a demo. |
| `project/guidelines/` | Foundation specimen cards for colour, type, spacing, and effects. |
| `project/ui_kits/wintick-app/` | A recreation of the product's native UI: onboarding, tray menu, settings window. |
| `project/assets/` | The placeholder product glyph, in brand and mono variants. |

Read `project/readme.md` top to bottom before implementing anything, then follow its imports
so you can see how the tokens and components fit together.

## What is deliberately absent

A marketing landing page, a studio landing page, a brand slide deck, the compiled component
bundle, and two standalone viewer builds are not published. The landing pages carried
performance and availability claims that were never approved and spoke on behalf of a studio
that is still being established; the deck carried the same claims as headline copy. The
compiled bundle and the standalone viewers are generated artifacts that had embedded copies of
the landing pages, so excluding the sources without excluding the build output would have
published the same content anyway.

## These prototypes reach the network — the product does not

Worth stating plainly, because the two facts sit next to each other and are easy to conflate.
`PRIVACY.md` says the product sends no network traffic, and that is accurate: the Rust binaries
contain no socket and no HTTP client. **The prototypes in this directory are different.** Opening
one in a browser fetches React and Babel from a package CDN and typefaces from Google Fonts, so
those requests are visible to those hosts. Nothing here is instrumented and no data about you is
sent deliberately, but a request is a request.

That also means the third-party libraries and typefaces used by the prototypes are **not
redistributed** in this repository — they are referenced by URL. `NOTICE` covers the Rust
dependencies that are actually compiled into the product, so it does not list them. If you
vendor any of them later, their licences become your obligation: the typefaces are under the
Open Font License, and the JavaScript libraries are MIT-family.

## Two visual worlds

The one rule in here that is easy to break by accident: this system holds two type and colour
worlds that must never blend.

The **brand world** covers documentation and any marketing surface — warm, local, and
confident, using the brand tokens. The **native world** is the product's own UI, which mimics
Windows 11 Fluent so it feels first-party, using the `.native` tokens and the OS font stack.
Brand colour never appears in native chrome; the only injected colour there is the tray alert
red that Windows itself mandates for that state.

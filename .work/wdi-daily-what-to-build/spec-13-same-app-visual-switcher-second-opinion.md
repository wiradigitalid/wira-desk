# Second-Opinion Review Packet: SPEC-13 Same-App Visual Switcher

## Instructions for Reviewer (Opus)
Please perform an independent, deep second-opinion analysis and review of the drafted specification and tickets below against the user's original raw notes and the Wira Desk codebase principles (lightweight native Win32/Rust, zero-copy DWM thumbnail usage, zero idle footprint, bounded memory, and clean UI/UX).

Review the drafted SPEC and tickets for:
1. Adherence to the owner's original raw notes (did we capture the user's exact needs without drifting or over-engineering?).
2. Technical viability and edge cases (DWM thumbnail lifecycle, DPI scaling, multi-monitor bounds, minimized windows, cancellation via Esc, and hold-vs-tap timing).
3. Soundness of vertical slicing and ticket boundaries across the 4 tickets.

If you find gaps, edge-case risks, or architectural ambiguities, you are authorized to edit the documents directly or output concrete, actionable recommendations to fold in.

Standing mandate:
"If this draft touches the architecture spine, an SRS, an SDD, or a SPEC, you're authorized to run `wdi-review` on it yourself and edit the document directly to apply its stamp — no need to ask first, that permission is already given for this dispatch."

---

## 1. Path to Drafted SPEC and Tickets
- Spec: `.scratch/spec-13-same-app-visual-switcher/SPEC.md`
- Tickets:
  - `.scratch/spec-13-same-app-visual-switcher/issues/01-configuration-schema-and-settings-pane.md`
  - `.scratch/spec-13-same-app-visual-switcher/issues/02-domain-layout-grid-pagination-and-state-machine.md`
  - `.scratch/spec-13-same-app-visual-switcher/issues/03-win32-overlay-window-and-dwm-thumbnail-lifecycle.md`
  - `.scratch/spec-13-same-app-visual-switcher/issues/04-hook-interaction-hold-timer-and-multi-monitor-activation.md`
- Registry entry:
  - `.control/registry/specs.yaml` (entry `SPEC-13`)

---

## 2. Original Raw Notes (Verbatim)
```text
Kalau alternate tab di Windows atau Windows tab atau Tax View Sebenernya kayak Tax View sih lebih tepatnya ya Jadi Tax View itu, tapi alternate tab juga boleh Artinya saya pengen ada tampilan window yang berisikan Bukan tampilan window, ibaranya tampilan di depan layar saya, Windows ini beberapa app yang sejenis muncul. Kalau Command Backtick atau Windows Backtick itu kan rotasi langsung. Nah saya ingin tahu dari Virtual Desktop Sat ini saya pengen tahu misalnya Chrome ada window nya apa aja kayak di Macbook, Macbook kan bisa begitu kan jadi kita bisa kayak TouchView untuk aplikasi aplikasi sejenis apakah memungkinkan itu dilakukan di Windows biar ditaruh di WiraDex, coba kamu riset

ya saya ingin seperti Alt+Tab saja, tapi same app. mengenai jika banyak window lebih dari yang bisa ditampilkan, saya ingin ada uiux solusinya yang enak dilihat (bagaimana navigasi yang tepat). Dan apakah ini pekerjaan besar? serta apakah sangat jauh memperbesar runtime memory?
```

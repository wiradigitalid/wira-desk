---
title: Sprint Change Proposal — Phase 4 Code Review Resolution
created: 2026-08-02
author: kodesh87
workflow: bmad-correct-course
status: partially-executed
revised: 2026-08-02
branch: docs/correct-course-fase4
baseline_commit: 0149ec8
---

> **Revision notice (2026-08-02).** §4.1 has been substantially corrected after the
> measurements it relied on were shown to describe the wrong code path. Three of its claims
> were wrong; the corrections are recorded in place rather than silently rewritten, because
> the delta is the useful part. Its four edits are now **held** pending a valid measurement.
> §4.2 has been implemented and merged (`0e81cfb`). Everything else stands as first drafted.

# Sprint Change Proposal — Phase 4 Code Review Resolution

## 1. Issue Summary

### 1.1 Trigger

The `bmad-code-review` run of 2026-08-02 (three fresh-context passes in a dedicated
worktree, branch `review/fase4-code-review-epic4-5`, merged as PR #8 → PR #9) produced
**35 findings** across Stories 4.4, 5.2, and 5.3–5.6. The 18 `patch`-tagged findings were
applied and verified. The **11 `decision-needed` findings were deliberately left open**
because they appeared to require product or architecture judgement rather than a
mechanical fix. Per the review workflow's own status rule, all six affected stories were
returned from `review` to `in-progress`.

A twelfth item is folded into this proposal because it is the same class of problem and
gates the same epics: **NFR10 last measured p95 = 17.5 ms against a < 1 ms target**
(Story 2.6, elevated run of 2026-07-26), which formally holds Epics 3/4/5 through
Story 2.6's acceptance gate.

### 1.2 Core problem, precisely stated

Categorisation (checklist item 1.2): this is **not** one issue but three distinct
categories that were incorrectly pooled under one label.

| Category | Count | Nature |
| --- | --- | --- |
| **A — Mislabelled: already decided by an existing AC** | 7 | The binding artifact (epics.md AC, PRD FR, UX-DR, or architecture AD) already dictates the answer. No product decision exists to make; the code or the story doc simply diverges from the contract. |
| **B — Genuine gap: no AC covers it** | 2 | A real defect that no acceptance criterion anticipated. Requires a new AC. |
| **C — Genuine requirement conflict** | 1 | A requirement is unachievable by construction because two binding requirements contradict each other. Requires a requirement change. |

The remaining 2 items are documentation corrections that follow from the above.

**The single most consequential discovery** made while analysing these items is a
dependency the review did not detect: **finding 5.5-A (onboarding practice exercise
unimplemented) cannot be implemented at all until finding 5.4-A (daemon hook swallows the
captured combination) is resolved.** Story 5.5 AC forbids onboarding from installing a
second global keyboard hook, so the onboarding window can only observe `Win+Backtick`
through normal window input — which the daemon's hook currently swallows system-wide
before any window sees it. Two items filed as independent are in fact one ordered pair.

### 1.3 Evidence

| Claim | Evidence |
| --- | --- |
| Daemon never handles `WM_APP_RELOAD_CONFIG` | `grep -rn "WM_APP_RELOAD_CONFIG" crates/` returns only: `shared/constants.rs:46` (definition), `settings/persistence.rs:158` (post), `settings/persistence.rs:405` (test), `daemon/tray.rs:57` (doc comment). No handler. Message falls through to `DefWindowProcW`. |
| The architecture SSOT expects a file that was never created | `ARCHITECTURE-SPINE.md` Source Tree declares `daemon/config.rs # Config reload on WM_APP_RELOAD_CONFIG`. `ls crates/daemon/src/config.rs` → does not exist. |
| NFR10 is unachievable by construction | NFR7 mandates live `EnumWindows` per keypress with caching prohibited; AR-04/AD-9 mandate an `IVirtualDesktopManager` COM query per candidate. Measured breakdown (root `/3p.md`, 2026-07-26): hoisting COM creation to `thread_local!` moved p95 24.2 → 17.5 ms; the residual ~14 ms is `EnumWindows` + `OpenProcess`/`QueryFullProcessImageNameW` per window + per-candidate COM. Sub-1 ms cannot contain that work. **That measurement is stale** — see §4.1 for the three later commits that changed the measured path. |
| NFR10 has a sanctioned amendment path | `epics.md:587` — "any miss prevents NFR10 from being marked satisfied **unless an explicit approved requirement change replaces the target**." |
| 5.2's "description" requirement is a story-doc overclaim, not an epic AC | `epics.md:1097` requires "a stable role, accessible name, current value, enabled state, and checked or listening state". FR-21 and UX-DR7 both say **state**. Neither requires a description/HelpText. The phrase "descriptions … attached via `WidgetInfo`" exists only in the story file's paraphrase. |
| Onboarding practice cannot use its own hook | `epics.md:1234` — "**And** installs no second global keyboard hook". |

---

## 2. Impact Analysis

### 2.1 Epic impact (checklist section 2)

| Epic | Can it complete as planned? | Change required |
| --- | --- | --- |
| **Epic 2** | No — Story 2.6 blocked on NFR10 | NFR10 target replaced (§4.1). No story added or removed. |
| **Epic 3** | Yes — unblocked as a consequence | None directly. Stories 3.2/3.4 remain `in-progress` on their own untested runtime matrices, unrelated to this proposal. |
| **Epic 4** | Yes, with one new AC | Story 4.4 gains an AC for maximized-window restore (§4.4). Ownership of the shell-surface guard clarified as 4.4, not 4.5 (§4.5). |
| **Epic 5** | Yes, with one new AC and one ordering constraint | Story 5.3 gains a shortcut-collision AC (§4.6). Story 5.6 must land the daemon-side capture-state handler **before** Story 5.4's capturer and Story 5.5's practice exercise can be verified end-to-end (§4.7, §4.8). |

No epic is invalidated, no new epic is needed, and no epic is resequenced. Epic
**ordering within Epic 5 changes**: 5.6's capture-state handler now strictly precedes
end-to-end verification of both 5.4 and 5.5.

### 2.2 Artifact conflicts (checklist section 3)

| Artifact | Conflict | Action |
| --- | --- | --- |
| **PRD** `prd.md` §8 Success Metrics | "Latensi Rotasi (perceived, end-to-end): Sub-milidetik (< 1ms)" is unachievable under §4.2's own Stateless Z-Order prohibition | Amend the metric (§4.1) |
| **epics.md** NFR10 (line 57) | Same conflict, requirement form | Amend (§4.1) |
| **epics.md** Story 2.6 AC (line 586) | Gate cites the 1 ms figure | Amend (§4.1) |
| **epics.md** Story 4.4 AC | No AC covers a maximized foreground window | Add AC (§4.4) |
| **epics.md** Story 5.3 AC | No AC covers cross-field shortcut collision | Add AC (§4.6) |
| **ARCHITECTURE-SPINE.md** AD-5 | Rule is correct and binding; the **implementation** is absent, and the declared `daemon/config.rs` was never created | No spine change. Implement to match (§4.2) |
| **ARCHITECTURE-SPINE.md** AD-6 | **No conflict.** VM/RDP bypass is a separate mechanism and is deliberately left untouched (§4.7) | None |
| **ARCHITECTURE-SPINE.md** Consistency Conventions | No row documents a Settings→daemon capture-state signal; AD-1 already permits the channel | Add one conventions row (§4.7) |
| **EXPERIENCE.md** IA (Core Switcher / Window Snapping / Stack Layout) | Implemented panes are General/Shortcuts/Layout/About with all six shortcut fields flattened | No UX change. Implement to match (§4.9) |
| **Story file** `5-2-…md` AC-5.2-002 | Overclaims a description requirement absent from epics.md/FR-21/UX-DR7 | Correct the story doc (§4.3) |
| **Story file** `5-3-to-5-7-…md` Status table | "Settings→daemon direction is done" conflates *posted* with *handled* | Correct the story doc (§4.2) |
| **UX DESIGN.md / EXPERIENCE.md** | No other conflict found | N/A |

### 2.3 Technical impact

- **New file:** `crates/daemon/src/config.rs` (declared by the spine, never created).
- **Modified:** `daemon/tray.rs` (message arm), `daemon/hook.rs` + `daemon/context/vm_bypass.rs`
  (self-process passthrough), `daemon/arrangement/win32.rs` (restore + shell-surface guard),
  `settings/src/{app,main,persistence}.rs` (panes, collision validation, skip-persist,
  onboarding exercise + consent).
- **Verification:** `verify-story-5-settings-runtime.ps1` extended with real keyboard
  operation; a new elevated helper-window harness for Story 4.4 (already required by
  AC-4.4-005 and still absent).
- **No change** to: ring buffer, `u8` command contract, throttle, heartbeat, tray state
  machine, Task Scheduler auto-start, release profile.

---

## 3. Recommended Approach

### 3.1 Path evaluation (checklist section 4)

| Option | Viability | Effort | Risk | Assessment |
| --- | --- | --- | --- | --- |
| **1 — Direct Adjustment** (modify/add stories in place) | **Viable** | Medium | Low | Covers 11 of 12 items. Nothing needs redesigning; the contracts are right and the code diverges from them. |
| **2 — Rollback** | **Not viable** | — | — | Nothing recently completed is *wrong* in a way reverting would simplify. The applied 18 patches are verified clean. Rollback would discard correct work and resolve nothing. |
| **3 — PRD MVP Review** | **Viable and required, for exactly one item** | Low | Medium | NFR10 cannot be satisfied without abandoning NFR7/AR-04, which are load-bearing. The metric — not the architecture — is what must change. |

### 3.2 Selected path: **Hybrid (Option 1 + narrowly-scoped Option 3)**

Apply Option 1 to all items except NFR10, and apply Option 3 to NFR10 alone.

**Rationale.** Seven of the twelve items are not decisions at all — the binding artifact
already answers them, and the honest classification is "code diverges from an accepted
contract", which is ordinary `bmad-dev-story` work. Two items need a new AC, which is
routine backlog editing. Only NFR10 represents a requirement that cannot be met, and it
is met by changing the number rather than the system, because the number contradicts two
requirements that are themselves correct and deliberately chosen (live enumeration for
mouse-sync correctness; COM virtual-desktop queries for spatial isolation). Deliberately
**not** chosen: relaxing NFR7 to permit Z-order caching in order to reach sub-1 ms — that
would trade a real correctness property (desynchronisation with mouse interaction, the
documented reason for AD-3) for a number that no user can perceive.

### 3.3 MVP impact

**MVP scope is unchanged.** No FR is dropped, deferred, or reduced. One success *metric*
is restated to a value that is both achievable and still imperceptible to users. Every
P1 MUST feature (FR-1…FR-13, FR-16…FR-21) and both P2 SHOULD features (FR-14, FR-15)
remain in scope.

---

## 4. Detailed Change Proposals

Twelve items, ordered by consequence. Each states the category (A/B/C per §1.2), the
decision, the evidence, and the concrete edit.

### 4.1 NFR10 — replace an unachievable latency target `[Category C]`

**Decision:** Replace the sub-1 ms perceived-latency target, which is unachievable by
construction. **The replacement number is deliberately left open by this proposal.** The
measurement needed to justify one did not exist when this section was first drafted, and the
instrument capable of producing it was only built on 2026-08-02. Retain NFR4's binding
< 10 ms hook-callback budget unchanged, and keep the two distributions reported separately
as NFR10 already requires.

**Why the target must change rather than the system:** reaching < 1 ms requires caching
Z-order (prohibited by NFR7, for a stated correctness reason) or dropping the per-candidate
virtual-desktop query (prohibited by AR-04/AD-9). Both prohibitions are deliberate and
load-bearing. This part of the original analysis is untouched by everything below and stands.

---

**Revision 2026-08-02, after measurement. Three claims in the first draft were wrong.**

*(a) Every NFR10 figure on record measures a cycle that never moved focus.* The
`WM_APP_DEBUG_CYCLE_BURST` seam posts commands straight to the Worker, bypassing the keyboard
hook. Windows grants foreground rights to the process that received the last input event, so
a daemon driven that way is always denied: every `SetForegroundWindow` is refused and every
cycle terminates `Exhausted`. Re-measured on the current path, the counters read
`activated=0, exhausted=1000`. Both 17.5 ms and 27.6 ms are therefore the cost of enumeration
alone, not of rotating a window. The limitation was already stated in
`verify-scenario-two-windows.ps1`'s own docstring; what went unnoticed is that it invalidates
the figures this section was built on.

*(b) The focus-confirmation poll neither explains the regression nor needs accommodating.*
The first draft attributed the rise to the bounded poll added in `3a473d7` and wrote the
ceiling to "accommodate" it. The poll in fact never executed in any burst measurement, and on
the real path it exits on its first check because activation succeeds immediately — a
successful cycle pays no poll cost at all. The 17.5 → 27.6 ms rise is better explained by
`57d9fd1`'s cloak filter: one `DwmGetWindowAttribute` per enumerated window, across roughly
590 windows per cycle.

*(c) Synthetic input cannot stand in for a human.* Foreground rights follow the *injecting*
process, not the hook observer, so no `SendInput` harness reaches the activation path either.
Physical keystrokes are the only driver that works. A dedicated attempt with the shortcut
delivered through the hook confirmed this: `accepted=20` (the transport ran) but
`activated=0`.

**What is actually measured today.** From real keypresses, on the full hook → ring → Worker →
activation chain: **p50 29.5 ms, p95 47.1 ms, max 197 ms over 36 samples**, with
`accepted=36, activated=35`. That p95 sits 6% under the 50 ms the first draft proposed — not
the comfortable margin the burst figures implied. But 36 samples make the reported p95 the
second-largest observation, the run includes daemon cold start, and it is a debug build. It
is an indication, not a basis for a requirement.

**Why no threshold is set here.** Fixing 50 ms from a 36-sample debug run would repeat the
first draft's mistake in a new form. The number must follow the measurement, and the
measurement is possible for the first time:

- `[profile.release-metrics]` (added 2026-08-02) compiles the existing `debug_assertions`
  metric seams into release codegen, so NFR10 can finally be read off a build shaped like the
  one users run. Every prior figure came from an unoptimised debug binary — NFR10 had never
  been measured on the artifact NFR10 governs.
- Samples must come from **physical** keystrokes; there is no automated substitute.

The practical method is to run the `release-metrics` daemon during ordinary use and dump
`WM_APP_DEBUG_DUMP_CYCLE_METRICS` once a few hundred *activating* cycles have accumulated.
The dump reports `activated` alongside the percentiles, so a sample drawn from the wrong path
is visible rather than silent — the failure this revision exists to prevent.

**One structural finding the threshold must account for.** Cycle cost is two distributions,
not one, and they differ by roughly 10×. A cycle that activates costs about the enumeration
price. A cycle whose candidates all fail activation pays two ~20 ms polls *per candidate*
across every eligible candidate — 389 ms observed at worst. A single p95 hides that split.
NFR10 should either scope itself explicitly to the activating path or gate both.

**The four edits below are drafted but HELD.** Each carries `<P95>` where the threshold
belongs. None should land until the measurement described above exists. Landing them with a
guessed number would write an unjustified figure into `epics.md` and `prd.md` as an approved
requirement — materially harder to undo than leaving the amendment open a few days longer.
The false "accommodates the focus-confirmation wait" justification has been struck from
Edits 1 and 3; it was the clearest symptom of setting a number before measuring.

**Edit 1 — `epics.md` line 57**

```
OLD:
NFR10: Perceived end-to-end window-rotation latency targets less than 1 ms and must be
measured separately from the hook-callback budget.

NEW:
NFR10: Perceived end-to-end window-rotation latency must remain imperceptible, gated at
p95 below <P95> measured from Worker command receipt through activation completion, over a
sample in which activation actually occurred, with p50, p95, maximum, and the activated
count recorded separately from the hook-callback budget. The sub-1 ms figure originally
recorded here was unachievable alongside NFR7's prohibition on Z-order caching and AR-04's
per-candidate virtual-desktop query; it is replaced under the amendment clause of Story
2.6's performance gate.
```

**Edit 2 — `epics.md` line 586 (Story 2.6 performance gate)**

```
OLD:
**And** p95 remains below 1 ms for NFR10 to pass

NEW:
**And** p95 remains below <P95> for NFR10 to pass
**And** the measurement is taken on a build carrying release codegen, not a debug build
**And** the sample is driven by physical keystrokes through the keyboard hook, with the
reported activated count confirming focus actually moved
```

**Edit 3 — `prd.md` §8 Success Metrics, latency row**

```
OLD:
| Latensi Rotasi (*perceived*, end-to-end) | Sub-milidetik (< 1ms) — target persepsi
pengguna atas perpindahan fokus; **berbeda** dari *budget* eksekusi hook callback < 10ms
(Bab 4.1) | Tidak boleh menyembunyikan jendela *hang* demi kecepatan |

NEW:
| Latensi Rotasi (*perceived*, end-to-end) | Tak terasa oleh pengguna — p95 < <P95> diukur
dari penerimaan perintah oleh Worker hingga aktivasi selesai, pada sampel yang aktivasinya
benar-benar terjadi, dengan p50/p95/maksimum dan jumlah aktivasi dicatat; **berbeda** dari
*budget* eksekusi hook callback < 10ms (Bab 4.1). Target sub-milidetik semula tidak dapat
dicapai bersamaan dengan larangan cache Z-Order (Bab 4.2) dan query virtual-desktop per
kandidat; diganti via Sprint Change Proposal 2026-08-02. | Tidak boleh menyembunyikan
jendela *hang* demi kecepatan; tidak boleh meng-cache Z-Order demi kecepatan |
```

**Edit 4 — `verify-story-2-6-convergence.ps1` lines 36-37**

Necessary but **not sufficient**, and the first draft treated it as sufficient. Raising the
default stops the harness contradicting the requirement it enforces, but the harness still
cannot produce a valid NFR10 sample: it drives the burst seam, which bypasses the hook and
therefore never activates. Whatever number it reports is the enumeration-only path.

```
OLD:
    # NFR10: p95 below 1 ms.
    [int]$MaxP95Ns = 1000000,

NEW:
    # NFR10: p95 below <P95> (SCP 2026-08-02; supersedes the original 1 ms target).
    #
    # This harness CANNOT satisfy AC-2.6-005 on its own. The burst seam posts past the
    # keyboard hook, so the daemon never holds foreground rights and every cycle ends
    # `Exhausted`. Treat a pass here as evidence about enumeration cost only; the NFR10
    # verdict needs the release-metrics build driven by physical keystrokes.
    [int]$MaxP95Ns = <P95_NS>,
```

**Consequence:** Story 2.6's performance gate does **not** become passable yet — the first
draft claimed it would, on the strength of a figure now known to measure the wrong path.
What this section delivers instead is the removal of an impossible target, an honest account
of why the old numbers cannot support a new one, and a measurement method that exists. The
threshold lands in a follow-up once samples accumulate. Story 2.6's other blockers
(AC-2.6-005/006/008 soak reconciliation) are untouched and still gate it. Epics 3/4/5 remain
formally held by the latency figure until then — which is the honest position, and the one
the first draft was too quick to leave.

### 4.2 Daemon config reload — implement AD-5's missing half `[Category A]`

**Decision:** Implement in **Story 5.6**. Not a scope question — Story 5.6's own AC names
it. Story 5.7 retains ownership of `main.rs` entry-point wiring only.

**Evidence:** `epics.md:1283-1285` — "**When** the daemon receives `WM_APP_RELOAD_CONFIG`
/ **Then** it reads and validates the completed file exactly in response to that message
/ **And** uses no watcher, polling loop, or repeated idle wake-up." Reinforced by AD-5,
AR-05, SR-03, and the spine's declared-but-absent `daemon/config.rs`.

**Ownership resolution** (the one genuine ambiguity): `epics.md:1329` gives Story 5.7
exclusive ownership of "daemon Hook/Worker wiring". Split as follows — 5.6 owns the new
`daemon/config.rs`, the `WM_APP_RELOAD_CONFIG` message arm, and the control-plane
snapshot mechanism required by AC lines 1289-1292; 5.7 owns only the `main.rs` entry-point
integration. No AC edit needed; recorded here as the binding interpretation.

**Edit — story file `5-3-to-5-7-settings-ui-and-convergence.md`, Status by Story table**

```
OLD:
| 5.6 Daemon Settings bridge and first-run orchestration | partially complete |
daemon-side launch of `--onboarding` not wired; the daemon also never handles the
`WM_APP_RELOAD_CONFIG` it receives (see Review Findings, decision-needed) |

NEW:
| 5.6 Daemon Settings bridge and first-run orchestration | incomplete — AC not met |
The daemon never handles `WM_APP_RELOAD_CONFIG`: Settings posts it and reports "saved and
applied", but no handler exists and the message reaches `DefWindowProcW`. `daemon/config.rs`
is declared in ARCHITECTURE-SPINE's source tree and was never created. The earlier
"Settings→daemon direction is done" claim conflated *posted* with *handled* and is
withdrawn. Owned by 5.6 per SCP 2026-08-02 §4.2. |
```

Additionally, strike the phrase "Settings→daemon direction is done" from that file's
Dev Notes ("What Story 5.6 Is Missing") section.

### 4.3 Accessible descriptions — correct the story doc, not the code `[Category A]`

**Decision:** Accept role + accessible name + value + enabled state + checked/listening
state as the required exposure. Correct the story doc's overclaim. `.on_hover_text()`
remains as a mouse-only affordance and is not presented as an accessibility mechanism.

**Evidence:** `epics.md:1097` lists exactly "a stable role, accessible name, current
value, enabled state, and checked or listening state as applicable". FR-21: "mendeskripsikan
**status aktif/non-aktifnya**". UX-DR7: "expose meaningful **state**". No binding artifact
requires a description or HelpText. egui 0.35 exposes no generic description slot for
`Checkbox`/`Button`, and the review confirmed `on_hover_text` never touches accesskit —
but no accepted requirement asked for it.

**Edit — story file `5-2-accessible-native-theme-runtime-contract.md`, AC-5.2-002**

Replace the AC-5.2-002 annotation text with the epics.md wording (role / accessible name /
current value / enabled state / checked-or-listening state), and record that the
"descriptions attached via `WidgetInfo`" phrasing was a paraphrase error introduced in the
story file and never present in `epics.md`, FR-21, or UX-DR7.

**One item does require verification, not a decision:** `epics.md:1098` — "**And**
Listening mode is not communicated through visual text alone." The capture button's label
changes to "Listening…", which does reach UI Automation as the element's *accessible name*.
Whether that satisfies the AC must be confirmed by the harness extension in §4.10, not
assumed.

### 4.4 Maximized window not restored before `SetWindowPos` `[Category B]`

**Decision:** Add an AC to Story 4.4 and implement via an **asynchronous** restore —
`PostMessageW(hwnd, WM_SYSCOMMAND, SC_RESTORE, 0)` — followed by the existing
`SetWindowPos` with `SWP_ASYNCWINDOWPOS`. Neither call blocks on the target thread, so
AC-4.4-004's "no blocking cross-process call" holds and the hazard that `SWP_ASYNCWINDOWPOS`
exists to prevent is not reintroduced.

**Why not the options the review offered:** it framed the choice as accept-bounded-blocking
(`ShowWindow(SW_RESTORE)`), find an async workaround, or scope the case out. The async
workaround exists and was not identified: `PostMessage` places `SC_RESTORE` in the target's
own message queue, and `SWP_ASYNCWINDOWPOS` likewise posts rather than sends, so both are
queued to the same thread in submission order. A hung target simply never drains either —
identical to today's behaviour, with no new blocking edge.

**Explicitly checked for conflict:** Story 2.6 AC (`epics.md:580`) prohibits any "restore,
or maximize action" — that gate governs the **cycling** path only. The arrangement path has
no equivalent prohibition, so this does not violate AC-2.6-004.

**Honest caveat:** the queue-ordering argument is sound in principle but **unverified on a
real window**. It must be proven by the elevated helper-window test that AC-4.4-005 already
requires and that still does not exist. If that test shows the placement racing the restore,
the documented fallback is to scope the already-maximized case out of Story 4.4 with the
limitation recorded — not to adopt a blocking restore.

**Edit — `epics.md`, new AC block in Story 4.4 after line 964**

```
NEW:
**Given** the foreground window is maximized
**When** an arrangement command targets it
**Then** the maximized state is cleared through an asynchronous, non-blocking request
before placement is applied
**And** no synchronous cross-process call is used to perform the restore
**And** the elevated helper-window test demonstrates the final geometry matches the plan
rather than the maximized bounds.
```

### 4.5 Shell-surface foreground guard — ownership `[Category A]`

**Decision:** **Story 4.4** owns it, as a local self-contained guard inside
`arrangement/win32.rs`. No AC edit required.

**Evidence:** AC-4.4-001 (`epics.md:944-948`) assigns foreground-window resolution to the
adapter: "**When** platform context is requested / **Then** the adapter resolves the
foreground window…". Story 4.5's ACs concern the Hook→Worker pipeline and planner
invocation, not foreground validation. Precedent inside the same file and same review
cycle: the DWM-cloak filter was placed locally in `win32.rs` rather than reaching into
`cycling::source`, deliberately preserving story isolation. `Progman`/`WorkerW`/
`Shell_TrayWnd` filtering follows that established shape.

### 4.6 Cross-field shortcut collision `[Category B]`

**Decision:** Hard-block duplicates at both capture time and Save time, with the
accessible corrective microcopy path already wired by the applied patches. Add an AC to
Story 5.3.

**Rationale for hard-block over warn-but-allow:** two commands bound to one chord is never
a useful configuration — the daemon's `match_shortcut` resolves one deterministically and
the other becomes silently unreachable. A user cannot diagnose that from the UI. FR-6's
precise-matching principle implies one chord resolves to one command.

**Edit — `epics.md`, new AC block in Story 5.3 after line 1173**

```
NEW:
**Given** a shortcut value already bound to another field is captured or present in the draft
**When** capture completes or Save is attempted
**Then** the duplicate is rejected before persistence
**And** the conflicting field is identified in neutral, direct, solution-oriented microcopy
**And** the rejection is exposed through the accepted accessibility adapter.
```

### 4.7 Shortcut passthrough while capture is active `[Category A — blocking §4.8]`

**Decision (revised after stakeholder review):** Suspend hook matching **only while a
shortcut field is actively Listening, or while the onboarding practice step is active** —
not for the entire time `wintick-settings.exe` holds the foreground. Signalled from
Settings through a new `WM_APP_CAPTURE_STATE` control message. AD-6 (VM/RDP bypass) is
**not** widened and its semantics remain untouched.

**Superseded proposal.** An earlier draft of this section bypassed the hook whenever
`wintick-settings.exe` was foreground, on the grounds that foreground-based bypass carries
no state that can become stuck. That is broader than the actual requirement and was
correctly rejected: it would kill snapping and stacking of the Settings window itself, for
the entire session, to solve a problem that only exists during the brief moment a field is
listening.

**Ownership:** Story 5.4 owns the Settings-side signal for capture; Story 5.5 owns it for
the onboarding practice step; Story 5.6 owns the daemon-side handler (consistent with its
settings-bridge ownership, and required because Story 5.4's AC at `epics.md:1221` mandates
independent verification *without a daemon process*). `WM_APP_CAPTURE_STATE` is an
additive `shared::constants` entry, following the precedent of `ONBOARDING_FLAG`.

**No architecture amendment required.** AD-1 already sanctions "Win32 Window Messages
(settings→daemon)" as a permitted cross-actor channel. Only the Consistency Conventions
table gains a row documenting the new message (§4.7 edit below).

**Stuck-state elimination.** The earlier objection to an IPC signal — Settings dies while
suspended, hook stays suspended — is removed by making suspension *conditional* rather
than latched. All three conditions must hold on every evaluation, and the check runs inside
the existing allocation-free foreground-identity path:

1. the capture flag is set, **and**
2. the foreground window still belongs to `wintick-settings.exe`, **and**
3. a bounded timeout since the last capture signal has not elapsed.

Any one failing restores normal matching immediately. A crashed or backgrounded Settings
process therefore self-heals with no recovery message and no daemon-side timer to leak.

**What the user keeps.** Outside Listening, every WinTick shortcut behaves normally while
Settings is foreground — including snapping and stacking the Settings window itself.

**Behaviour worth stating precisely, because it is not what it appears:** the cycling
shortcut is a no-op while Settings is foreground *regardless of this decision*. Cycling is
same-application by construction (FR-1/AD-4, identity by executable basename), and
`cycling/selection.rs` filters the active window out of the candidate set rather than
skipping it in-loop. With a single `wintick-settings.exe` window the candidate set is
therefore empty. The practical benefit of the narrow scope is confined to the snap and
stack commands, which do act on the Settings window. (Two concurrent
`wintick-settings.exe` processes — Settings plus onboarding — would be "same application"
under AD-4 and would cycle between each other; an edge case, not a design goal.)

**Onboarding is the accepted exception.** During the first-run practice step the shortcut
must reach the onboarding window rather than the desktop, so suspension there is intended,
scoped to the practice step, and ends when onboarding completes.

**Edit — `ARCHITECTURE-SPINE.md`, Consistency Conventions table, new row after "Data format (IPC reload)"**

```
NEW:
| **Data format (IPC capture state)** | Custom Win32 message `WM_APP_CAPTURE_STATE`.
Settings signals that a shortcut field is listening (or that the onboarding practice step
is active) so the Hook Thread stops matching its own shortcuts long enough for the
combination to be captured. Suspension is conditional, never latched: it lapses as soon as
the signalling process stops owning the foreground or a bounded timeout elapses, so a
terminated Settings process cannot leave the hook suspended. AD-6's VM/RDP bypass list is
a separate mechanism and is not involved. |
```

### 4.8 Onboarding interactive practice exercise `[Category A]`

**Decision:** Implement in **Story 5.5**. **Sequenced after §4.7.** No scope reduction.

**Evidence — four independent binding sources:** FR-17 (P1 MUST, names the dummy window
explicitly); UX-DR9; `epics.md:1233` and `:1237-1241`; `EXPERIENCE.md` Flow 1 step 4.
Descoping this would require amending all four.

**Ordering constraint (new, not in the review):** `epics.md:1234` forbids a second global
hook, so the practice exercise can only receive `Win+Backtick` as ordinary window input —
which requires §4.7's capture-state suspension. Story 5.5 cannot be verified end-to-end
before Story 5.6 delivers the daemon-side handler.

### 4.9 Onboarding auto-start consent, Skip Tutorial persistence, pane grouping `[Category A ×3]`

All three are already dictated by an existing AC. Implement; no AC edits.

| Item | Binding AC | Current behaviour | Fix |
| --- | --- | --- | --- |
| Auto-start consent | `epics.md:1254-1257` — "onboarding offers Start with Windows … `general.auto_start` reflects the explicit choice … never enabled silently" | No toggle in any onboarding step; `finish_onboarding()` persists the `false` default without asking | Add the consent control to the onboarding completion step (Story 5.5) |
| Skip Tutorial persistence | `epics.md:1248-1252` — "**Then** onboarding closes through the normal completion path **And** a valid default configuration is persisted **And** the next daemon start does not reopen onboarding solely because it was skipped" | `skip_onboarding()` only advances to `Done`; persistence needs a second "Finish" click, so closing the window leaves no `config.toml` and onboarding reopens | Make Skip persist and complete in one action (Story 5.5). This is a plain AC violation, not a decision. |
| Pane grouping | `epics.md:1143` + UX-DR3 + `EXPERIENCE.md` IA — "feature-oriented groups for Core Switcher, Window Snapping, Stack Layout" | `General`/`Shortcuts`/`Layout`/`About`, with all six shortcut fields flattened into one undifferentiated Shortcuts tab | Regroup by owning feature area (Story 5.3) |

### 4.10 Keyboard operation — raise the verification bar `[Category A]`

**Decision:** Extend `verify-story-5-settings-runtime.ps1` to drive real `Tab` /
`Shift+Tab` / `Space` / `Enter` / `Escape` input and assert that focus moves in the
declared order and that activation and cancellation actually occur. Structural verification
alone is insufficient.

**Evidence:** `epics.md:1103` — "**And** controls can be reached and **operated** without a
mouse". `epics.md:1122-1125` — "**When** any required role, state, **keyboard operation**,
or theme behavior **cannot be demonstrated** / **Then** Story 5.2 is not accepted". The AC
names demonstration as the gate, so trusting egui's built-in traversal is not an option
the AC permits.

**Secondary benefit:** this harness extension empirically settles two findings currently
filed as `defer` — whether `Enter`/`Space` and `Tab` are capturable as shortcut main keys.
Both were deferred *pending runtime verification*; this is that verification.

---

## 5. Implementation Handoff

### 5.1 Scope classification: **Moderate**, with one Major element

- **Major:** §4.1 (NFR10) alters the PRD and a success metric → Product Manager /
  Architect sign-off.
- **Moderate:** §4.4 and §4.6 add ACs to `epics.md`; §4.2 and §4.7 record binding
  interpretations and add one Consistency Conventions row → backlog reorganisation.
- **Minor:** the remaining items are ordinary `bmad-dev-story` execution against
  unchanged ACs.

### 5.2 Routing

| Recipient | Responsibility | Items |
| --- | --- | --- |
| **PM / Architect** | Approve the NFR10 replacement and the new capture-state control message | §4.1, §4.7 |
| **PO / Dev** | Apply `epics.md` AC additions and story-doc corrections | §4.2, §4.3, §4.4, §4.6 |
| **Dev (`bmad-dev-story`)** | Implement, one story at a time | §4.2, §4.4, §4.5, §4.6, §4.7, §4.8, §4.9, §4.10 |
| **Dev (harness)** | New elevated helper-window harness for 4.4; extend the Settings harness | §4.4, §4.10 |

### 5.3 Execution sequence

Ordering is constrained by two real dependencies, not preference.

1. **§4.1** — ~~unblocks Story 2.6's gate~~ **[revised 2026-08-02]** No longer step 1 and no
   longer unblocking: the threshold is held pending measurement, so Epics 3/4/5 stay formally
   held by the latency figure. Removing the impossible target is done; setting the
   replacement waits on `release-metrics` samples. Nothing else in this list depends on it.
2. **§4.2** — Story 5.6 reload handler + `daemon/config.rs`. **DONE 2026-08-02** (`0e81cfb`).
   12 desktop-free tests; both accept and reject paths verified at runtime. Settings now
   genuinely applies. Remainder of Story 5.6 is §4.7 plus the 5.7 no-restart AC.
3. **§4.7** — capture-state suspension: daemon-side handler (5.6), then the Settings-side
   signal in 5.4. **Must precede step 4.** The only remaining item still carrying an
   architecture decision (`WM_APP_CAPTURE_STATE`, new Consistency Conventions row).
4. **§4.8 + §4.9** — Story 5.5 practice exercise, consent, Skip persistence. Depends on 3.
5. **§4.9** pane grouping + **§4.6** collision validation — Story 5.3. Independent of 3–4,
   and therefore the natural next unit of work.
6. **§4.10** — harness extension; also closes the two deferred capture findings.
7. **§4.4 + §4.5** — Story 4.4 restore + shell-surface guard, gated on the new elevated
   helper-window harness.

### 5.4 Success criteria

- All six stories reach `review` with **zero** open `decision-needed` findings.
- **[revised 2026-08-02]** NFR10 is measured on a `release-metrics` build driven by physical
  keystrokes, with p50/p95/max **and the activated count** recorded, reported separately from
  hook-callback timing, over a sample where activation actually occurred. Neither the 17.5 ms
  figure nor any `verify-story-2-6-convergence.ps1` result is carried forward: that harness
  bypasses the hook and measures cycles that never move focus.
- `WM_APP_RELOAD_CONFIG` demonstrably changes live daemon behaviour — a captured shortcut
  becomes active with no daemon restart (Story 5.7 AC, `epics.md:1342`).
- The Settings harness demonstrates real keyboard operation, not structural presence.
- Story 4.4's elevated helper-window test exists and exercises a maximized target.
- No regression in: hook callback < 10 ms, ring capacity, throttle, heartbeat, tray state
  precedence, idle CPU, binary size.

### 5.5 Items deliberately left out of scope

- Stories 3.2 / 3.4 / 4.5 remain `in-progress` on their own missing runtime matrices
  (COM lifecycle, multi-monitor, helper-window placement). Unrelated to these findings.
- The 10 `defer`-tagged findings from the review keep their deferred status, except the two
  capture-key items that §4.10 incidentally resolves.
- Story 2.6's AC-2.6-005/006/008 soak reconciliation is untouched; §4.1 addresses the
  latency gate only, and Story 2.6 remains blocked on the soak until that is separately closed.

---

## 6. Checklist Record

| § | Item | Status |
| --- | --- | --- |
| 1.1 | Triggering story identified | [x] Done — 4.4, 5.2, 5.3–5.6 review; plus 2.6 (NFR10) |
| 1.2 | Core problem categorised | [x] Done — three categories, not one; 7 mislabelled |
| 1.3 | Evidence gathered | [x] Done — grep, measured percentiles, AC line refs |
| 2.1 | Current epic completable | [x] Done — yes for all, with edits |
| 2.2 | Epic-level changes | [x] Done — 2 AC additions, 3 amendments, 0 epics added/removed |
| 2.3 | Remaining epics reviewed | [x] Done — Epic 3 unblocked as a consequence |
| 2.4 | Epics invalidated / new needed | [N/A] None |
| 2.5 | Epic order / priority | [!] Action-needed — 5.4 must now precede 5.5 (§4.7/§4.8) |
| 3.1 | PRD conflicts | [!] Action-needed — §8 latency metric (§4.1). Edit **held**: threshold pending measurement (revision 2026-08-02) |
| 3.2 | Architecture conflicts | [x] AD-5 **resolved** 2026-08-02 (`0e81cfb`) — `daemon/config.rs` created, `WM_APP_RELOAD_CONFIG` handled. [!] One conventions row still to add (§4.7). AD-6 deliberately untouched |
| 3.3 | UI/UX conflicts | [!] Action-needed — code diverges from EXPERIENCE IA (§4.9) |
| 3.4 | Other artifacts | [!] Action-needed — 2 harnesses (§4.4, §4.10) |
| 4.1 | Option 1 Direct Adjustment | [x] Viable — Medium effort, Low risk |
| 4.2 | Option 2 Rollback | [x] Not viable — nothing wrong to revert |
| 4.3 | Option 3 MVP Review | [x] Viable — required for NFR10 only |
| 4.4 | Path selected | [x] Done — Hybrid (1 + narrow 3) |
| 5.1–5.5 | Proposal components | [x] Done — §1–§5 |
| 6.1–6.2 | Review and accuracy | [x] Done. **Revised 2026-08-02** after measurement: three claims in §4.1 were wrong and are corrected in place, with the delta recorded rather than silently rewritten |
| 6.3 | User approval | [~] **Partial.** §4.2 was authorised and executed directly (`0e81cfb`). §4.1 is explicitly **not** approved — its edits are held pending measurement. §4.3–§4.10 remain unapproved |
| 6.4 | `sprint-status.yaml` update | [x] Done 2026-08-02 — annotated with §4.2's completion and the NFR10 revision. No epic or story added or removed |
| 6.5 | Handoff confirmed | [~] Partial — §4.2 handed off and delivered. Next unit of work is Story 5.3 (§4.6 + §4.9), the only remaining item with no dependency and no open decision |

---
type: uc
id: UC-8
component: settings
satisfies: [FR-25]
critical: false
created: '2026-09-03'
---

# UC-8 — Check for updates from the About pane

## Trigger

User clicks "Check for updates" in the About pane.

## Precondition

- Settings is open and the daemon is running (`daemon_watch`).
- The "Check for updates automatically" toggle governs only the periodic background check (CAP-13); the manual button in this use case is always available regardless of the toggle's state.

## Main Flow

1. User opens the About pane and clicks "Check for updates". The button shows "Checking…" and disables itself.
2. System requests the release descriptor over HTTPS, carrying no payload beyond the request itself (BR-8).
3. System compares the descriptor's version against the running version and validates the descriptor (checksum shape, release URL pinned to this product's own channel).
4. A newer, valid release exists: the About pane shows the new version and an "Install" action.
5. User clicks "Install". System downloads the installer to a fresh, unpredictable temporary directory.
6. System verifies the download's SHA-256 checksum against the one the descriptor published.
7. On a match, system launches the verified installer elevated and Settings exits — the installer stops the daemon and replaces `wiradesk-settings.exe`, which cannot happen while it is running.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| Step 3 | No newer release exists | About pane shows the product is up to date; no further action offered. |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| Step 2 | Request fails (network, non-HTTPS target, oversized or unreadable response) | System shows a specific, worded reason (not a generic "update failed") | Button re-enables; user may try again |
| Step 3 | The descriptor names an unusable version, a malformed checksum, or a download URL not pinned to this product's own release channel | System refuses the release and reports why, offering nothing to install | No install offered; the descriptor is treated as untrustworthy, not as "no update" |
| Step 6 | Downloaded file's checksum does not match the published hash | System deletes the file and never launches it | Nothing is installed; user is told the verification failed |

## Outcome

The user learns whether a newer release exists and, on confirmation, ends up running it — with every step verified before anything installed runs, and nothing sent over the network but the request itself.

## Business Rules

- `BR-8`

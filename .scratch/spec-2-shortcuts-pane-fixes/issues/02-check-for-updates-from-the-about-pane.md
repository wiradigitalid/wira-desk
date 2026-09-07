---
id: SPEC-2-02
component: settings
satisfies: [UC-8]
blocked_by: []
status: done
tests:
  - update::tests::a_checksum_that_is_not_sha256_is_refused
  - update::tests::a_download_url_off_this_repository_is_refused
  - update::tests::the_real_workflow_url_shape_is_accepted
  - update::tests::a_newer_version_is_offered
  - update::tests::an_older_version_is_never_offered
  - updatecheck::tests::a_version_is_announced_once_however_often_it_is_found
  - updatecheck::tests::a_run_of_failures_announces_once
  - updatecheck::tests::a_success_ends_the_failure_run
---

# 02: Check for updates from the About pane

**Backfill, not new work.** `UC-8`/`FR-25` shipped with the product — the About pane's "Check for
Updates" control (`crates/settings/ui/panes/about_pane.slint`), the descriptor validation and
version-comparison logic (`crates/shared/src/update.rs`), the download/checksum-verify/launch
pipeline (`crates/settings/src/update.rs`), and the daemon-side announce/dedup state machine
(`crates/daemon/src/updatecheck.rs`) — but no ticket was ever opened for it, so `uc-scheduled`
reports it red now that `settings` has tickets. Same reasoning as `SPEC-1`'s `W1-S4..S6` backfill:
this documents real, already-delivered, already-tested work against a real gap in the tracking
record: it does not fabricate history, and it adds no behavior.

- [x] Requesting an update check from the About pane validates the fetched descriptor's version,
      checksum shape, and download URL before ever treating it as real
      (`update::tests::a_checksum_that_is_not_sha256_is_refused`,
      `update::tests::a_download_url_off_this_repository_is_refused`,
      `update::tests::the_real_workflow_url_shape_is_accepted`).
- [x] A newer published version is offered; the current or an older one is not
      (`update::tests::a_newer_version_is_offered`, `update::tests::an_older_version_is_never_offered`).
- [x] A given version is announced once, not once per poll, and a run of check failures announces
      once rather than repeatedly (`updatecheck::tests::a_version_is_announced_once_however_often_it_is_found`,
      `updatecheck::tests::a_run_of_failures_announces_once`, `updatecheck::tests::a_success_ends_the_failure_run`).
- [x] Confirming the offered update downloads the installer over HTTPS from this project's own
      releases, verifies it against the published SHA-256, and only launches it elevated on a match
      (`crates/settings/src/update.rs`; live end-to-end path covered by the ignored,
      manually-invoked `update::tests::the_real_installer_downloads_and_verifies`, per its own
      doc comment on why it cannot run unattended).
- [x] Full test suite green — confirmed by scoped runs of every test named above
      (`cargo test --workspace update:: updatecheck::`, all passing) before this ticket closed;
      `wdi-build`'s own full-suite pass for `SPEC-2` covers the rest.

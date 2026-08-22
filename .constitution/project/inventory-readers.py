"""inventory readers — how THIS product's code is read. Owned by the product, not the method.

Written for Wira Desk at wave W1. The engine
(`.constitution/method/scripts/inventory.py`) is the method's and is replaced on every update;
this file is the product's and is never written over.

WHAT THIS STACK ACTUALLY IS, and why the three readers look nothing like a web app's:

Wira Desk is two Win32 binaries and a shared crate. There is no database, no HTTP server, and no
router. Forcing those three shapes onto this product would produce three empty inventories, which
would read as "nothing here" rather than "nothing of that kind here". So each reader is pointed at
the thing that genuinely plays that role:

    db      the TOML configuration schema. `crates/shared/src/config.rs` defines one struct per
            `[section]`, and those sections are the only structured, persisted, keyed data this
            product owns. Serde is the schema.
    api     the Win32 message surface. `WM_APP_*` in `crates/shared/src/constants.rs` is the whole
            inter-process and inter-thread contract; it is what another process can send us.
    screen  the egui panes and the onboarding steps in `crates/settings/src/app.rs`, which are
            enums, plus nothing else — the tray menu and the startup `MessageBox` are Win32 shell
            surfaces, not routed screens, and they are reported as unread rather than invented.

THE RULE THAT NO STACK CHANGES, applied here: anything a pattern cannot read goes to `unread`. Two
things deliberately go there every run — the diagnostic log and the legacy migration source are
persisted files with no schema to read, and a use case cannot be derived from code at all. They are
named so a reader sees the boundary of what was machine-read.
"""

from __future__ import annotations

import re
from pathlib import Path

# The engine refuses to run while this is set. The readers below read real files, so it is gone.


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


# --------------------------------------------------------------------------- db

# `pub struct NameConfig {` ... up to the closing brace at column 0.
_STRUCT = re.compile(r"^pub struct (\w+Config)\s*\{(.*?)^\}", re.S | re.M)
_FIELD = re.compile(r"^\s*pub (\w+):\s*([^,\n]+),", re.M)
# The `Config` root maps field name -> section name; each field is one `[section]` in the TOML.
_ROOT = re.compile(r"^pub struct Config\s*\{(.*?)^\}", re.S | re.M)


def derive_db(root: Path) -> "Derived":       # noqa: F821 — injected by the engine
    """Every table this product stores — here, every `[section]` of the TOML config schema."""
    src = root / "crates/shared/src/config.rs"
    text = read(src)
    if not text:
        return Derived(unread=[f"{src.as_posix()} could not be read"])  # noqa: F821

    rel = src.relative_to(root).as_posix()
    structs = {name: body for name, body in _STRUCT.findall(text)}

    # Section name comes from the root struct's field names, not from the struct names: the TOML
    # key is what is actually persisted, and `vm_bypass` is not `VmBypassConfig` lowercased.
    root_match = _ROOT.search(text)
    sections: list[tuple[str, str]] = []
    if root_match:
        for field, ty in _FIELD.findall(root_match.group(1)):
            sections.append((field, ty.strip()))

    rows, unread = [], []
    for section, ty in sections:
        body = structs.get(ty)
        if body is None:
            unread.append(f"{rel}: `[{section}]` is typed `{ty}`, whose struct was not found here")
            continue
        cols = [f"`{name}`" for name, _ in _FIELD.findall(body)]
        if not cols:
            unread.append(f"{rel}: `[{section}]` has no readable `pub` fields")
            continue
        # The key MUST be exactly what the engine's `plan_keys` derives from the first cell --
        # for `db` that is `cells[0].strip("`")`. Any other shape makes every row read as both
        # "planned but not in code" and "in code but not planned", which is how a reader reports
        # ten findings about a file that agrees with it perfectly.
        rows.append(Row(                                                  # noqa: F821
            key=f"config.toml [{section}]",
            cells=[
                f"`config.toml [{section}]`",
                "`_platform`",
                f"Persisted user configuration for {section.replace('_', ' ')}",
                ", ".join(cols),
                "active",
            ],
            source=rel,
        ))

    if root_match is None:
        unread.append(f"{rel}: no root `pub struct Config` found, so no section list could be read")

    # Persisted, but not schema. Named rather than dropped, and not invented into rows.
    unread.append(
        "`%APPDATA%\\WiraDesk\\wiradesk.log` is persisted but has no schema to read — it is "
        "append-only formatted text written by `crates/daemon/src/log.rs`"
    )
    unread.append(
        "`%APPDATA%\\WinTick\\config.toml` is read once by `crates/shared/src/migrate.rs` for "
        "one-time migration. It is a legacy shape this product does not define, so its columns "
        "are deliberately not derived"
    )
    return Derived(rows=rows, unread=unread)                              # noqa: F821


# -------------------------------------------------------------------------- api

_CONST = re.compile(r"^pub const (WM_APP_\w+):\s*u32\s*=\s*WM_APP\s*\+\s*(\d+);", re.M)


def derive_api(root: Path) -> "Derived":      # noqa: F821 — injected by the engine
    """Every endpoint this product serves — here, every Win32 `WM_APP_*` message it accepts."""
    src = root / "crates/shared/src/constants.rs"
    text = read(src)
    if not text:
        return Derived(unread=[f"{src.as_posix()} could not be read"])    # noqa: F821

    rel = src.relative_to(root).as_posix()
    lines = text.splitlines()

    # Which crate references a constant decides the owning component. Read, not assumed: a message
    # named for the hook may well be posted by the settings binary.
    def refs(name: str) -> set[str]:
        out = set()
        for crate, pc in (("daemon", "window-management"), ("settings", "settings")):
            base = root / "crates" / crate / "src"
            for f in base.rglob("*.rs"):
                if re.search(rf"\b{name}\b", read(f)):
                    out.add(pc)
                    break
        return out

    rows, unread = [], []
    for match in _CONST.finditer(text):
        name, offset = match.group(1), int(match.group(2))
        # The doc comment immediately above the constant is its description.
        idx = text[: match.start()].count("\n")
        doc = []
        for line in reversed(lines[:idx]):
            stripped = line.strip()
            if stripped.startswith("///"):
                doc.append(stripped[3:].strip())
            elif stripped.startswith("//") or not stripped:
                if doc:
                    break
                continue
            else:
                break
        description = " ".join(reversed(doc)).strip() or "(no doc comment on the constant)"

        owners = refs(name)
        # A debug-only message is not part of the shipped surface, and saying so is the point.
        debug = "_DEBUG_" in name
        rows.append(Row(                                                  # noqa: F821
            key=f"win32-message POST {name}",
            cells=[
                "win32-message",
                "POST",
                f"`{name}`",
                ", ".join(f"`{o}`" for o in sorted(owners)) or "`_platform`",
                f"`WM_APP + {offset}` (0x{0x8000 + offset:04X}). {description}",
                "debug-only" if debug else "active",
            ],
            source=rel,
        ))

    if not rows:
        unread.append(f"{rel}: no `pub const WM_APP_* : u32 = WM_APP + N;` declarations matched")
    unread.append(
        "The ring buffer (`crates/daemon/src/ring.rs`) is an in-process channel, not a message "
        "another process can send, so it is not an endpoint and is not listed here"
    )
    return Derived(rows=rows, unread=unread)                              # noqa: F821


# ----------------------------------------------------------------------- screen

_ENUM = re.compile(r"^pub enum (\w+)\s*\{(.*?)^\}", re.S | re.M)
_VARIANT = re.compile(r"^\s*(\w+),", re.M)


def derive_screen(root: Path) -> "Derived":   # noqa: F821 — injected by the engine
    """Every screen this product renders — here, the settings panes and the onboarding steps."""
    src = root / "crates/settings/src/app.rs"
    text = read(src)
    if not text:
        return Derived(unread=[f"{src.as_posix()} could not be read"])    # noqa: F821

    rel = src.relative_to(root).as_posix()
    enums = {name: body for name, body in _ENUM.findall(text)}

    inv = root / ".how/_platform/inventory-screen.md"
    states, _platform = decisions(inv)                                    # noqa: F821

    def snake(name: str) -> str:
        return re.sub(r"(?<!^)(?=[A-Z])", "-", name).lower()

    rows, unread = [], []
    for enum_name, spa in (("Pane", "settings"), ("OnboardingStep", "onboarding")):
        body = enums.get(enum_name)
        if body is None:
            unread.append(f"{rel}: `pub enum {enum_name}` not found, so its screens were not read")
            continue
        for variant in _VARIANT.findall(body):
            route = f"/{snake(variant)}"
            rows.append(Row(                                              # noqa: F821
                key=f"{spa}:{route}",
                cells=[
                    f"`{spa}/{variant}`",
                    f"`{route}`",
                    ", ".join(sorted(k for k, v in states.items() if v == route)) or "—",
                    "`settings`",
                    "",   # a UC cannot be derived from code; declared in the artifact, not guessed
                ],
                source=rel,
            ))

    unread.append(
        "The UC each screen serves is not derivable from code. It is a promise, and it is "
        "declared in `.how/_platform/inventory-screen.md` rather than pattern-matched here"
    )
    unread.append(
        "The tray context menu (`crates/daemon/src/menu.rs`) and the Tier-1 startup `MessageBox` "
        "(`crates/daemon/src/error.rs`) are Win32 shell surfaces with no route. They are real UI "
        "and are recorded in the artifact, but they are not screens this reader can derive"
    )
    return Derived(rows=rows, unread=unread)                              # noqa: F821

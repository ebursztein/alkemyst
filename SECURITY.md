# Security

Report vulnerabilities privately through GitHub's private vulnerability reporting
for this repository. Do not attach raw Arena logs or credentials to public issues.

Alkemyst stores games locally and authenticates loopback API/MCP access. Only
sanitized, minimal diagnostic data should be shared. Dependencies are checked by
OSV-Scanner on changes, releases and weekly. There is no stable release yet.

## Reviewed desktop dependency exceptions

The supported builds are `aarch64-apple-darwin` and `x86_64-pc-windows-msvc`.
Linux is not a supported build or distribution target. Tauri's lockfile nevertheless
contains its Linux GTK and D-Bus dependencies. Their current advisories are reviewed
in [security-exceptions.json](security-exceptions.json), owned by **ebursztein** and
expiring **2026-11-05 at 00:00 UTC**:

| Exact crate version      | Advisory                                                                                              | Reason and removal condition                                                                                             |
| ------------------------ | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `glib` 0.18.5            | [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), alias GHSA-wrw7-89jp-8q8g | Iterator unsoundness; fixed in GLib 0.20. GTK 0.18 requires GLib 0.18. Remove when Tauri's GTK family can adopt the fix. |
| `proc-macro-error` 1.0.4 | [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html)                            | Unmaintained macro dependency of the Linux GTK family; replace with that family.                                         |
| `zbus_names` 4.3.5       | [RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html)                            | Standalone crate merged into zbus 6; Linux single-instance plugin requires zbus 5.                                       |
| `zvariant` 5.15.1        | [RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html)                            | Standalone crate merged into zbus 6; migrate with the single-instance plugin.                                            |
| `zvariant_derive` 5.15.1 | [RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html)                            | Standalone macro crate merged into zbus 6; migrate with the single-instance plugin.                                      |

These are target-exclusion exceptions, not assertions that the affected crates are
safe. Each security run generates both target dependency graphs with all workspace
features enabled. An exception fails if its exact crate becomes reachable in either
graph, its package/version/advisory changes, or it expires. New targets require a
policy review; Linux support must resolve these dependencies first.

The pinned OSV scanner first scans **both complete lockfiles without ignores**.
The policy rejects every finding except the exact reviewed tuples above. A second
native OSV run applies only those reviewed advisory IDs with expiration dates and
must return exit zero. Scanner/download/checksum/network errors fail the gate;
a findings exit status is never sufficient for success. Both reports include all
packages, and the lockfiles must remain unchanged throughout the review.

CI publishes the raw report, reviewed report, accepted-advisory list, and dependency
graphs as security artifacts. The console explicitly reports the exception count.
Exceptions have a maximum 30-day horizon, cannot renew themselves, and must be
removed when no longer necessary. Sprinty S05-008 records the initial review.

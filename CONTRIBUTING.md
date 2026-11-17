# Contributing

Thanks for helping. A few things make reviews quick:

- **Read the map first.** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) explains how the pieces fit,
  and [docs/MODULES.md](docs/MODULES.md) shows how to add a feature or a format.
- **Tests come with changes.** `cargo test --workspace` must pass, and `cargo clippy --workspace
  --all-targets` must be clean. Front-end changes need `pnpm test` and `pnpm check` in
  `apps/desktop`.
- **Keep files' licence headers.** New source files start with
  `// SPDX-License-Identifier: AGPL-3.0-or-later` and `// Copyright (C) 2026 Sean Hawkey`.
- **Plain words in the interface.** Labels say what things do; errors say what went wrong and how
  to fix it.

## Contributor Licence Agreement

By opening a pull request you agree to [CLA.md](CLA.md). It lets me offer Wrangler under other
licences in future (for example a paid app-store build) while every open-source release stays
open. You keep the copyright in your own work. The project's licence will never change for
releases that already exist.

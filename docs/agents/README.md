# Agent documentation

This folder holds context documents for agents. Treat every `index.md` as a jump point — only open the linked files that are relevant to the current task.

| If you want to know... | Go to |
| --- | --- |
| Root index | [../AGENTS.md](../AGENTS.md) |
| Runtime architecture | [architecture/index.md](architecture/index.md) |
| Development and testing | [development/index.md](development/index.md) |
| Release and installers | [release/index.md](release/index.md) |
| Maintenance rules | [maintenance/index.md](maintenance/index.md) |

## Maintenance note

When adding new context:

1. Put detail in the relevant `docs/agents/{category}/` subfile.
2. If the category does not exist, create the folder and an `index.md`.
3. Link the new subfile from the category `index.md`.
4. If it is a new top-level category, add a row to `AGENTS.md` and this table.
5. Never paste long details directly into `AGENTS.md`.
6. Any new document under `docs/agents/` must follow the same index style: start with a short “when to read this” description, use an index table when it covers multiple subtopics, and keep long details in linked subfiles.


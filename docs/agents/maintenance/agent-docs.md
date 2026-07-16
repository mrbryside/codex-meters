# Updating agent documentation

`AGENTS.md` is an index only. Put durable detail in `docs/agents/{category}/{subtopic}.md`, link it from the category index, and keep the root structure and category jump tables current.

After a meaningful code or workflow change:

1. Read the last documented commit in `AGENTS.md`.
2. Inspect `git diff --name-only {last_commit}..HEAD` and read changed files first.
3. Update only the relevant detail files; do not copy long implementation notes into the root index.
4. Set `Last documented commit` in `AGENTS.md` to the new `git rev-parse HEAD` value.
5. Verify every relative link resolves and every category index links back to the root.

Changes that affect release behavior should update `docs/agents/release/` even if the code change is small. Changes that alter a project rule should update `rules.md` and explain the new expectation in the closest user-facing README section when appropriate.

Back to [maintenance/index.md](index.md)


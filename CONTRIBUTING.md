# Contributing

Direct human-written code contributions are welcome. The project currently
develops software primarily through Codex-driven agentic workflows, but changes
written by people or agents follow the same accountable review,
evidence-gated provenance, licensing, testing, and repository-validation
requirements.

Work from an issue and preserve the dependency boundaries in `AGENTS.md` and
`policy/architecture.json`. New code, shaders, tests, and assets are MIT.
Independent implementation is the default where exact reuse is not proven.
Exact historical Classic material may be inspected or reused only when the
[local provenance record](PROVENANCE.md) and
[canonical grant registry](https://github.com/atrinik/atrinik/blob/main/docs/PROVENANCE.md)
admit every copyrightable portion at an exact source revision. Complete
rename-aware history must prove each portion is original past work solely
authored by its grantor; present-day blame, majority authorship, later edits,
and agent-assisted commits are insufficient. An admitted destination may copy,
adapt, port, translate, and MIT-relicense that material, but must not depend on
the GPL Classic source. The destination grant does not change the Classic
repository's GPL distribution. Record the exact evidence and exclude every
uncovered portion. Visual assets remain separately licensed authored inputs and
require their own exact provenance.

Before opening a pull request, run:

```sh
tools/validate.sh
actionlint .github/workflows/*.yml
git diff --check
```

Public scene, resource, rendering, semantic-output, fixture, and CLI behavior is
versioned API. Pull-request titles and commits use Conventional Commits.

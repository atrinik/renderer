# Contributing

Work from an issue and preserve the dependency boundaries in `AGENTS.md` and
`policy/architecture.json`. New code, shaders, tests, and assets are MIT.
Do not consult or copy classic renderer implementation or visual assets.

Before opening a pull request, run:

```sh
tools/validate.sh
actionlint .github/workflows/*.yml
git diff --check
```

Public scene, resource, rendering, semantic-output, fixture, and CLI behavior is
versioned API. Pull-request titles and commits use Conventional Commits.

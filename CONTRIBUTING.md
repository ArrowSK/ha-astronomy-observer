# Contributing

Contributions are welcome when they improve astronomical correctness, Home Assistant compatibility, resource use, documentation or source resilience.

Before opening a pull request:

1. Explain the observing problem being solved.
2. Keep calculations deterministic and document empirical weighting.
3. Add or update tests for changed scoring behaviour.
4. Cite a primary technical or scientific source when introducing a new physical model.
5. Do not add telemetry, advertising, mandatory accounts, paid API dependencies or remote scoring services.
6. Keep location handling private by default and send no more location precision than a source needs.
7. Keep third-party licensing and attribution explicit.
8. Run `python3 tests/validate_repository.py` and the Rust test suite.

Changes to the score must also update `docs/SCORING.md` so an observer can see what changed and why.

By contributing code, you agree that your contribution can be distributed as part of this project under the project's PolyForm Noncommercial License 1.0.0. Third-party data or code must not be copied into the repository unless its licence is compatible and documented.

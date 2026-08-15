# Contributing

Contributions are welcome when they improve astronomical correctness, platform compatibility, resource use, documentation or source resilience.

Before opening a pull request:

1. Explain the observing problem being solved.
2. Keep calculations deterministic and document empirical weighting.
3. Add or update tests for changed scoring behaviour.
4. Cite a primary technical or scientific source when introducing a new physical model.
5. Do not add telemetry, advertising, mandatory accounts, paid API dependencies or remote scoring services.
6. Keep location handling private by default and send no more location precision than a source needs.
7. Keep third-party licensing and attribution explicit; do not relicense bundled data merely because it ships with the project.
8. Run `python3 tests/validate_repository.py`, `python3 webapp/validate.py`, `python3 android/validate.py` and the Rust test suite when shared code is affected.
9. If shared runtime or interface code changes, also build the standalone Docker image and, where an Android toolchain is available, run `./android/build-apk.sh`.

Changes to the score must also update `docs/SCORING.md` so an observer can see what changed and why.

The Home Assistant, web/Docker and Android editions are adapters around the same observing engine. Do not create a second scoring/target implementation under a platform directory merely to solve a platform integration problem.

## Branch hygiene

Use short-lived topic branches for pull requests and delete them after the pull request is merged. Keep `main` as the long-lived release branch; do not keep merged implementation or one-off maintenance branches around as parallel histories. Before deleting a branch, confirm its pull request is merged and that no unique work still needs to be recovered from it.

By contributing original project code, you agree that your contribution can be distributed as part of Astronomy Observer under the project's PolyForm Noncommercial License 1.0.0. Third-party data/code retains its own licence and must not be copied or bundled unless its terms and required attribution have been checked and documented.

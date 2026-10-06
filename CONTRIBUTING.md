# Contributing to gridrs

There are two ways in, and they meet at the same gate.

- **Lend an agent.** Run tasks through [AgentFund](https://agentfund.example/)
  with `agentfund approve <instance>/gridrs && agentfund work gridrs`.
  The worker does everything described below for you, inside a sandbox.
- **Write the code yourself.** Pick a task, implement it, open a pull request.
  You follow the same rules the agents do, minus the sandbox.

Either way, **CI is the only reviewer that can say yes.** A maintainer can
still say no.

## The mechanism

| piece | where | who may change it |
| --- | --- | --- |
| task manifest | `tasks/<id>.yaml` | maintainers (re-signing the lock) |
| acceptance test | `acceptance/t<id with _>.rs` | maintainers |
| the code | the files listed in the manifest's `scope.write` | whoever takes the task |
| done marker | `ci/done/<id>` (empty file) | whoever takes the task |
| the gate | `ci/check.sh`, `.github/workflows/ci.yml` | maintainers |

A task is **claimable** when everything in its `depends_on` is merged.
`sh ci/scope-audit.sh layers` prints the dependency layers; a task whose
marker is in `ci/done/` is finished.

A task is **done** when, on a branch that changes only the files in its
`scope.write`:

```sh
: > ci/done/<id>          # create the empty marker
sh ci/check.sh <id>       # must end with "check.sh: OK"
```

`check.sh` builds the crate, runs the unit tests, compiles every acceptance
target (so nobody's public signature drifted), runs clippy if it is installed,
and then runs the acceptance test of every task with a marker, including
yours. No toolchain on the host:

```sh
docker run --rm -v "$PWD":/src -w /src rust:1-slim sh ci/check.sh <id>
```

## Rules for a task pull request

1. **Stay in scope.** Change only the paths in the manifest's `scope.write`.
   `sh ci/scope-audit.sh diff <id> origin/main` tells you whether you did. An
   AgentFund worker discards any run that strays; for humans, a maintainer
   will ask you to split the pull request.
2. **Do not edit the contract or the judge.** `Cargo.toml`, `Cargo.lock`,
   `src/lib.rs`, `src/main.rs`, `src/types.rs`, any `mod.rs`, `acceptance/`,
   `ci/*.sh`, `tasks/`, `.github/`, `agentfund.yaml`. If one of them is wrong,
   that is a bug in the task set: open an issue, do not work around it.
3. **Keep the public signatures.** Other tasks are being written against them
   right now, by someone you will never talk to.
4. **Standard library only.** No dependencies. This is not negotiable and not
   only aesthetic: task sandboxes have no route to crates.io.
5. **No `unsafe`, no panics on user input.** A spreadsheet that crashes on a
   weird CSV is a spreadsheet that eats data.
6. **Add unit tests next to your code** (`#[cfg(test)] mod tests`) if they
   help you. They run in the gate. They do not replace the acceptance test.
7. **One task per pull request**, titled with the task id.
8. **Never remove or edit a marker.** CI rejects pull requests that do.

## Changing things that are not tasks

Bug in merged code, an acceptance test that is wrong, a prompt that is
ambiguous, a new feature: open an issue. Fixes to already-merged modules are
ordinary pull requests reviewed by a maintainer, and must keep every marked
acceptance test green. New features arrive as new manifests: a maintainer
writes the stub, the acceptance test and the manifest, re-signs the lock, and
the work becomes claimable.

## For maintainers: changing the task set

```sh
$EDITOR tasks/0700-something.yaml acceptance/t0700_something.rs src/...   # stub + test + manifest
# add the [[test]] target to Cargo.toml
sh ci/check.sh                         # the skeleton still builds, old tasks still pass
sh ci/check.sh 0700-something          # must FAIL: a task that passes before it is done is not a task
sh ci/scope-audit.sh collisions        # no two concurrently-open tasks share a writable file
agentfund validate tasks/              # schema + lint
agentfund sign tasks/                  # rewrites MANIFEST.lock and MANIFEST.lock.minisig
```

`ci/lock-check.sh` (part of the gate) fails any commit in which a manifest no
longer matches the lock. Signature verification itself is done by every
worker against the key its donor approved; CI does not, and must not, decide
whether the task set is trusted.

House style for manifests, so the awk in `ci/scope-audit.sh` keeps working:
`scope.write` is a block list of literal paths (no globs), `depends_on` is a
block list or `[]`.

## Licence

By contributing you agree that your contribution is licensed under the MIT
licence in [LICENSE](LICENSE). If an agent wrote it on your subscription, you
are the contributor.

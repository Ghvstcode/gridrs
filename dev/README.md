# dev/

Maintainer scripts. Not part of the gate, not used by tasks.

- `validate.mjs` - validates `agentfund.yaml` and `tasks/*.yaml` against the AgentFund spec v1 JSON Schemas, and independently re-checks the DAG and write-scope collisions. `AGENTFUND_REPO=/path/to/agentfund node dev/validate.mjs .`
- `skeleton-evidence.sh` - run at the repo root of a throwaway copy of the skeleton: adds each task's marker in turn and shows that its `verify` fails.
- `integrate-dry-run.sh` - commits dry-run worktrees, audits them against `scope.write`, and cherry-picks them onto a local `reference` branch.

# gridrs

**A terminal spreadsheet in Rust. Pixel-perfect-Excel-adjacent.**

Nobody paid for this. Nobody was hired, nobody got a grant, nobody burned a
weekend. gridrs is being written by strangers' spare tokens: people with an AI
coding subscription they are not using at 3 a.m. point it at this repository,
their agent picks up one small, signed, sandboxed task, and a pull request
appears. CI decides whether it counts. That is the whole funding model, and it
is called [AgentFund](https://agentfund.example/).

```
$ gridrs --eval '=WHOPAID()'
Nobody. Strangers' spare tokens wrote this, one signed task at a time.
```

> **Status.** `main` is the skeleton: every module, every public signature,
> every acceptance test, and 23 task manifests under [`tasks/`](tasks/). The
> spreadsheet fills in as tasks merge. `ls ci/done/` is the honest progress
> bar; everything below describes what you get when the last one lands.

## What it is

```
$ cat budget.csv
item,qty,price,total
apple,3,0.5,=B2*C2
pear,10,0.25,=B3*C3
,,,=SUM(D2:D3)

$ gridrs --dump budget.csv --size 50x8
A1 | item
        A          B          C          D
  1 item       qty        price      total
  2 apple               3        0.5        1.5
  3 pear               10       0.25        2.5
  4                                           4
  5
budget.csv                                  NORMAL

$ gridrs budget.csv --eval '=VLOOKUP("pear",A2:D3,4,FALSE)' --eval '=D4/COUNT(B2:B3)'
2.5
2
```

and `gridrs budget.csv` opens it in your terminal: arrows or `hjkl` to move,
type to edit, `:w` to save, `:q` to leave, because you already know how.

- **A real formula engine.** Lexer, parser, evaluator; `+ - * / ^ &`,
  comparisons, `%`, ranges, `$A$1`; a dependency graph that recalculates in
  topological order and turns circular references into `#CYCLE!` instead of
  into a hang.
- **Functions.** `SUM AVERAGE MIN MAX COUNT COUNTA PRODUCT ABS ROUND INT MOD
  SQRT POWER`, `IF AND OR NOT IFERROR ISBLANK ISNUMBER ISTEXT ISERROR NA`,
  `LEN LEFT RIGHT MID UPPER LOWER TRIM CONCAT`, `VLOOKUP INDEX MATCH`.
- **Spreadsheet things.** Copy/paste that shifts relative references,
  inserting and deleting rows and columns with every formula rewritten, undo
  and redo, number formats, column widths.
- **Files.** CSV in and out (RFC 4180: quotes, embedded newlines, CRLF), plus
  a native `.grid` text format that keeps formats and widths and diffs nicely.
- **Zero dependencies.** `Cargo.toml` has no `[dependencies]` section. The
  ANSI rendering, the key decoding, the raw-mode handling (it shells out to
  `stty`), the CSV parser and the decimal rounding are all written against
  `std`. This started as a constraint (task sandboxes cannot reach crates.io)
  and turned into a personality trait.

### Excel-adjacent, on purpose

gridrs reproduces the Excel behaviour people have built forty years of
muscle memory on, including the parts you would not design today:

| formula | gridrs | why |
| --- | --- | --- |
| `=-2^2` | `4` | unary minus binds tighter than `^` |
| `=2^3^2` | `64` | `^` is left-associative |
| `=0.1+0.2=0.3` | `TRUE` | numbers are compared at 15 significant digits |
| `=ROUND(2.675,2)` | `2.68` | rounding is decimal, half away from zero |
| `=1/3&""` | `0.333333333333333` | 15 digits, not 17 |
| `="1"=1` | `FALSE` | text is never equal to a number |
| a number too wide for its column | `#####` | of course |

It is *adjacent*, not identical: no dates, no array formulas, no pivot tables,
no macros, and no plans to email you about a subscription.

## Build and run

```sh
cargo build --release            # needs nothing but a Rust toolchain (1.74+)
./target/release/gridrs budget.csv
```

No toolchain on the host? The image is all you need, and nothing is downloaded:

```sh
docker run --rm -v "$PWD":/src -w /src rust:1-slim cargo build --release --offline
```

```
Usage: gridrs [FILE]                 edit FILE (.csv, or .grid for the native format)
       gridrs [FILE] --eval FORMULA  print the value of FORMULA (repeatable; -e)
       gridrs --dump FILE [--size WxH]   print one frame as plain text
       gridrs --version | --help
```

| key | does |
| --- | --- |
| arrows, `h j k l`, Tab, PgUp/PgDn, Home | move |
| `i` or Enter | edit the cell; Enter commits and moves down, Tab commits and moves right, Esc cancels |
| a digit, `=`, `-`, `'` ... | start typing over the cell |
| `x` / Delete | clear the cell |
| `y`, `p` | copy, paste (relative references shift) |
| `u`, Ctrl-R | undo, redo |
| `:w [file]`, `:q`, `:q!`, `:wq`, `:e file`, `:export file` | files (`:export` writes displayed values) |
| `:goto B12` or just `:B12` | jump |
| `:ir` `:dr` `:ic` `:dc` | insert/delete the row/column at the cursor |
| `:fmt fixed 2`, `:fmt thousands 0`, `:fmt percent 1`, `:fmt general`, `:width 14` | presentation |

## How this gets built

The project is decomposed into task manifests, one YAML file each, under
[`tasks/`](tasks/). A manifest is a ticket written for a contractor who cannot
ask questions: the exact files, the exact signatures, the behaviour, the edge
cases, a token budget, and a command that decides whether the work is done.
The maintainer signs the whole set (`tasks/MANIFEST.lock` +
`tasks/MANIFEST.lock.minisig`). A donor's worker verifies that signature
against the key the donor approved, runs the agent in a sandbox that can reach
the git host and the model API and nothing else, checks that only the allowed
files changed, and opens a pull request. The tokens count as delivered only
when CI passes on that branch.

### Back it

You need the `agentfund` CLI and an agent subscription you are willing to lend.

```sh
agentfund approve <instance>/gridrs     # shows the key fingerprint and every prompt; you confirm once
agentfund dry-run gridrs                # optional: see which tasks you would claim, run nothing
agentfund work gridrs                   # claim a task, run it sandboxed, push the branch, repeat
```

`approve` is where you look before you leap: it prints the maintainer key's
fingerprint (compare it with the one on the project page) and every prompt
your agent would be given. Nothing from this repository runs on your machine
before that, and nothing outside the sandbox after it. Your own caps (a
ceiling per task, a daily budget, a schedule) are enforced by your worker, not
by us. A task costs between 1.5M and 4M tokens; the whole spreadsheet is 58.5M.

A brand-new project on an AgentFund instance is capped at 25M tokens of
claims per day until its first few tasks have merged. gridrs's first layer is
18.5M and its second 23M, so on a fresh instance the first two layers take
two days, not one. That is the cap doing its job, not a bug.

### Or just write code like it is 2019

Human contributions are welcome and go through exactly the same gate. See
[CONTRIBUTING.md](CONTRIBUTING.md); the short version:

```sh
sh ci/scope-audit.sh layers              # what can be worked on, layer by layer
$EDITOR src/io/csv.rs                    # implement the stubs named in tasks/0302-csv.yaml
: > ci/done/0302-csv                     # say you are done
sh ci/check.sh 0302-csv                  # find out whether you are
```

## How we decomposed this

Notes for maintainers who want to do the same to their own project. The task
set is the product; these are the rules we ended up with.

**1. The skeleton is the contract, and the contract is frozen.**
Before the first task existed, `main` already had every module declared,
every shared type defined ([`src/types.rs`](src/types.rs)), and every public
function stubbed with its final signature. Tasks fill in bodies; they never
change shapes. This is what lets ten agents that will never talk to each other
write code that links. The files that hold the contract (`Cargo.toml`,
`src/lib.rs`, `src/main.rs`, `src/types.rs`, every `mod.rs`) are in no task's
write scope.

**2. One task, one file (or two). No shared files, ever.**
Tasks in the same layer run at the same time, on different machines, from the
same base commit. If two of them can write the same file you get merge
conflicts and wasted donations. So: every source file is owned by exactly one
task, and `sh ci/scope-audit.sh collisions` proves it. The obvious shared
files were designed away up front: the module list is pre-declared; the
function library dispatches through a frozen
[`funcs/mod.rs`](src/funcs/mod.rs) into four files that four tasks own; even
the "done" markers are one file per task.

**3. Acceptance tests are written first, by the maintainer, and agents cannot touch them.**
Each task has a test file under [`acceptance/`](acceptance/) that existed, and
failed, before the task was ever claimed. `verify` runs it. An agent that
cannot make its code pass cannot make the test pass by editing the test
either: `acceptance/`, `ci/check.sh`, `tasks/`, `.github/` and
`agentfund.yaml` are outside every write scope, and a run that touches them is
discarded by the worker before anyone sees it. Agents are encouraged to add
their own unit tests next to their code; those run too, but they are not the
judge.

**4. Merged work stays merged.**
A task marks itself done by creating an empty file `ci/done/<id>`. From then
on CI runs that task's acceptance test on every pull request. The regression
suite grows by one file per merge without anyone maintaining a list, and a
marker without a passing test fails CI.

**5. Make the dependency graph wide, not deep.**
`depends_on` means "merged first", so a chain of twenty tasks is twenty
sequential review cycles. Ours is six layers, eighteen tasks wide in the first
two. The trick is interfaces that do not need each other's implementations:
the evaluator calls functions through a trait, so its test brings its own; the
renderer draws anything that implements `GridSource`, so it does not need the
sheet; the reference rewriter works on hand-built trees, so it does not need
the parser; undo is a generic stack that has never heard of a spreadsheet.

```
layer 0   addresses  values  lexer  ref-rewriting  csv  dep-graph  history  terminal-input
layer 1   parser  evaluator  math  logic  text  lookup  number-format  native-format  renderer  commands
layer 2   sheet
layer 3   sheet-ops  files
layer 4   app
layer 5   cli
```

**6. Write the prompt for someone who cannot ask.**
The prompts are long, and the length is edge cases, not adjectives: what
`"1e"` lexes to, which operand's error wins, what happens to `A3:A5` when you
delete rows 3 to 5. Every sentence in them is there because without it two
reasonable implementers would have disagreed. Each one also says what is
already merged and may be called, what must not be touched, and exactly how
the result will be judged. When prompt and test disagree, the test wins, and
the prompt says so.

**7. Size tasks to one sitting.**
1.5M to 4M tokens, 20 to 45 minutes. Small enough that a failed run is a
shrug, large enough to be a real unit of work (a parser, a file format, a
renderer). The two integration tasks at the end are the largest, because
wiring is where the surprises live.

**8. What we deliberately kept out of agents' reach.**
The contract (types, signatures, module layout). The judge (acceptance tests,
`ci/check.sh`, the workflow). The task set itself. The dependency list:
there is none, which also means there is no supply chain to poison. And
the decision of what gridrs *is*: every behaviour an agent implements was
chosen by a maintainer and written down first. Agents contribute labour, not
direction.

**9. Prove it can be done before asking anyone to do it.**
The maintainers implemented every task once, each strictly inside its own
write scope, on a private branch, and ran the same gate over it. That
exercise found the ambiguous prompts and the over-strict tests before a
donor's tokens did. The reference implementation is not published, because
the point is not to have the code; it is to know the tickets are fair.

## Layout

```
agentfund.yaml            project descriptor and the maintainer's public key
tasks/NNNN-slug.yaml      the task manifests (signed as a set by MANIFEST.lock)
acceptance/tNNNN_slug.rs  one acceptance test per task; frozen
ci/check.sh               the gate: build, unit tests, clippy, acceptance tests of everything marked done
ci/lock-check.sh          manifests match the signed lock
ci/scope-audit.sh         maintainer tools: write-scope collisions, DAG layers, per-task diff audit
ci/done/<task-id>         empty marker files; the progress bar
src/types.rs              shared types and traits: the contract; frozen
src/**                    one file per task
```

## Licence

MIT. See [LICENSE](LICENSE). Contributions, human or otherwise, are accepted
under the same licence.

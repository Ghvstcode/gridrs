// Validates a project repo against the AgentFund spec-v1 JSON Schemas (ajv 2020 + yaml),
// plus the cross-file rules from spec/manifest.md, plus an independent write-scope collision check.
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import { join, basename } from "node:path";
const MONO = process.env.AGENTFUND_REPO;
if (!MONO || !process.argv[2]) { console.error("usage: AGENTFUND_REPO=/path/to/agentfund node dev/validate.mjs <repo>   (needs ajv, ajv-formats, yaml in that checkout)"); process.exit(2); }
const require = createRequire(join(MONO, "package.json"));
const Ajv2020 = require("ajv/dist/2020.js").default ?? require("ajv/dist/2020.js");
const addFormats = require("ajv-formats").default ?? require("ajv-formats");
const YAML = require("yaml");
const repo = process.argv[2];
const schema = (n) => JSON.parse(readFileSync(join(MONO, "spec/schemas", n), "utf8"));
const ajv = new Ajv2020({ allErrors: true, strict: true });
addFormats(ajv);
const vManifest = ajv.compile(schema("manifest.schema.json"));
const vProject = ajv.compile(schema("agentfund.schema.json"));
let bad = 0;
const fail = (m) => { bad++; console.log("FAIL " + m); };
// strict YAML: core schema, no anchors/aliases/tags/multi-docs
const load = (file) => {
  const text = readFileSync(file, "utf8");
  const docs = YAML.parseAllDocuments(text, { schema: "core", merge: false, uniqueKeys: true });
  if (docs.length !== 1) fail(`${file}: ${docs.length} YAML documents`);
  const doc = docs[0];
  for (const e of doc.errors) fail(`${file}: ${e.message}`);
  YAML.visit(doc, { Alias() { fail(`${file}: alias`); }, Node(_, n) { if (n.anchor) fail(`${file}: anchor`); if (n.tag && !n.tag.startsWith("tag:yaml.org,2002:")) fail(`${file}: tag ${n.tag}`); } });
  return doc.toJS();
};
const project = load(join(repo, "agentfund.yaml"));
if (!vProject(project)) for (const e of vProject.errors) fail(`agentfund.yaml ${e.instancePath} ${e.message}`);
const files = readdirSync(join(repo, "tasks")).filter((f) => f.endsWith(".yaml")).sort();
const tasks = new Map();
for (const f of files) {
  const m = load(join(repo, "tasks", f));
  if (!vManifest(m)) { for (const e of vManifest.errors) fail(`tasks/${f} ${e.instancePath} ${e.message}`); continue; }
  if (m.id !== basename(f, ".yaml")) fail(`tasks/${f}: id ${m.id} != file stem`);
  tasks.set(m.id, m);
}
let sum = 0;
for (const [id, m] of tasks) {
  sum += m.max_tokens;
  for (const d of m.depends_on ?? []) if (!tasks.has(d)) fail(`${id}: depends_on unknown ${d}`);
  if (m.max_tokens > 4_000_000) fail(`${id}: max_tokens ${m.max_tokens} > 4M`);
  if ((m.timeout_minutes ?? 60) > 45) fail(`${id}: timeout > 45`);
  if (m.verify !== `sh ci/check.sh ${id}`) fail(`${id}: verify is ${m.verify}`);
  if (!m.scope.write.includes(`ci/done/${id}`)) fail(`${id}: marker not writable`);
  for (const g of m.scope.write) {
    if (/[*?{[]/.test(g)) fail(`${id}: glob ${g} (expected literal paths)`);
    if (g === "agentfund.yaml" || g.startsWith("tasks/") || g.startsWith(".github/")) fail(`${id}: never-writable path ${g}`);
    if (!g.startsWith("ci/done/") && !existsSync(join(repo, g))) fail(`${id}: scope.write names missing file ${g}`);
    if (/^(Cargo\.(toml|lock)|src\/(lib|main|types)\.rs|acceptance\/|ci\/[^d])/.test(g) || /\/mod\.rs$/.test(g)) fail(`${id}: frozen file ${g} is writable`);
  }
  for (const f of m.scope.write) if (f.startsWith("src/") && !m.prompt.includes(f)) fail(`${id}: prompt never mentions ${f}`);
  if (!m.prompt.includes(`ci/done/${id}`) || !m.prompt.includes(`sh ci/check.sh ${id}`)) fail(`${id}: prompt lacks marker/verify instructions`);
}
if (project.token_goal !== sum) fail(`token_goal ${project.token_goal} != sum of max_tokens ${sum}`);
// dependency closure + cycle check
const anc = new Map([...tasks.keys()].map((k) => [k, new Set()]));
const visit = (id, stack) => {
  if (stack.includes(id)) { fail(`dependency cycle: ${[...stack, id].join(" -> ")}`); return new Set(); }
  const s = anc.get(id); if (s.done) return s;
  for (const d of tasks.get(id).depends_on ?? []) { s.add(d); for (const x of visit(d, [...stack, id])) s.add(x); }
  s.done = true; return s;
};
for (const id of tasks.keys()) visit(id, []);
// collisions: unordered pairs sharing a writable path
const ids = [...tasks.keys()]; let pairs = 0, shared = 0;
for (let i = 0; i < ids.length; i++) for (let j = i + 1; j < ids.length; j++) {
  pairs++;
  const a = tasks.get(ids[i]), b = tasks.get(ids[j]);
  const common = a.scope.write.filter((p) => b.scope.write.includes(p));
  if (!common.length) continue;
  shared++;
  const ordered = anc.get(ids[i]).has(ids[j]) || anc.get(ids[j]).has(ids[i]);
  if (!ordered) fail(`COLLISION ${ids[i]} / ${ids[j]}: ${common.join(", ")}`);
}
// every src file except the frozen ones is owned by exactly one task
const owned = new Map();
for (const [id, m] of tasks) for (const p of m.scope.write) owned.set(p, [...(owned.get(p) ?? []), id]);
const walk = (d) => readdirSync(join(repo, d), { withFileTypes: true }).flatMap((e) => e.isDirectory() ? walk(join(d, e.name)) : [join(d, e.name)]);
const frozen = new Set(["src/lib.rs", "src/main.rs", "src/types.rs", "src/formula/mod.rs", "src/funcs/mod.rs", "src/io/mod.rs", "src/tui/mod.rs"]);
for (const f of walk("src")) { if (frozen.has(f)) continue; const o = owned.get(f) ?? []; if (o.length !== 1) fail(`${f} owned by ${o.length} tasks: ${o}`); }
// layers
const layer = new Map(); const depth = (id) => layer.get(id) ?? (layer.set(id, Math.max(-1, ...(tasks.get(id).depends_on ?? []).map(depth)) + 1), layer.get(id));
ids.forEach(depth);
const widths = []; for (const [id, l] of layer) (widths[l] ??= []).push(id);
console.log(`${tasks.size} manifests + agentfund.yaml validate against spec v1 schemas: ${bad ? "NO" : "yes"}`);
console.log(`token_goal ${project.token_goal} = sum(max_tokens) ${sum}; max single task ${Math.max(...[...tasks.values()].map((m) => m.max_tokens))}`);
console.log(`collisions: ${pairs} task pairs checked, ${shared} share a path, all of those ordered by the DAG: ${bad ? "NO" : "yes"}`);
widths.forEach((w, i) => console.log(`layer ${i} (width ${w.length}): ${w.join(" ")}`));
process.exit(bad ? 1 : 0);

---
name: use-internode-cli
description: Interface with Internode Organizational Intelligence (OI) via the internode CLI. Use when the user asks to read, browse, search, update, reorganize, clean up, or repair knowledge entities (topics, sub-topics, tasks, decisions, intents, teams, projects, statuses), recover soft-deleted entities, re-align embeddings, or run gated Cypher — or when bootstrapping context for a work session.
---

# Using the Internode CLI

The `internode` CLI is your interface to a user's **Organizational Intelligence (OI)** — a persistent knowledge graph of topics, sub-topics, tasks, decisions, intents, teams, projects, and statuses. Use it as long-term memory: read context before starting work, browse entities, update properties, reorganize and repair the graph, and search semantically.

## Prerequisites

Install the CLI if you haven't already:

```bash
curl -fsSL https://raw.githubusercontent.com/internodelabs/internode-cli/main/install.sh | sh
```

Configure with an API key before first use:

```bash
internode configure <api-key>   # one-time setup; key starts with ink_
internode auth status            # verify the key works
```

Config lives at `~/.config/internode/config.toml`.

## Output Format

**Every command** prints a single JSON line to stdout.

Success:
```json
{"ok":true,"data":{...}}
```

Error (exit code > 0):
```json
{"ok":false,"error":{"code":"AUTH_ERROR","message":"..."}}
```

Error codes: `BAD_INPUT` (exit 1), `AUTH_ERROR` (exit 2), `SERVER_ERROR` (exit 3), `NETWORK_ERROR` (exit 4).

Human-readable messages go to stderr. **Always parse stdout JSON, ignore stderr.**

## Permissions Model

The CLI is **read-heavy with structural-cleanup and repair writes**:

- **Read all**: topics, sub-topics, tasks, decisions, intents, teams, projects, statuses.
- **Diagnose** structural noise (uncapped edge counts) for decisions, topics, sub-topics, intents, plus forked version chains.
- **Update scalar fields** on tasks, topics, decisions, intents (and revise sub-topic versions append-only).
- **Reorganize edges**: move sub-topics between topics; link/unlink decision edges (sub-topic, task, intent); re-parent tasks (`HAS_SUBTASK`); normalize contradictory decision rel-types.
- **Merge** duplicate roots (topics, decisions, intents, tasks) and **split** an over-merged root (topics, decisions, intents) back apart.
- **Soft-archive** topics, sub-topics, decisions, intents, tasks (sets `deleted=true`; never hard-delete).
- **Recover** soft-deleted entities (`entity list-deleted` → `entity restore`).
- **Repair** forked version chains (`repair version-chains`).
- **Re-align embeddings** (`embeddings status` / `embeddings sync`) — the "commit my changes" step for the knowledge graph.
- **Create** net-new entities with an optional historical date: topics, decisions, intents, tasks (`<entity> create … --data-date`), plus projects/teams/statuses (`… --created-date`).
- **Fix a single version in place**: `<decisions|intents|tasks> version set-date <vid> --data-date …` re-dates one version; `… version delete <vid>` soft-deletes one bad version. Both re-linearize the chain by date.
- **Fix a root's creation date**: `<projects|teams|statuses> set-created-date <id> --created-date …`.
- **Gated Cypher**: `cypher run` executes a user-reviewed `.cypher` file behind a per-owner passphrase the agent does not know.

> **Hard delete is never available.** Every "archive" / "merge source" sets `deleted=true` so history stays traversable, and archives are reversible via `entity restore`.

## Recommended Workflow

1. **Bootstrap context** at the start of a session:
   ```bash
   internode context --max-tokens 4000
   ```
   Returns a pre-formatted OI summary optimized for LLM consumption.

2. **Search** when you need specific knowledge:
   ```bash
   internode search "deployment pipeline"
   ```

3. **Browse** entity lists to find what you need:
   ```bash
   internode topics list --category 3
   internode tasks list --team <id> --status "In Progress"
   internode subtopics list --type Idea
   ```

4. **Get details** on one or more entities by ID (knowledge molecule for tasks/decisions/sub-topics, full properties for everything else; max 20 IDs per call):
   ```bash
   internode entity get <id1> [<id2> ...]
   ```

5. **Update / reorganize** as you work — change task properties, re-parent, move sub-topics, fix edges.

6. **Clean up & repair** — see "Cleaning up & repairing the graph" for the diagnose → inspect → mutate → diagnose loop.

7. **Re-align embeddings** after content-affecting changes so semantic search stays accurate:
   ```bash
   internode embeddings sync
   ```

## Entity Types

| Entity | What it represents | Key property |
|---|---|---|
| **Topic** | A knowledge area, discussion, or theme | `topic_title` |
| **Sub-topic** | A typed conclusion under a topic (Idea, Problem, Solution, etc.) | `topic_conclusion` |
| **Task** | An actionable work item | `task_title` |
| **Decision** | A resolved choice with rationale | `decision_title` |
| **Intent** | A strategic intent or goal | `intent_title` |
| **Team** | An organizational group | `name` |
| **Project** | A body of work under a team | `name` |
| **Status** | A workflow state for tasks | `name` |

## Command Reference

### Context & Discovery

```bash
internode context [--max-tokens N]
# Full OI context dump for LLM consumption. Use --max-tokens to budget.

internode search "<query>"
# Semantic search across all entity types.

internode entity get <id1> [<id2> ... <idN>] [--include-deleted]
# Get full details for up to 20 entities by ID. Knowledge molecule for tasks,
# decisions, sub-topics; full properties for topics, intents, teams, projects,
# statuses. Response keyed by entity ID. --include-deleted returns a minimal
# payload for soft-deleted entities (use with the recovery workflow).
```

### List Endpoints

All list commands return lightweight results: `{ items: [{ id, label }], total, limit, offset }` where `label` is the key property capped to 200 characters.

```bash
internode topics list [--category N] [--search "text"] [--limit N] [--offset N]

internode subtopics list [--type "Idea|Problem|Solution|..."] [--topic ID] [--limit N] [--offset N]
# Types: Outcome, Problem, Constraint, Solution, Opportunity, Idea, Information.

internode tasks list [--team ID] [--project ID] [--status "name"] [--priority "..."] [--assignee "email"] [--search "text"] [--topic ID] [--intent ID] [--topic-category "index"] [--limit N] [--offset N]
# topic, intent, and topic-category filter tasks through the decision graph.

internode decisions list [--search "text"] [--limit N] [--offset N]
internode intents list [--limit N] [--offset N]
internode teams list
internode projects list [--team ID]
internode statuses list [--team ID]
```

### Inspecting a single entity (uncapped relationship dump)

`<entity> inspect <id>` returns the **full**, uncapped neighborhood for one root. Use this to plan a move/merge/split/unlink.

```bash
internode topics inspect    <topic_id>          # parent, sub-topic versions, decision links per sub-topic
internode subtopics inspect <sub_topic_id>      # parent topic + every incoming decision edge
internode decisions inspect <decision_id>       # every sub-topic, intent, and task edge
internode intents inspect   <intent_id>         # every supporting decision
```

### Diagnostics (uncapped — find the noise)

`internode diagnose` returns the **real** edge counts so you can see structural noise. Unlike `entity get` (which caps related-entity lists at 4), the diagnostic endpoints are uncapped.

```bash
internode diagnose decisions [--by sub_topics|tasks|intents] [--top N] [--min-edges N] [--offset N]
# OIDecisions ranked by outgoing sub-topic / task / intent edges. Default --by sub_topics.

internode diagnose topics    [--by sub_topics|decisions]      [--top N] [--min-edges N] [--offset N]
# OITopic roots ranked by sub-topic count and distinct decisions touching any sub-topic.

internode diagnose subtopics                                  [--top N] [--min-edges N] [--offset N]
# OITopicVersion sub-topics ranked by incoming decision-edge count.

internode diagnose intents                                    [--top N] [--min-edges N] [--offset N]
# OIIntents ranked by incoming SUPPORTS edge count from decisions.

internode diagnose version-chains [--labels OIDecision,OIIntent,OITask] [--limit N]
# Single-lineage roots whose version history has FORKED into multiple heads
# (breaks head resolution downstream). Repair with `repair version-chains`.
```

### Task mutations

```bash
internode tasks create --title "..." [--description "..."] [--priority "..."] [--assignee "email"] [--due-date "YYYY-MM-DD"] [--status ID] [--type "..."] [--team ID] [--project ID] [--parent <task_id>] [--data-date "YYYY-MM-DD"]
internode tasks update <id> [--title "..."] [--description "..."] [--priority "..."] [--assignee "email"] [--due-date "YYYY-MM-DD"] [--status ID] [--type "..."] [--team ID] [--project ID] [--user-notes "..."] [--blocked-by-reason "..."] [--parent <task_id>] [--clear-parent] [--data-date "YYYY-MM-DD"]
internode tasks archive <id>                                   # soft-delete root + stamp a deleted version
internode tasks merge   <source_id> --into <target_id>         # re-point decision edges, team/project, subtasks; then archive source
```

- **Team/project changes:** When changing a task's team, incompatible project, status, and assignee are auto-cleared. The response includes a `cleared_fields` list when this happens. Projects depend on teams — the target project must belong to the target team.
- **Re-parenting (`HAS_SUBTASK`):** `--parent <task_id>` makes this task a subtask of another; `--clear-parent` detaches it (becomes a root task). A task has at most one parent — re-parenting auto-detaches the old parent. Cycles and self-parenting are rejected (422).
- **`tasks merge`:** incoming decision edges (SPAWNS/BLOCKS/CANCELS/MODIFIES) re-point onto the target's live head version; `HAS_TASK` ownership and `HAS_SUBTASK` parent/child edges re-parent onto the target; then the source is archived.

### Project create

```bash
internode projects create --name "..." --team <team-id> [--key "..."] [--description "..."] [--created-date "YYYY-MM-DD"]
internode projects set-created-date <project_id> --created-date "YYYY-MM-DD"
internode teams create --name "..." [--key "..."] [--description "..."] [--created-date "YYYY-MM-DD"]
internode teams set-created-date <team_id> --created-date "YYYY-MM-DD"
internode statuses create --team <team-id> --name "..." [--description "..."] [--category "..."] [--created-date "YYYY-MM-DD"]
internode statuses set-created-date <status_id> --created-date "YYYY-MM-DD"
# A project always belongs to a team (--team is required).
```

### Topic mutations

```bash
internode topics create  --title "..." [--description "..."] [--category 1..11] [--conclusion "..."] [--conclusion-type "..."] [--primary-contributor "email"] [--data-date "YYYY-MM-DD"]
internode topics update  <id> [--title "..."] [--description "..."] [--category 1..11] [--primary-contributor "email"] [--data-date "YYYY-MM-DD"]
internode topics archive <id>                                  # soft-delete root + all versions
internode topics merge   <source_id> --into <target_id>        # re-parent every sub-topic version, then archive source
internode topics split   <source_id> --file <plan.json> [--keep-source] [--dry-run]
```

`topics split` re-parents groups of sub-topic versions onto existing and/or freshly-created topics. The plan file is a JSON array of entries, each with either `target_topic_id` **or** `new_topic` (`{topic_title, topic_description, category_index, data_date}`) plus `sub_topic_version_ids`. By default the source is archived after splitting; pass `--keep-source` to keep it. `--dry-run` prints the plan without writing.

### Sub-topic mutations

```bash
internode subtopics move    <sub_topic_id> --to-topic <target_topic_id>
internode subtopics archive <sub_topic_id>
internode subtopics update  <sub_topic_id> [--conclusion "..."] [--type Idea|Problem|Solution|Information|Outcome|Opportunity|Constraint] [--primary-contributor "email"] [--data-date "YYYY-MM-DD"]
```

- `move` atomically swaps `HAS_VERSION` so the version is owned by exactly one topic. The conclusion text (and embedding) is untouched.
- `update` **revises** a sub-topic: it appends a NEW version chained from the prior tail (versions are append-only — never overwritten in place).

### Decision mutations

```bash
internode decisions create  --title "..." [--description ...] [--rationale ...] [--status ...] [--decision-maker email] [--type explicit|implicit] [--priority ...] [--data-date "YYYY-MM-DD"]
internode decisions update  <id> [--title ...] [--description ...] [--rationale ...] [--status ...] [--decision-maker email] [--type explicit|implicit] [--priority ...] [--data-date "YYYY-MM-DD"]
internode decisions archive <id>
internode decisions merge   <source_id> --into <target_id>     # re-parent every sub-topic / task / intent edge, then archive source
internode decisions split   <source_id> --file <plan.json> [--keep-source] [--dry-run]

# Edge link/unlink — pass exactly one of --sub-topic, --task, --intent
internode decisions link   <id> --sub-topic <stid> [--type RATIFIES|REJECTS|DEFERS]
internode decisions link   <id> --task <task_or_version_id> [--type SPAWNS|BLOCKS|CANCELS|MODIFIES]
internode decisions link   <id> --intent <intent_id>

internode decisions unlink <id> --sub-topic <stid> [--type RATIFIES|REJECTS|DEFERS]
internode decisions unlink <id> --task <task_or_version_id> [--type SPAWNS|BLOCKS|CANCELS|MODIFIES]
internode decisions unlink <id> --intent <intent_id>

# Collapse contradictory rel-types on the same (decision, target) pair
internode decisions normalize-edges [--decision <id>] [--sub-topic-prefer RATIFIES,REJECTS,DEFERS] [--task-prefer SPAWNS,MODIFIES,BLOCKS,CANCELS] [--dry-run]
```

`decisions split` re-edges groups of sub-topic/task/intent edges onto existing and/or freshly-created decisions. The plan file is a JSON array of entries with either `target_decision_id` **or** `new_decision` (`{decision_title, description, rationale, decision_status, decision_maker_email, decision_type, priority, data_date}`) plus `edges` (`[{kind: "sub_topic"|"task"|"intent", target_id, rel_type}]`). `--keep-source` keeps the source; `--dry-run` previews.

`decisions normalize-edges` keeps the single most-preferred rel-type when a decision points at the same target with multiple conflicting types. Omit `--decision` to scan every live decision. Types not listed in a `*-prefer` order are never kept.

> **Decision invariant (HTTP 422 on violation):** Every live OIDecision MUST keep ≥1 sub-topic edge AND ≥1 intent edge. If `unlink` would drop either to zero, it's blocked with a structured 422. Add a replacement edge first, or call `merge` / `archive` to retire the decision explicitly.

> **`link --task` SPAWNS guard:** The backend reuses the V3 `_safe_link_decision_to_task_version` helper, so SPAWNS is auto-downgraded to MODIFIES when the target isn't the first task version, and BLOCKS/CANCELS/MODIFIES is auto-upgraded to SPAWNS when the target is a first version with no SPAWNS yet. The response carries a `note` field when this happens.

### Intent mutations

```bash
internode intents create  --title "..." [--statement ...] [--scope ...] [--signal "phrase" --signal ...] [--data-date "YYYY-MM-DD"]
internode intents update  <id> [--title ...] [--statement ...] [--scope ...] [--signals "a,b,c"] [--data-date "YYYY-MM-DD"]
internode intents archive <id>
internode intents merge   <source_id> --into <target_id>       # re-parent every incoming SUPPORTS edge, then archive source
internode intents split   <source_id> --file <plan.json> [--keep-source] [--dry-run]

internode intents add-signal    <id> --signal "phrase" [--signal "phrase" ...] [--data-date "YYYY-MM-DD"]   # deduped, case-insensitive
internode intents remove-signal <id> --signal "phrase" [--signal "phrase" ...] [--data-date "YYYY-MM-DD"]   # matched case-insensitively
internode intents set-scope     <id> "<scope text>" [--data-date "YYYY-MM-DD"]    # pass "" to clear

# Consolidate several source intents into one target
internode intents consolidate --into <target_id> --source <id> [--source <id> ...] [--statement-strategy keep_target|first_non_empty] [--scope-strategy keep_target|first_non_empty] [--signals-strategy union|keep_target] [--dry-run] [--data-date "YYYY-MM-DD"]
```

`intents split` undoes a false consolidation/merge: it re-points groups of supporting decisions (incoming `SUPPORTS` edges) onto existing and/or freshly-created intents. The plan file is a JSON array of entries with either `target_intent_id` **or** `new_intent` (`{intent_title, statement, scope, signals, data_date}`) plus `supporting_decision_ids`. `--keep-source` keeps the source; `--dry-run` previews.

`intents consolidate` is the inverse of split — it folds multiple source intents into one target (re-points their SUPPORTS edges, merges statement/scope/signals per strategy), then archives the sources.

> **Preserve history with `--data-date` / `--created-date`:** every command that writes a new version accepts an optional `--data-date <ISO-8601>` (`2025-03-14` or `2025-03-14T10:00:00Z`). This covers `create` and `update` (topics, sub-topics, tasks, decisions, intents), the intent version-writing ops `set-scope`, `add-signal`, `remove-signal`, `consolidate`, and the per-version `version set-date` fix. It stamps the version with that historical date instead of "now", so the timeline reflects when the knowledge actually happened — critical when correcting backfilled or split data. The version is inserted into the chain at the correct point by date. You may pass `--data-date` alone (no content change) to append a date-corrected version. An unparseable value returns 422. In `split` plans, add a `"data_date"` key inside any `new_topic` / `new_decision` / `new_intent` object to backdate the entity it creates (otherwise it defaults to today, which distorts history). Non-versioned roots (projects, teams, statuses) use `--created-date` at create time and `set-created-date` to fix an existing root.

### Per-version history fixes (decisions / intents / tasks)

Versions are append-only, so a date-only `update` adds a *new* correctly-dated version but leaves the wrongly-dated one behind. To correct an individual version in place, use the `version` sub-commands; the live chain is automatically re-linearized by date afterward.

```bash
internode decisions version set-date <version_id> --data-date "YYYY-MM-DD"   # re-date one OIDecisionVersion in place
internode decisions version delete   <version_id>                            # soft-delete one bad version
internode intents   version set-date <version_id> --data-date "YYYY-MM-DD"
internode intents   version delete   <version_id>
internode tasks     version set-date <version_id> --data-date "YYYY-MM-DD"
internode tasks     version delete   <version_id>
```

`version delete` is refused (422) when it would remove the only live version of an entity — archive the root instead. Find version ids with `entity get <root_id>` or `<entity> inspect <id>`.

### Recovery — restore soft-deleted entities

Every archive/merge is reversible. Find and restore soft-deleted roots:

```bash
internode entity list-deleted [--labels OITopic,OIDecision,OIIntent,OITask] [--search "text"] [--limit N] [--offset N]
internode entity get <id> --include-deleted          # verify it's the right entity before restoring
internode entity restore <id> --label OITopic|OIDecision|OIIntent|OITask
# restore un-deletes the root and its versions and re-enqueues pgvector embeddings.
```

### Repair — forked version chains

Single-lineage entities (OIDecision/OIIntent/OITask) keep one linear version history per root. Pipeline bugs can fork a chain into multiple heads, which breaks head resolution (snapshots, search, embeddings). Diagnose with `diagnose version-chains`, then repair:

```bash
internode repair version-chains --dry-run                        # preview every forked root that would be repaired
internode repair version-chains                                  # re-linearize all forked roots by data_date
internode repair version-chains --labels OIDecision,OIIntent     # restrict to specific types
internode repair version-chains --ids oidecision_abc,oiintent_xyz # repair only these roots
```

Repair drops the tangled `UPDATED_TO` edges among a root's live versions and rebuilds a single date-ordered chain with exactly one head, then re-enqueues embeddings (the head may have changed). OITopic is intentionally **not** repairable this way — a topic fans out into many independent sub-topic lineages, so "multiple heads" is normal there.

### Embeddings — re-align pgvector with Neo4j ("commit")

```bash
internode embeddings status
# Read-only drift report: Neo4j vs pgvector per entity type. Safe any time.

internode embeddings sync [--scope all|OITopic|OIIntent|OITask|OIDecision|OIProject|ExternalSyncJob|OITopicVersion] [--ids id1,id2] [--since 2024-09-01T00:00:00Z] [--force] [--dry-run] [--no-wait] [--timeout N]
# Realign embeddings after content-affecting changes. Default is synchronous
# (bounded by --timeout, default 120s, max 900). --no-wait backgrounds the work.
# --dry-run reports the plan without writing. --force re-embeds even when the
# v3 hash is unchanged. --ids and --since are mutually exclusive.
```

Run `embeddings sync` after edits that change searchable content (titles, descriptions, conclusions, merges, splits, repairs). Most mutation endpoints already enqueue embeddings automatically — `sync` is the explicit catch-up / drift-fixer.

### Gated Cypher runner

A user-only escape hatch for graph surgery the structured commands can't express. The agent drafts a `.cypher` file; the **user** reviews it and runs it, typing a per-owner passphrase the agent does not know — so an agent cannot execute the file it wrote.

```bash
internode cypher set-passphrase           # prompts twice; min length 12 chars (interactive TTY only)
internode cypher run <file.cypher>        # prompts for the passphrase, then executes blocks (separated by lines containing only ';')
internode cypher run <file.cypher> --dry-run   # validate guardrails (EXPLAIN) without executing any block
```

After a real run that mutates content, the API hint suggests `internode embeddings sync` to re-align pgvector. Queries are owner-scoped and guardrailed (denylist + owner-id binding).

## Cleaning up & repairing the graph

Recurring defects you may need to fix:

1. **Sub-topic noise under topics** — `OITopicVersion` nodes attached to the wrong `OITopic` root, plus duplicate roots about the same subject.
2. **Over-linked / contradictory decisions** — a single `OIDecision` carrying a huge fan-out, or pointing at the same target with conflicting rel-types.
3. **Duplicate roots** — two topics / decisions / intents / tasks about the same thing (false merges create one; false splits create two).
4. **Forked version chains** — a decision/intent/task root with multiple version heads.

### Workflow loop

```text
diagnose  →  inspect  →  mutate  →  diagnose  →  embeddings sync
```

1. **`diagnose`** to find outliers / forks (uncapped counts).
2. **`inspect`** the worst offenders to see every edge.
3. **`mutate`** with the right primitive (`move` / `merge` / `split` / `link` / `unlink` / `normalize-edges` / `archive` / `repair`).
4. **`diagnose`** again to confirm.
5. **`embeddings sync`** to re-align semantic search.

### Decision tree: which primitive?

| Symptom | Right primitive |
|---|---|
| Sub-topic attached to the wrong topic root | `subtopics move <sub_id> --to-topic <correct_topic_id>` |
| Sub-topic conclusion text needs revising | `subtopics update <sub_id> --conclusion "..." --type ...` (appends a new version) |
| Two `OITopic` roots about the same subject | `topics merge <duplicate_id> --into <canonical_id>` |
| One topic actually covers several distinct subjects | `topics split <id> --file plan.json` |
| Two `OIIntent` roots about the same goal | `intents merge <duplicate_id> --into <canonical_id>` |
| Several intents that should be one | `intents consolidate --into <target> --source <id> --source <id>` |
| One intent was falsely merged from several | `intents split <id> --file plan.json` |
| Two `OIDecision` roots about the same choice | `decisions merge <duplicate_id> --into <canonical_id>` |
| One decision conflates several distinct choices | `decisions split <id> --file plan.json` |
| Two `OITask` roots that are the same task | `tasks merge <duplicate_id> --into <canonical_id>` |
| A task belongs under a different parent task | `tasks update <id> --parent <parent_id>` (or `--clear-parent`) |
| Decision linked to a sub-topic/task with the wrong rel-type | `decisions unlink ... --type WRONG` then `decisions link ... --type RIGHT`, or `decisions normalize-edges` for bulk conflicts |
| Decision linked to something unrelated | `decisions unlink <did> --sub-topic <stid>` (422 if it's the last sub-topic) |
| Decision/topic/intent/task is genuinely wrong | `<entity> archive <id>` (reversible via `entity restore`) |
| Accidentally archived something | `entity list-deleted` → `entity restore <id> --label <RootLabel>` |
| A decision/intent/task has multiple version heads | `repair version-chains` (preview with `--dry-run` or `diagnose version-chains`) |

### Worked examples

**Find and inspect the worst over-linked decision**

```bash
internode diagnose decisions --by sub_topics --top 5
# → pick the worst id, e.g. oidecision_abc

internode decisions inspect oidecision_abc
# → shows every sub-topic, intent, task edge with parent topics
```

**Merge two duplicate topics, then re-align search**

```bash
internode diagnose topics --by sub_topics --top 50      # spot duplicates by title
internode topics merge oitopic_dup --into oitopic_canonical
internode embeddings sync --scope OITopic
```

**Split a falsely-merged intent (preview first)**

```bash
cat > /tmp/intent_split.json <<'JSON'
[
  { "new_intent": {"intent_title": "Reduce churn", "statement": "Lower monthly logo churn", "scope": "retention"},
    "supporting_decision_ids": ["oidecision_a", "oidecision_b"] }
]
JSON
internode intents split oiintent_mixed --file /tmp/intent_split.json --dry-run   # inspect the plan
internode intents split oiintent_mixed --file /tmp/intent_split.json             # apply (archives source by default; --keep-source to keep)
```

**Repair forked version chains**

```bash
internode diagnose version-chains                 # any forked roots?
internode repair version-chains --dry-run         # what would change
internode repair version-chains                   # re-linearize, re-embed
```

**Recover an accidentally archived decision**

```bash
internode entity list-deleted --labels OIDecision --search "pricing"
internode entity get oidecision_abc --include-deleted    # confirm
internode entity restore oidecision_abc --label OIDecision
```

## Key Patterns

### Parse output reliably

```bash
result=$(internode topics list 2>/dev/null)
if echo "$result" | jq -e '.ok' > /dev/null 2>&1; then
  echo "$result" | jq '.data'
fi
```

### Preview destructive structural changes

`split`, `normalize-edges`, `consolidate`, `repair version-chains`, and `embeddings sync` all support `--dry-run`. Use it to inspect the plan before committing — especially for split/merge/repair, which move many edges at once.

### Entity detail returns knowledge molecules

For tasks, decisions, and sub-topics, `entity get` returns a **knowledge molecule** — the entity plus its decision-centric neighborhood. For other types, the full property set. Up to 20 IDs per call; response keyed by entity ID. IDs that fail to resolve return an `error` field.

### Mutations are validated server-side

The API enforces allowed fields, entity types, and invariants (e.g. the decision invariant, single-parent tasks, split-target shape). Invalid input returns a `422` with a descriptive message — read it; it tells you exactly what to fix.

### IDs are UUIDs

All entity IDs are UUID-style strings returned in `data` from list/get commands. Store and reuse them.

### Sub-topic types

Valid types: `Outcome`, `Problem`, `Constraint`, `Solution`, `Opportunity`, `Idea`, `Information`. Filter with `--type` on `subtopics list`.

### Topic categories

Topics are grouped into business categories (index 1-11): Strategy & Leadership, Product & Innovation, Technology & Engineering, People & Talent, Finance & Business Operations, Marketing & Brand, Sales & Revenue, Customer Success & Support, Legal & Regulatory, Data & Analytics, Other. Filter with `--category` on `topics list`.

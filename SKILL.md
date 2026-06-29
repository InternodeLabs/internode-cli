---

## name: use-internode-cli
description: Interface with Internode Organizational Intelligence (OI) via the internode CLI. Use when the user asks to read, browse, search, update, reorganize, clean up, or repair knowledge entities (topics, sub-topics, tasks, decisions, intents, teams, projects, statuses), recover soft-deleted entities, re-align embeddings, or run gated Cypher — or when bootstrapping context for a work session.

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
- **Reorganize edges**: move sub-topics between topics; link/unlink decision edges (sub-topic, task, intent); re-parent tasks; normalize contradictory decision rel-types.
- **Merge** duplicate roots (topics, decisions, intents, tasks) and **split** an over-merged root (topics, decisions, intents) back apart.
- **Soft-archive** topics, sub-topics, decisions, intents, tasks (sets `deleted=true`; never hard-delete).
- **Recover** soft-deleted entities (`entity list-deleted` → `entity restore`).
- **Repair** forked version chains (`repair version-chains`).
- **Re-align embeddings** (`embeddings status` / `embeddings sync`) — the "commit my changes" step for the knowledge graph.
- **Create** net-new entities with an optional historical date: topics, decisions, intents, tasks (`<entity> create … --data-date`), plus projects/teams/statuses (`… --created-date`).
- **Review full version history**: `<decisions|intents|tasks> history <root_id>` returns the entire version timeline (content + dates + head/deleted flags), not just the head.
- **Fix a single version in place**: `<decisions|intents|tasks> version set-date <vid> --data-date …` re-dates one version; `… version delete <vid>` soft-deletes one bad version (both re-linearize the chain by date); `… version set-content <vid> --…` overwrites the content of one historical version in place (audit-only unless it's the head; rewrites otherwise-append-only history, so use sparingly).
- **Fix a root's creation date**: `<projects|teams|statuses> set-created-date <id> --created-date …`.
- **Gated Cypher**: `cypher run` executes a user-reviewed `.cypher` file behind a per-owner passphrase the agent does not know.
- **Bulk text correction**: `replace-text "<search>" "<replacement>"` corrects a misspelling / bad transcription across **every text property of every node you own**, in place. Dry-run by default (`--apply` writes); search must be ≥4 characters; identity/structural keys are protected.

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



## Data Model — how the knowledge connects

OI is a **versioned, decision-centric knowledge graph**. Read this section before
browsing: it explains what the entities are, how they link, what versioning means
for reads, and what a "knowledge molecule" actually is. The CLI commands below all
operate on this model.

### Entity types and their fields

**Knowledge entities** — extracted from meetings, notes, messages, and documents:


| Entity                           | Represents                                                                                                                                    | Head-version fields                                                                                                                                                                                |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **OITopic**                      | A knowledge area / theme. Grouped into one of 11 business categories.                                                                         | `topic_title`, `topic_description`                                                                                                                                                                 |
| **Sub-topic** (`OITopicVersion`) | A single **typed conclusion** under a topic. Each sub-topic is one version in the topic's history.                                            | `topic_conclusion`, `topic_conclusion_type` (Outcome, Problem, Constraint, Solution, Opportunity, Idea, Information)                                                                               |
| **OITask**                       | An actionable work item. Users also call these *tickets*, *issues*, *action items*, or *to-dos* — all the same entity. Can nest via subtasks. | `task_title`, `description`, `priority`, `assignee_email`, `due_date`, `user_notes`, `blocked_by_reason`, `task_type` (`action_item` | `deal_opportunity`); status is held on an edge, not a field |
| **OIDecision**                   | A resolved choice with rationale. **The hub that connects everything else.**                                                                  | `decision_title`, `description`, `rationale`, `decision_status` (typically proposed, deferred, accepted, rejected, superseded), `decision_type`, `priority`, `decision_maker_email`                |
| **OIIntent**                     | A strategic intent or goal.                                                                                                                   | `intent_title`, `statement`, `scope`, `signals` (list of phrases)                                                                                                                                  |


**Structural entities** — the org scaffold that tasks hang off:


| Entity              | Represents                                                                   | Fields                                                                      |
| ------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
| **OITeam**          | An organizational group. Owns projects, statuses, and tasks.                 | `name`, `key`, `description`, `member_emails`                               |
| **OIProject**       | A body of work under **exactly one** team.                                   | `name`, `key`, `description`                                                |
| **OIStatus**        | A per-team workflow state for tasks.                                         | `name`, `description`, `category` (not_started, in_progress, done, blocked) |
| **OITopicCategory** | Static grouping of topics (the 11 categories listed at the end of this doc). | `name`                                                                      |




### The relationship graph is decision-centric

The `OIDecision` **is the hub**. Topics, tasks, and intents almost never link to
each other directly — they connect *through* decisions. To understand why a task
exists, or what a topic led to, you walk through the decisions that touch it.

```
                      ┌──────────────────────────┐
    sub-topic  ◄──────┤         OIDecision        ├──────►  OITaskVersion
 (OITopicVersion)     │   (the connecting hub)    │
   RATIFIES /         └─────────────┬─────────────┘   SPAWNS / BLOCKS /
   REJECTS / DEFERS                 │ SUPPORTS          CANCELS / MODIFIES
                                    ▼
                                 OIIntent
```

Hub edges:

- `OIDecision -[RATIFIES | REJECTS | DEFERS]-> sub-topic` — the decision acts on a conclusion (an `OITopic` version).
- `OIDecision -[SPAWNS | BLOCKS | CANCELS | MODIFIES]-> OITaskVersion` — decisions create and change tasks. The edge targets a specific task **version**: `SPAWNS` the first version; the others later versions (this is why `link --task` auto-adjusts the rel-type — see Decision mutations).
- `OIDecision -[SUPPORTS]-> OIIntent` — the choice advances a goal.
- `OITask -[HAS_SUBTASK]-> OITask` — task hierarchy (one parent max; sibling subtasks keep an explicit order).

Structural edges:

- `OITopicCategory -[CONTAINS]-> OITopic`
- `OITopic -[HAS_VERSION]-> sub-topic` — a topic's sub-topics *are* its typed conclusions.
- `OITeam -[HAS_PROJECT]-> OIProject`, `OITeam -[HAS_TASK]-> OITask`, `OIProject -[HAS_TASK]-> OITask`
- `OITeam | OIProject -[HAS_STATUS]-> OIStatus`, and `task version -[HAS_STATUS]-> OIStatus`

> **Decision invariant:** every live `OIDecision` keeps **≥1 sub-topic edge AND ≥1 intent edge** (enforced on `unlink`, see Decision mutations). That is what guarantees the hub stays anchored to both a conclusion and a goal — and why the graph stays navigable.



### Versioning model — roots vs. versions

Topics, sub-topics, tasks, decisions, and intents are **versioned**:

- A **root** node (e.g. `OITask`) is the stable identity and the ID you reference everywhere.
- Content lives on **append-only version** nodes chained in date order. Editing never overwrites in place — `update` appends a *new* version, so the chain is a full audit history.
- Every normal read resolves the single **head** (latest) version. Use `<entity> history <root_id>` to see the whole chain, and the `version` sub-commands to correct one historical version.
- `OITask` / `OIDecision` / `OIIntent` each keep **one linear chain** (head = newest). A chain can occasionally fork into multiple heads — find and fix these with `diagnose version-chains` → `repair version-chains`.
- `OITopic` **is the exception:** its "versions" are the **sub-topics**, which fan out into many independent conclusion lineages. A topic legitimately has many heads (one per sub-topic), so it is **not** repairable by `repair version-chains`.
- **Structural roots are not versioned** — `OITeam` / `OIProject` / `OIStatus` / `OITopicCategory` carry their fields directly and have a single `created_at` (fixable via `set-created-date`).



### Knowledge molecules

A **knowledge molecule** is *one entity plus its decision-centric neighborhood* —
the surrounding entities you reach by walking through the hub. This is the unit
`entity get` returns for **tasks, decisions, and sub-topics** (full properties for
everything else). It bundles the connected entities so you can understand an item
in one call instead of traversing edges by hand.


| `entity get <id>` on…                | Returns                                                                                                                                                                     |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Task**                             | the task (current state) + the decisions acting on it (with their rel-type, each carrying its own sub-topics and intents) + parent and subtask references |
| **Decision**                         | the decision + the sub-topics, intents, and tasks it connects                                                                                                        |
| **Sub-topic**                        | its **parent topic's molecule**: the topic + category + the topic's sub-topics + the connected decisions and tasks                             |
| **Topic** (root id)                  | the topic's own properties + category (to read its conclusions, pass a sub-topic id, or use `topics inspect` / `subtopics list --topic`)             |
| **Intent / Team / Project / Status** | the full property set                                                                                                                                                       |


The molecule is what makes the graph *navigable in one call*: from any task you
immediately see the decisions that drove it and, through them, the topics and goals
involved — without manually traversing edges. A molecule returns a topic's
sub-topics in full, while other connected lists are a representative sample for very
large neighborhoods; reach for `inspect`/`diagnose` when you need every edge or
suspect an entity is over-linked.

## Retrieval strategies

Two non-obvious habits make retrieval much more reliable.

### 1. Match the lookup to the field — concepts vs. state

Semantic `search` ranks by **meaning** in titles, descriptions, conclusions,
decisions, and intents. It does **not** consider a task's current *state* — status,
assignee, team, and project are not part of what it compares — so queries like
"blocked tasks" or "the platform team's open work" won't reliably surface through
`search`. Use the surface that matches the question:

- **Concept / "what do we know about X"** → `search "<natural language>"`.
- **State & structure / "which tasks are blocked, assigned to, or under team Y"** →
  `tasks list` filters (`--status`, `--assignee`, `--team`, `--project`,
  `--priority`). The `--topic`, `--intent`, and `--topic-category` filters go
  further: they walk the decision graph and return every task connected to that
  topic, intent, or category — retrieval you can't reach by phrasing alone.

### 2. Treat search hits as entry points, then expand through the decision hub

`search` is precise, not exhaustive: it returns only the **strongest matches per
entity type**, grouped by type, and drops weak ones. Two consequences:

- **One broad query under-recalls.** Run a few sharply-phrased queries (the exact
  term, a synonym, the opposite framing) rather than relying on a single one. If a
  type comes back empty, nothing cleared the relevance bar — rephrase instead of
  concluding the knowledge isn't there.
- **A hit is a doorway, not the whole room.** Expand it: `entity get` a **task** or
  **decision** hit to pull its decision-centric molecule; for a **topic or
  conclusion** hit, open the conclusions and the decisions on them with
  `topics inspect` (a plain `entity get` on a topic returns only its headline).
  Because the graph is decision-centric, the *why* behind any hit sits one hop away
  through its decisions — pivot there to turn a flat match into connected knowledge.

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

internode changes [--since <ISO>] [--types OITopic,OIIntent,OIDecision,OITask]
# Incremental change feed. Returns {id, type, change, at, content_hash} per
# changed root, where change is created | updated | archived. Omit --since for
# a FULL baseline (every live root + its content hash) — so a fresh consumer
# can seed its manifest in one call. With --since, you get only the delta.
# Use this instead of downloading the whole graph: persist content_hash per
# entity and diff it on the next run to find exactly what changed.

internode graph export [--include-deleted]
# Full graph dump for a local NetworkX MultiDiGraph mirror. Returns
# {nodes:[{id,labels,properties}], edges:[{source,target,type,properties}], ...}
# mapping 1:1 onto nx.MultiDiGraph (node key = id, edge key = relationship type).
# Schema-agnostic: labels/rels/properties are discovered dynamically and scoped
# by owner_id. The ONLY exclusion is the raw external work-item mirror (Linear /
# Jira Work* nodes), already duplicated into distilled OI. Version chains
# (UPDATED_TO) are collapsed to their live head — only the head node is exported,
# edges to superseded versions are re-pointed to the head. Soft-archived nodes
# excluded unless --include-deleted; vectors are never stored / always stripped.
# Build a local mirror once, then re-sync incrementally with `internode changes`.

internode graph schema [--include-deleted]
# Derive the schema of the exported graph FROM THE SAME version-collapsed graph:
# per label-set the node count + property keys (value types + how many nodes
# carry them); per relationship type the count, observed (from)->(to) endpoint
# pairings, and property types. Same exclusions as export. New labels surface
# automatically.
```



### List commands

All list commands return lightweight results: `{ items: [{ id, label }], total, limit, offset }` where `label` is a short display string for the entity.

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

`internode diagnose` returns the **real** edge counts so you can see structural noise. Where `entity get` shows a representative slice of an entity's relationships, the diagnostic commands report the complete counts.

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
- **Re-parenting:** `--parent <task_id>` makes this task a subtask of another; `--clear-parent` detaches it (becomes a root task). A task has at most one parent — re-parenting auto-detaches the old parent. Cycles and self-parenting are rejected (422).
- `tasks merge`**:** incoming decision edges (SPAWNS/BLOCKS/CANCELS/MODIFIES) re-point onto the target's current version; team/project ownership and subtask parent/child links re-parent onto the target; then the source is archived.



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

- `move` re-parents the sub-topic so it's owned by exactly one topic. The conclusion text (and its search index entry) is untouched.
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

> `link --task` **SPAWNS guard:** SPAWNS is auto-downgraded to MODIFIES when the target isn't the first task version, and BLOCKS/CANCELS/MODIFIES is auto-upgraded to SPAWNS when the target is a first version with no SPAWNS yet. The response carries a `note` field when this happens.



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

> **Preserve history with** `--data-date` **/** `--created-date`**:** every command that writes a new version accepts an optional `--data-date <ISO-8601>` (`2025-03-14` or `2025-03-14T10:00:00Z`). This covers `create` and `update` (topics, sub-topics, tasks, decisions, intents), the intent version-writing ops `set-scope`, `add-signal`, `remove-signal`, `consolidate`, and the per-version `version set-date` fix. It stamps the version with that historical date instead of "now", so the timeline reflects when the knowledge actually happened — critical when correcting backfilled or split data. The version is inserted into the chain at the correct point by date. You may pass `--data-date` alone (no content change) to append a date-corrected version. An unparseable value returns 422. In `split` plans, add a `"data_date"` key inside any `new_topic` / `new_decision` / `new_intent` object to backdate the entity it creates (otherwise it defaults to today, which distorts history). Non-versioned roots (projects, teams, statuses) use `--created-date` at create time and `set-created-date` to fix an existing root.



### Reviewing version history (decisions / intents / tasks)

Every other read resolves the single **head** version. To see the *whole* timeline of a single root — every version with its content, `data_date`, `created_at`, and `is_head` / `deleted` flags, ordered chronologically — use `history`:

```bash
internode decisions history <decision_id>
internode intents   history <intent_id>
internode tasks     history <task_id>
```

Use this to audit how an entity evolved, or to find the exact `version_id` to re-date, delete, or edit. (Topics don't have a single chain — their "versions" are sub-topics; use `topics inspect <id>` to list them.)

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

`version delete` is refused (422) when it would remove the only live version of an entity — archive the root instead. Find version ids with `entity get <root_id>`, `<entity> inspect <id>`, or `<entity> history <root_id>`.

#### Editing the *content* of a historical version in place (`version set-content`)

`set-content` overwrites content fields on **one specific version** — the escape hatch for fixing wrong/backfilled/imported text in history without appending a new version.

```bash
internode decisions version set-content <version_id> [--title ...] [--description ...] [--rationale ...] [--status ...] [--decision-maker email] [--type explicit|implicit] [--priority ...]
internode intents   version set-content <version_id> [--title ...] [--statement ...] [--scope ...] [--signal "phrase" --signal ...]
internode tasks     version set-content <version_id> [--title ...] [--description ...] [--priority ...] [--assignee email] [--due-date "YYYY-MM-DD"] [--blocked-by-reason ...] [--task-type ...]
```

> **This rewrites history — use it sparingly.** The graph is otherwise append-only by design; for routine revisions prefer `update` (appends a new version) and for genuinely bad versions prefer `version delete`. `set-content` exists for in-place corrections (typos, PII scrubbing, fixing imported junk).
>
> **Only the head feeds search.** The response includes `is_head` and `reembedded`. Editing a non-head version is an **audit-only** correction — it does *not* change semantic search (which follows the head). Only editing the head updates search. Editing a soft-deleted version is refused (422) — restore the root first. Only fields on the version node are editable; edge-backed task attributes (status/team/project/parent) are still changed via `tasks update`.



### Recovery — restore soft-deleted entities

Every archive/merge is reversible. Find and restore soft-deleted roots:

```bash
internode entity list-deleted [--labels OITopic,OIDecision,OIIntent,OITask] [--search "text"] [--limit N] [--offset N]
internode entity get <id> --include-deleted          # verify it's the right entity before restoring
internode entity restore <id> --label OITopic|OIDecision|OIIntent|OITask
# restore un-deletes the root and its versions and refreshes the search index.
```



### Repair — forked version chains

Single-lineage entities (OIDecision/OIIntent/OITask) keep one linear version history per root. A fork can split a chain into multiple heads, which breaks head resolution (reads and search rely on a single head). Diagnose with `diagnose version-chains`, then repair:

```bash
internode repair version-chains --dry-run                        # preview every forked root that would be repaired
internode repair version-chains                                  # re-linearize all forked roots by data_date
internode repair version-chains --labels OIDecision,OIIntent     # restrict to specific types
internode repair version-chains --ids oidecision_abc,oiintent_xyz # repair only these roots
```

Repair rebuilds a single date-ordered version chain with exactly one head, then refreshes the search index (the head may have changed). OITopic is intentionally **not** repairable this way — a topic fans out into many independent sub-topic lineages, so "multiple heads" is normal there.

### Embeddings — re-align the search index ("commit")

```bash
internode embeddings status
# Read-only drift report: knowledge graph vs. search index, per entity type. Safe any time.

internode embeddings sync [--scope all|OITopic|OIIntent|OITask|OIDecision|OIProject|ExternalSyncJob|OITopicVersion] [--ids id1,id2] [--since 2024-09-01T00:00:00Z] [--force] [--dry-run] [--no-wait] [--timeout N]
# Realign the search index after content-affecting changes. Default waits for
# completion (use --timeout to bound the wait); --no-wait backgrounds the work.
# --dry-run reports the plan without writing. --force re-indexes even when the
# content looks unchanged. --ids and --since are mutually exclusive.
```

Run `embeddings sync` after edits that change searchable content (titles, descriptions, conclusions, merges, splits, repairs). Most mutations refresh the search index automatically — `sync` is the explicit catch-up / drift-fixer.

### Gated Cypher runner

A user-only escape hatch for graph surgery the structured commands can't express. The agent drafts a `.cypher` file; the **user** reviews it and runs it, typing a per-owner passphrase the agent does not know — so an agent cannot execute the file it wrote.

```bash
internode cypher set-passphrase           # prompts twice for a passphrase (interactive terminal only)
internode cypher run <file.cypher>        # prompts for the passphrase, then executes blocks (separated by lines containing only ';')
internode cypher run <file.cypher> --dry-run   # validate guardrails (EXPLAIN) without executing any block
```

After a real run that mutates content, the response suggests `internode embeddings sync` to re-align the search index. Queries are owner-scoped and guardrailed (a denylist plus owner-id binding).

### Bulk text correction — fix a misspelling everywhere (`replace-text`)

Correct a misspelling or bad transcription across **every text property of every entity you own** in one call — titles, descriptions, conclusions, rationale, statements, scope, signals, names, notes, etc., across all roots *and* every version in history. This is the tool for "the transcript spelled it 'Qualeon' everywhere; it should be 'Qualcomm'".

```bash
internode replace-text "<search>" "<replacement>"            # dry-run preview (writes nothing)
internode replace-text "<search>" "<replacement>" --apply    # perform the in-place rewrite
internode replace-text "Qualeon" "Qualcomm" --apply
internode replace-text "teh customer" "the customer" --apply
```

Rules and safety:

- **Minimum 4 characters.** `search` must be at least 4 characters long; a shorter fragment is rejected with `BAD_INPUT`. Replacing tiny fragments graph-wide is almost never an intended correction and risks mass, unwanted edits.
- **Dry-run by default.** Without `--apply` the command previews the matched nodes/fields (`before`/`after`) and writes nothing — it reports `nodes_matched` / `properties_matched` plus a capped `sample`. Add `--apply` to commit (`nodes_changed` / `properties_changed`). Always preview first.
- **Case-sensitive, literal substring.** No regex, no case folding — `Qualeon` does not match `qualeon`.
- **Protected fields.** Identity and structural data are never touched: ids, owner ids, the denormalized team/task keys that back uniqueness constraints (`key`, `key_owner`, `short_key`, `short_id`, …), counters, content/embedding hashes, and timestamps. Only human-readable text (and string-list fields like intent `signals`) is rewritten.
- **In-place across all versions.** Because this rewrites the otherwise append-only history everywhere at once, use it for genuine data corrections only.
- **Re-align search afterward.** After an `--apply` run, run `internode embeddings sync` so semantic search reflects the corrected content (the response returns this as a `suggested_followup`).

#### Authoring guardrail-safe Cypher

The runner enforces **owner isolation**: **every OI node pattern must bind** `owner_id: $oid`. The runner injects the caller's owner id into `$oid` automatically — never hard-code an owner id literal (that fails `BAD_INPUT`). On a violation you get:

```json
{"ok":false,"error":{"code":"BAD_INPUT","message":"Every OI node pattern must bind owner_id: $oid for owner isolation. Offending pattern: (:OITopic)"}}
```

Rules to keep a script accepted:

- **Bind** `owner_id: $oid` **on every labeled OI node**, in *every* clause — `MATCH`, `OPTIONAL MATCH`, and inside variable-length / path patterns. Example: `MATCH (tp:OITopic {owner_id: $oid})-[:HAS_VERSION]->(v {owner_id: $oid})`.
- **Bind it on intermediate and target nodes too**, not just the anchor — e.g. `-[r:NEXT]->(b {owner_id: $oid})`, not `-[r:NEXT]->()`. A bare `()` or an unbound relationship endpoint that resolves to an OI node will be rejected.
- **The check scans the raw file text, including comments.** A literal node pattern like `(:OITopic)` written in a `//` comment trips the guardrail. Describe patterns in prose inside comments, or always include `{owner_id: $oid}` even in illustrative snippets.
- **Use** `$oid`**, never a literal UUID.** Owner scoping is provided by the runner; passing your own id is both unnecessary and blocked.
- **Split statements with a line containing only** `;`**.** Each block is validated and executed independently; `--dry-run` runs `EXPLAIN` on every block without writing.
- **Always** `--dry-run` **first** to clear the guardrail (binding + denylist) before the real run.



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

1. `diagnose` to find outliers / forks (uncapped counts).
2. `inspect` the worst offenders to see every edge.
3. `mutate` with the right primitive (`move` / `merge` / `split` / `link` / `unlink` / `normalize-edges` / `archive` / `repair`).
4. `diagnose` again to confirm.
5. `embeddings sync` to re-align semantic search.



### Decision tree: which primitive?


| Symptom                                                     | Right primitive                                                                                                               |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Sub-topic attached to the wrong topic root                  | `subtopics move <sub_id> --to-topic <correct_topic_id>`                                                                       |
| Sub-topic conclusion text needs revising                    | `subtopics update <sub_id> --conclusion "..." --type ...` (appends a new version)                                             |
| Two `OITopic` roots about the same subject                  | `topics merge <duplicate_id> --into <canonical_id>`                                                                           |
| One topic actually covers several distinct subjects         | `topics split <id> --file plan.json`                                                                                          |
| Two `OIIntent` roots about the same goal                    | `intents merge <duplicate_id> --into <canonical_id>`                                                                          |
| Several intents that should be one                          | `intents consolidate --into <target> --source <id> --source <id>`                                                             |
| One intent was falsely merged from several                  | `intents split <id> --file plan.json`                                                                                         |
| Two `OIDecision` roots about the same choice                | `decisions merge <duplicate_id> --into <canonical_id>`                                                                        |
| One decision conflates several distinct choices             | `decisions split <id> --file plan.json`                                                                                       |
| Two `OITask` roots that are the same task                   | `tasks merge <duplicate_id> --into <canonical_id>`                                                                            |
| A task belongs under a different parent task                | `tasks update <id> --parent <parent_id>` (or `--clear-parent`)                                                                |
| Decision linked to a sub-topic/task with the wrong rel-type | `decisions unlink ... --type WRONG` then `decisions link ... --type RIGHT`, or `decisions normalize-edges` for bulk conflicts |
| Decision linked to something unrelated                      | `decisions unlink <did> --sub-topic <stid>` (422 if it's the last sub-topic)                                                  |
| Decision/topic/intent/task is genuinely wrong               | `<entity> archive <id>` (reversible via `entity restore`)                                                                     |
| Accidentally archived something                             | `entity list-deleted` → `entity restore <id> --label <RootLabel>`                                                             |
| A decision/intent/task has multiple version heads           | `repair version-chains` (preview with `--dry-run` or `diagnose version-chains`)                                               |




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

For tasks, decisions, and sub-topics, `entity get` returns a **knowledge molecule** — the entity plus its decision-centric neighborhood (a topic's sub-topics come back in full, and getting a sub-topic id returns its parent topic's molecule; for very large neighborhoods the other connected lists are a representative sample — use `inspect` for the complete set). For other types, the full property set. See "Data Model → Knowledge molecules" for the per-type shape. Up to 20 IDs per call; response keyed by entity ID. IDs that fail to resolve return an `error` field.

### Mutations are validated for you

The service enforces allowed fields, entity types, and invariants (e.g. the decision invariant, single-parent tasks, split-target shape). Invalid input returns a `422` with a descriptive message — read it; it tells you exactly what to fix.

### Bulk corrections need a ≥4-char search and a dry-run first

`replace-text` rewrites a substring across every text property you own. The search string must be at least 4 characters, matching is case-sensitive, and it is a dry-run unless you pass `--apply`. Preview first, apply second, then `embeddings sync`. Structural ids / keys / hashes / timestamps are protected automatically, so a correction can never corrupt the graph.

### IDs are UUIDs

All entity IDs are UUID-style strings returned in `data` from list/get commands. Store and reuse them.

### Sub-topic types

Valid types: `Outcome`, `Problem`, `Constraint`, `Solution`, `Opportunity`, `Idea`, `Information`. Filter with `--type` on `subtopics list`.

### Topic categories

Topics are grouped into business categories (index 1-11): Strategy & Leadership, Product & Innovation, Technology & Engineering, People & Talent, Finance & Business Operations, Marketing & Brand, Sales & Revenue, Customer Success & Support, Legal & Regulatory, Data & Analytics, Other. Filter with `--category` on `topics list`.
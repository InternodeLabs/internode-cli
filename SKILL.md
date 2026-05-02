---
name: use-internode-cli
description: Interface with Internode Organizational Intelligence (OI) via the internode CLI. Use when the user asks to read, update tasks, search, or browse knowledge entities (topics, sub-topics, tasks, decisions, intents, teams, projects, statuses), or when bootstrapping context for a work session.
---

# Using the Internode CLI

The `internode` CLI is your interface to a user's **Organizational Intelligence (OI)** — a persistent knowledge graph of topics, sub-topics, tasks, decisions, intents, teams, projects, and statuses. Use it as long-term memory: read context before starting work, browse entities, update task properties, and search the knowledge graph.

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

The CLI is **read-heavy with structural-cleanup writes**:

- **Read all**: topics, sub-topics, tasks, decisions, intents, teams, projects, statuses
- **Diagnose** V2 reconciliation noise (uncapped edge counts) for decisions, topics, sub-topics, intents
- **Update scalar fields** on tasks **and** topics, sub-topics-via-move, decisions, intents
- **Reorganize edges**: move sub-topics between topics, link/unlink decision edges (sub-topic, task, intent), merge duplicate roots
- **Soft-archive** topics, sub-topics, decisions, intents (sets `deleted=true`; never hard-delete)
- **Create**: projects only

> **Hard delete is never available.** Every "archive" / "merge source" sets `deleted=true` so history stays traversable.

## Recommended Workflow

1. **Bootstrap context** at the start of a session:
   ```bash
   internode context --max-tokens 4000
   ```
   This returns a pre-formatted OI summary optimized for LLM consumption.

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

4. **Get details** on one or more entities by ID (returns knowledge molecule for tasks/decisions/sub-topics, full properties for everything else; max 20 IDs per call):
   ```bash
   internode entity get <id1> [<id2> ...]
   ```

5. **Update tasks** as you work — change status, assignee, team, project:
   ```bash
   internode tasks update <id> --status <status-id> --assignee "user@example.com"
   ```

6. **Clean up V2 noise** — see the "Cleaning up V2 reconciliation noise" section below for the diagnose → inspect → mutate loop.

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

internode entity get <id1> [<id2> ... <idN>]
# Get full details for up to 20 entities by ID. Returns knowledge molecule for
# tasks, decisions, sub-topics. Returns full properties for topics, intents,
# teams, projects, statuses. Response is keyed by entity ID.
```

### List Endpoints

All list commands return lightweight results: `{ items: [{ id, label }], total, limit, offset }` where `label` is the key property capped to 200 characters.

```bash
internode topics list [--category N] [--search "text"] [--limit N] [--offset N]
# List main topics. Filter by topic category index.

internode subtopics list [--type "Idea|Problem|Solution|..."] [--topic ID] [--limit N] [--offset N]
# List sub-topics (topic versions). Filter by type: Outcome, Problem, Constraint,
# Solution, Opportunity, Idea, Information.

internode tasks list [--team ID] [--project ID] [--status "name"] [--priority "..."] [--assignee "email"] [--search "text"] [--topic ID] [--intent ID] [--topic-category "index"] [--limit N] [--offset N]
# List tasks with PM and OI filters. topic, intent, and topic-category
# filter tasks through the decision graph.

internode decisions list [--search "text"] [--limit N] [--offset N]
# List decisions.

internode intents list [--limit N] [--offset N]
# List intents.

internode teams list
# List teams.

internode projects list [--team ID]
# List projects, optionally filtered by team.

internode statuses list [--team ID]
# List statuses, optionally filtered by team.
```

### Task Update

```bash
internode tasks update <id> [--title "..."] [--description "..."] [--priority "..."] [--assignee "email"] [--due-date "YYYY-MM-DD"] [--status ID] [--type "..."] [--team ID] [--project ID] [--user-notes "..."] [--blocked-by-reason "..."]
```

**Team/project changes:** When changing a task's team, incompatible project, status, and assignee are automatically cleared. The response includes a `cleared_fields` list when this happens. Projects are dependent on teams — ensure the target project belongs to the target team.

### Project Create

```bash
internode projects create --name "..." --team <team-id> [--key "..."] [--description "..."]
```

A project always belongs to a team (`--team` is required on create).

### Diagnostics (uncapped — find the noise)

`internode diagnose` returns the **real** edge counts so you can see V2 reconciliation noise. Unlike `entity get` (which caps related-entity lists at 4), the diagnostic endpoints are uncapped.

```bash
internode diagnose decisions [--by sub_topics|tasks|intents] [--top N] [--min-edges N] [--offset N]
# OIDecisions ranked by outgoing sub-topic / task / intent edges. Default --by sub_topics.

internode diagnose topics    [--by sub_topics|decisions]      [--top N] [--min-edges N] [--offset N]
# OITopic roots ranked by sub-topic count and the number of distinct decisions touching any sub-topic.

internode diagnose subtopics                                  [--top N] [--min-edges N] [--offset N]
# OITopicVersion sub-topics ranked by incoming decision-edge count.

internode diagnose intents                                    [--top N] [--min-edges N] [--offset N]
# OIIntents ranked by incoming SUPPORTS edge count from decisions.
```

### Inspecting a single entity (uncapped relationship dump)

`<entity> inspect <id>` returns the **full**, uncapped neighborhood for one root — every sub-topic, task, intent, or decision edge. Use this to plan a move/merge/unlink.

```bash
internode topics inspect    <topic_id>          # parent, sub-topic versions, decision links per sub-topic
internode subtopics inspect <sub_topic_id>      # parent topic + every incoming decision edge
internode decisions inspect <decision_id>       # every sub-topic, intent, and task edge
internode intents inspect   <intent_id>         # every supporting decision
```

### Topic mutations

```bash
internode topics update  <id> [--title "..."] [--description "..."] [--category 1..11] [--primary-contributor "email"]
internode topics archive <id>                                # soft-delete root + all versions
internode topics merge   <source_id> --into <target_id>      # re-parent every sub-topic version, then archive source
```

### Sub-topic mutations

```bash
internode subtopics move    <sub_topic_id> --to-topic <target_topic_id>
internode subtopics archive <sub_topic_id>
```

`move` atomically swaps `HAS_VERSION` so the version is owned by exactly one topic. The version's conclusion text (and embedding) is untouched.

### Decision mutations

```bash
internode decisions update  <id> [--title ...] [--description ...] [--rationale ...] [--status ...] [--decision-maker email] [--type explicit|implicit] [--priority ...]
internode decisions archive <id>
internode decisions merge   <source_id> --into <target_id>     # re-parent every sub-topic / task / intent edge, then archive source

# Edge link/unlink — pass exactly one of --sub-topic, --task, --intent
internode decisions link   <id> --sub-topic <stid> [--type RATIFIES|REJECTS|DEFERS]
internode decisions link   <id> --task <task_or_version_id> [--type SPAWNS|BLOCKS|CANCELS|MODIFIES]
internode decisions link   <id> --intent <intent_id>

internode decisions unlink <id> --sub-topic <stid> [--type RATIFIES|REJECTS|DEFERS]
internode decisions unlink <id> --task <task_or_version_id> [--type SPAWNS|BLOCKS|CANCELS|MODIFIES]
internode decisions unlink <id> --intent <intent_id>
```

> **Decision invariant (HTTP 422 on violation):** Every live OIDecision MUST keep ≥1 sub-topic edge AND ≥1 intent edge. If `unlink` would drop either to zero, it's blocked with a structured 422 response. The agent must add a replacement edge first, or call `merge` / `archive` to retire the decision explicitly.

> **`link --task` SPAWNS guard:** The backend reuses the V3 `_safe_link_decision_to_task_version` helper, so SPAWNS is auto-downgraded to MODIFIES when the target isn't the first task version, and BLOCKS/CANCELS/MODIFIES is auto-upgraded to SPAWNS when the target is a first version with no SPAWNS yet. The response carries a `note` field when this happens.

### Intent mutations

```bash
internode intents update  <id> [--title ...] [--statement ...] [--scope ...] [--signals "a,b,c"]
internode intents archive <id>
internode intents merge   <source_id> --into <target_id>      # re-parent every incoming SUPPORTS edge, then archive source
```

## Cleaning up V2 reconciliation noise

V2 reconciliation produced two recurring defects you may need to clean up before V3 takes over:

1. **Sub-topic noise under topics** — `OITopicVersion` nodes attached to the wrong `OITopic` root, plus duplicate roots about the same subject.
2. **Over-linked decisions** — single `OIDecision` roots carrying a very large fan-out of `RATIFIES|REJECTS|DEFERS` edges (and matching noise on `SUPPORTS` edges to intents).

### Decision invariant — read this first

Every live `OIDecision` must keep:

- **≥1 sub-topic edge** (`RATIFIES`, `REJECTS`, or `DEFERS` to an `OITopicVersion`)
- **≥1 intent edge** (`SUPPORTS` to an `OIIntent`)

If you try to `unlink` an edge whose removal would drop either count to zero, the backend returns **HTTP 422** with a structured error and refuses the change. To make progress in that case, **first add a replacement edge** (`link`), or call `decisions merge` to fold this decision into another, or `decisions archive` to retire it.

### Workflow loop

```text
diagnose  →  inspect  →  mutate  →  diagnose
```

1. **`diagnose`** to find outliers (uncapped counts).
2. **`inspect`** the worst offenders to see every edge.
3. **`mutate`** with the appropriate primitive (`move` / `merge` / `link` / `unlink` / `archive`).
4. **`diagnose`** again to confirm the count came down.

### Decision tree: move vs merge vs archive vs unlink

| Symptom | Right primitive |
|---|---|
| Sub-topic attached to the wrong topic root | `subtopics move <sub_id> --to-topic <correct_topic_id>` |
| Two `OITopic` roots about the same subject | `topics merge <duplicate_id> --into <canonical_id>` |
| Two `OIIntent` roots about the same goal | `intents merge <duplicate_id> --into <canonical_id>` |
| Two `OIDecision` roots about the same choice | `decisions merge <duplicate_id> --into <canonical_id>` |
| Decision is correctly linked to a sub-topic but with the wrong rel-type | `decisions unlink <did> --sub-topic <stid> --type WRONG` then `decisions link <did> --sub-topic <stid> --type RIGHT` |
| Decision linked to a sub-topic that doesn't actually relate to it | `decisions unlink <did> --sub-topic <stid>` (blocked with 422 if it's the last sub-topic) |
| Decision is genuinely wrong / over-linked beyond saving | `decisions archive <did>` |
| Sub-topic conclusion text is wrong | `subtopics archive <sub_id>` and let the chat layer / next transcript create a corrected one (sub-topic versions are append-only — never edit the conclusion in place) |

### Worked examples

**Find and inspect the worst over-linked decision**

```bash
internode diagnose decisions --by sub_topics --top 5
# → pick the worst id, e.g. oidecision_abc

internode decisions inspect oidecision_abc
# → shows every sub-topic, intent, task edge with parent topics
```

**Re-parent a mis-attached sub-topic**

```bash
internode subtopics inspect oitopicv_xyz
# → parent_topic_id is wrong

internode subtopics move oitopicv_xyz --to-topic oitopic_correct
```

**Merge two duplicate topics**

```bash
internode diagnose topics --by sub_topics --top 50
# → spot the duplicates by title

internode topics merge oitopic_dup --into oitopic_canonical
# → moves every sub-topic version, then archives oitopic_dup
```

**Unlink a noisy decision↔sub-topic edge (with invariant check)**

```bash
internode decisions inspect oidecision_abc
# → confirm there are still other sub-topic + intent edges

internode decisions unlink oidecision_abc --sub-topic oitopicv_noise --type RATIFIES
# → if this would leave 0 sub-topic edges, you'll get 422 — add a replacement first or archive the decision
```

**Resolve a 422 when the only sub-topic edge is wrong**

```bash
# Option A: link the correct sub-topic, THEN unlink the wrong one
internode decisions link   oidecision_abc --sub-topic oitopicv_correct --type RATIFIES
internode decisions unlink oidecision_abc --sub-topic oitopicv_wrong   --type RATIFIES

# Option B: archive the decision entirely
internode decisions archive oidecision_abc
```

## Key Patterns

### Parse output reliably

```bash
result=$(internode topics list 2>/dev/null)
if echo "$result" | jq -e '.ok' > /dev/null 2>&1; then
  echo "$result" | jq '.data'
fi
```

### Entity detail returns knowledge molecules

For tasks, decisions, and sub-topics, `entity get` returns a **knowledge molecule** — the entity plus its decision-centric neighborhood (related decisions, topics, intents, tasks). For other types, it returns the full property set. You can pass up to 20 IDs in a single call; the response is a dict keyed by entity ID. Entities that fail to resolve return an `error` field instead.

### Mutations are validated server-side

The API enforces allowed fields and entity types. If you send an invalid field, you get a `422` with a descriptive error. Read the error message — it tells you exactly what went wrong.

### IDs are UUIDs

All entity IDs are UUIDs returned in `data` from list/get commands. Store and reuse them.

### Sub-topic types

Sub-topics are typed conclusions attached to topics. Valid types: `Outcome`, `Problem`, `Constraint`, `Solution`, `Opportunity`, `Idea`, `Information`. Filter with `--type` on `subtopics list`.

### Topic categories

Topics are grouped into business categories (index 1-11): Strategy & Leadership, Product & Innovation, Technology & Engineering, People & Talent, Finance & Business Operations, Marketing & Brand, Sales & Revenue, Customer Success & Support, Legal & Regulatory, Data & Analytics, Other. Filter with `--category` on the topics list.

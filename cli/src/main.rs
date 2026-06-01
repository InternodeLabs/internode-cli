mod client;
mod commands;
mod config;
mod error;
mod output;

use clap::{Parser, Subcommand};

use crate::error::CliError;

#[derive(Parser)]
#[command(
    name = "internode",
    about = "Agent-native CLI for Internode Organizational Intelligence",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save your CLI API key (get one from Settings > CLI API Key in the portal)
    Configure {
        /// Your CLI API key (starts with ink_)
        api_key: String,
        /// Override the API base URL
        #[arg(long = "api-url")]
        api_url: Option<String>,
    },
    /// Authentication commands
    #[command(
        after_help = "Examples:
  internode auth status"
    )]
    Auth {
        #[command(subcommand)]
        command: AuthCmd,
    },
    /// Browse and reorganize OI topics
    #[command(
        after_help = "Examples:
  internode topics list
  internode topics list --category 3 --search \"deployment\" --limit 20 --offset 0
  internode topics inspect <topic_id>
  internode topics update  <topic_id> --title \"Pricing\" --category 5
  internode topics archive <topic_id>
  internode topics merge   <source_topic_id> --into <target_topic_id>

Tip:
  Run 'internode topics <subcommand> --help' for full argument details.
  See cleanup workflow in 'internode-cli/SKILL.md'."
    )]
    Topics {
        #[command(subcommand)]
        command: TopicsCmd,
    },
    /// Browse and reorganize OI sub-topics (Ideas, Problems, Solutions, etc.)
    #[command(
        after_help = "Examples:
  internode subtopics list
  internode subtopics list --type Idea --topic <topic_id> --limit 10
  internode subtopics inspect <sub_topic_id>
  internode subtopics move    <sub_topic_id> --to-topic <target_topic_id>
  internode subtopics archive <sub_topic_id>

Tip:
  Run 'internode subtopics <subcommand> --help' for full argument details."
    )]
    Subtopics {
        #[command(subcommand)]
        command: SubtopicsCmd,
    },
    /// Browse and update OI tasks
    #[command(
        after_help = "Examples:
  internode tasks list
  internode tasks list --team <team_id> --status \"In Progress\" --priority high
  internode tasks update <id> --status <status_id> --assignee \"user@example.com\"

Tips:
  Run 'internode tasks list --help' for list filters.
  Run 'internode tasks update --help' for mutation fields."
    )]
    Tasks {
        #[command(subcommand)]
        command: TasksCmd,
    },
    /// Browse and reorganize OI decisions
    #[command(
        after_help = "Examples:
  internode decisions list
  internode decisions list --search \"pricing model\" --limit 10
  internode decisions inspect <decision_id>
  internode decisions update  <decision_id> --title \"Adopt usage-based pricing\"
  internode decisions archive <decision_id>
  internode decisions merge   <source_decision_id> --into <target_decision_id>
  internode decisions link    <decision_id> --sub-topic <sub_topic_id> --type RATIFIES
  internode decisions link    <decision_id> --intent <intent_id>
  internode decisions link    <decision_id> --task <task_id_or_version_id> --type SPAWNS
  internode decisions unlink  <decision_id> --sub-topic <sub_topic_id>
  internode decisions unlink  <decision_id> --task <task_id_or_version_id> --type MODIFIES
  internode decisions unlink  <decision_id> --intent <intent_id>

Invariant:
  Every live decision MUST keep ≥1 sub-topic edge AND ≥1 intent edge.
  Unlinks that would drop either to zero are rejected (HTTP 422)."
    )]
    Decisions {
        #[command(subcommand)]
        command: DecisionsCmd,
    },
    /// Browse and reorganize OI intents
    #[command(
        after_help = "Examples:
  internode intents list
  internode intents list --limit 50 --offset 0
  internode intents inspect <intent_id>
  internode intents update  <intent_id> --title \"Increase ARR by 50%\" --signals \"ARR,growth\"
  internode intents archive <intent_id>
  internode intents merge   <source_intent_id> --into <target_intent_id>"
    )]
    Intents {
        #[command(subcommand)]
        command: IntentsCmd,
    },
    /// Diagnose V2 reconciliation noise (over-linked decisions, duplicate roots, etc.)
    #[command(
        after_help = "Examples:
  internode diagnose decisions --by sub_topics --top 20
  internode diagnose decisions --by tasks --min-edges 10
  internode diagnose topics --by sub_topics --top 30
  internode diagnose subtopics --min-edges 5
  internode diagnose intents --min-edges 10

Output is uncapped: each item carries the *real* edge count so the
agent can see the noise (vs. 'entity get' which caps lists at 4).
Use this BEFORE calling topics/subtopics/decisions/intents mutations."
    )]
    Diagnose {
        #[command(subcommand)]
        command: DiagnoseCmd,
    },
    /// Repair structural corruption in the OI graph
    #[command(
        after_help = "Examples:
  internode repair version-chains --dry-run
  internode repair version-chains
  internode repair version-chains --labels OIDecision,OIIntent
  internode repair version-chains --ids oidecision_abc,oiintent_xyz

Notes:
  version-chains re-linearizes forked version histories (multiple heads)
  for single-lineage entities (OIDecision/OIIntent/OITask) by data_date.
  Run 'internode diagnose version-chains' first to preview affected roots,
  or use --dry-run here. After a real repair, the API re-enqueues embeddings."
    )]
    Repair {
        #[command(subcommand)]
        command: RepairCmd,
    },
    /// Retrieve detailed entity information (knowledge molecules)
    #[command(
        after_help = "Examples:
  internode entity get oitask_123
  internode entity get oitask_123 oidecision_456 oitopicv_789
  internode entity get oitopic_abc --include-deleted
  internode entity list-deleted
  internode entity list-deleted --labels OITopic,OIIntent --search \"pricing\"
  internode entity restore oitopic_abc --label OITopic

Notes:
  Use one or more entity IDs (max 20) for 'get'.
  Do not pass bracket syntax like ['id']; pass raw IDs as arguments.
  For uncapped relationship dumps, use 'topics inspect', 'decisions inspect', etc.

  Recovery workflow:
    1. internode entity list-deleted
    2. internode entity get <id> --include-deleted   (verify the right entity)
    3. internode entity restore <id> --label OITopic (or matching root label)
    4. internode embeddings sync --ids <id>          (optional; restore already enqueues)

Tip:
  Run 'internode entity get --help' for argument details."
    )]
    Entity {
        #[command(subcommand)]
        command: EntityCmd,
    },
    /// Browse OI teams
    #[command(
        after_help = "Examples:
  internode teams list"
    )]
    Teams {
        #[command(subcommand)]
        command: TeamsCmd,
    },
    /// Browse and create OI projects
    #[command(
        after_help = "Examples:
  internode projects list
  internode projects list --team <team_id>
  internode projects create --name \"v2\" --team <team_id> --key PRJ --description \"Version 2\"

Tip:
  Run 'internode projects create --help' for create arguments."
    )]
    Projects {
        #[command(subcommand)]
        command: ProjectsCmd,
    },
    /// Browse OI statuses
    #[command(
        after_help = "Examples:
  internode statuses list
  internode statuses list --team <team_id>"
    )]
    Statuses {
        #[command(subcommand)]
        command: StatusesCmd,
    },
    /// Realign pgvector with Neo4j (drift report + sync). The "commit my changes"
    /// command for the OI knowledge graph.
    #[command(
        after_help = "Examples:
  internode embeddings status
  internode embeddings sync
  internode embeddings sync --scope OITopic
  internode embeddings sync --ids oitopic_abc,oidecision_xyz --force
  internode embeddings sync --since 2024-09-01T00:00:00Z
  internode embeddings sync --dry-run
  internode embeddings sync --no-wait

Notes:
  status   – read-only drift report; safe to run any time.
  sync     – default is synchronous (bounded by --timeout, default 120s).
             --no-wait drops items on the in-process background queue.
             --dry-run reports the plan without writing."
    )]
    Embeddings {
        #[command(subcommand)]
        command: EmbeddingsCmd,
    },
    /// Gated user-only Cypher runner.
    ///
    /// The chat agent drafts a .cypher file; the user reviews it; the
    /// user runs ``internode cypher run <file>`` and types a per-owner
    /// passphrase at the prompt.  Only the user knows the passphrase,
    /// so an agent cannot execute the file it wrote.
    #[command(
        after_help = "Examples:
  internode cypher set-passphrase
  internode cypher run cleanup.cypher
  internode cypher run cleanup.cypher --dry-run

Notes:
  - set-passphrase prompts twice; min length 12 characters.
  - run reads the file from disk, computes its SHA-256 client-side,
    and prompts for the passphrase.  The passphrase is never echoed
    and is sent only over the HTTPS connection to the API.
  - After a real run that mutates content, the API hint will suggest
    'internode embeddings sync' to re-align pgvector."
    )]
    Cypher {
        #[command(subcommand)]
        command: CypherCmd,
    },
    /// Semantic search over organizational intelligence
    Search {
        /// Search query text
        query: String,
    },
    /// Dump structural OI context optimized for LLM consumption
    Context {
        /// Maximum token budget for context output
        #[arg(long = "max-tokens")]
        max_tokens: Option<i64>,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Verify API key and show account info
    Status,
}

#[derive(Subcommand)]
enum TopicsCmd {
    /// List topics with optional filters
    List {
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
        #[arg(long)]
        search: Option<String>,
        /// Filter by topic category index
        #[arg(long)]
        category: Option<i64>,
    },
    /// Show uncapped relationship dump for one topic (sub-topics + incident decisions)
    Inspect {
        /// OITopic id
        id: String,
    },
    /// Update topic root-level fields (title, description, category)
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        /// Topic category index (1-11, see docs/USE_INTERNODE_CLI.md)
        #[arg(long)]
        category: Option<i64>,
        /// Primary contributor email
        #[arg(long = "primary-contributor")]
        primary_contributor: Option<String>,
    },
    /// Soft-archive a topic (sets deleted=true; sub-topics get re-parented before this)
    Archive {
        id: String,
    },
    /// Merge a duplicate source topic into a target topic
    Merge {
        /// Source topic id (will be re-parented then archived)
        source_id: String,
        /// Target topic id (will absorb every sub-topic version)
        #[arg(long = "into")]
        target_id: String,
    },
    /// Split a topic by re-parenting groups of sub-topic versions
    /// to existing and/or freshly-created target topics.
    ///
    /// The split plan is loaded from a JSON file: an array of entries
    /// each containing either ``target_topic_id`` or ``new_topic``
    /// ({topic_title, topic_description, category_index}) plus
    /// ``sub_topic_version_ids``.
    Split {
        /// Source OITopic id
        source_id: String,
        /// Path to a JSON file describing the split entries
        #[arg(long = "file")]
        splits_file: String,
        /// Skip archiving the source topic after splitting
        #[arg(long = "keep-source")]
        keep_source: bool,
        /// Print the plan without modifying Neo4j
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SubtopicsCmd {
    /// List sub-topics with optional filters
    List {
        /// Sub-topic type (Idea, Problem, Solution, Information, Outcome, etc.)
        #[arg(long = "type")]
        type_filter: Option<String>,
        /// Filter by parent topic ID
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Show uncapped parent topic + every incoming decision edge for one sub-topic
    Inspect {
        /// OITopicVersion id
        id: String,
    },
    /// Re-parent a sub-topic to a different OITopic root
    Move {
        /// OITopicVersion id
        id: String,
        /// Target OITopic id
        #[arg(long = "to-topic")]
        target_topic_id: String,
    },
    /// Soft-archive a single sub-topic version
    Archive {
        /// OITopicVersion id
        id: String,
    },
    /// Revise a sub-topic: appends a new version chained from the prior tail.
    Update {
        /// OITopicVersion id (the version to supersede)
        id: String,
        /// New conclusion text
        #[arg(long)]
        conclusion: Option<String>,
        /// New conclusion type (Idea, Problem, Solution, Information,
        /// Outcome, Opportunity, Constraint)
        #[arg(long = "type")]
        conclusion_type: Option<String>,
        /// Primary contributor email
        #[arg(long = "primary-contributor")]
        primary_contributor: Option<String>,
    },
}

#[derive(Subcommand)]
enum TasksCmd {
    /// List tasks with optional filters
    List {
        #[arg(long)]
        team: Option<String>,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        search: Option<String>,
        /// Filter by related topic ID
        #[arg(long)]
        topic: Option<String>,
        /// Filter by related intent ID
        #[arg(long)]
        intent: Option<String>,
        /// Filter by topic category
        #[arg(long = "topic-category")]
        topic_category: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Update an existing task
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assignee: Option<String>,
        #[arg(long = "due-date")]
        due_date: Option<String>,
        /// Status ID to assign
        #[arg(long)]
        status: Option<String>,
        /// Team ID to assign (auto-clears incompatible project/status/assignee)
        #[arg(long)]
        team: Option<String>,
        /// Project ID to assign (must belong to task's team)
        #[arg(long)]
        project: Option<String>,
        #[arg(long = "user-notes")]
        user_notes: Option<String>,
        #[arg(long = "blocked-by-reason")]
        blocked_by_reason: Option<String>,
        #[arg(long = "type")]
        task_type: Option<String>,
        /// Re-parent this task under another OITask (HAS_SUBTASK)
        #[arg(long)]
        parent: Option<String>,
        /// Detach this task from its current parent (make it a root task)
        #[arg(long = "clear-parent", default_value_t = false)]
        clear_parent: bool,
    },
    /// Soft-archive a task (sets deleted=true; reversible via 'entity restore')
    Archive {
        /// OITask id
        id: String,
    },
    /// Merge a duplicate source task into a target task
    Merge {
        /// Source task id (edges re-pointed to target, then archived)
        source_id: String,
        /// Target task id (absorbs decision edges, team/project, subtasks)
        #[arg(long = "into")]
        target_id: String,
    },
}

#[derive(Subcommand)]
enum DecisionsCmd {
    /// List decisions with optional filters
    List {
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Show uncapped relationship dump for one decision
    Inspect {
        /// OIDecision id
        id: String,
    },
    /// Update decision scalar fields
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        rationale: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long = "decision-maker")]
        decision_maker: Option<String>,
        /// Decision type (explicit | implicit)
        #[arg(long = "type")]
        decision_type: Option<String>,
        #[arg(long)]
        priority: Option<String>,
    },
    /// Soft-archive a decision
    Archive {
        id: String,
    },
    /// Merge a duplicate source decision into a target decision
    Merge {
        /// Source decision id (will be re-edged then archived)
        source_id: String,
        /// Target decision id (absorbs every sub-topic / task / intent edge)
        #[arg(long = "into")]
        target_id: String,
    },
    /// Add a single edge from this decision (sub-topic, task, or intent)
    Link {
        /// OIDecision id
        id: String,
        /// OITopicVersion id (kind=sub_topic)
        #[arg(long = "sub-topic")]
        sub_topic: Option<String>,
        /// OITask id or OITaskVersion id (kind=task)
        #[arg(long)]
        task: Option<String>,
        /// OIIntent id (kind=intent)
        #[arg(long)]
        intent: Option<String>,
        /// Relationship type. RATIFIES|REJECTS|DEFERS for sub-topic, SPAWNS|BLOCKS|CANCELS|MODIFIES for task. Defaults: RATIFIES (sub-topic), SPAWNS (task). Ignored for intent.
        #[arg(long = "type")]
        rel_type: Option<String>,
    },
    /// Remove a single edge from this decision (blocked if it would violate the invariant)
    Unlink {
        /// OIDecision id
        id: String,
        #[arg(long = "sub-topic")]
        sub_topic: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        intent: Option<String>,
        /// Optional rel-type filter. If omitted, every matching edge of any valid type is removed.
        #[arg(long = "type")]
        rel_type: Option<String>,
    },
    /// Split a decision by re-edging groups of sub-topic / task / intent
    /// edges to existing and/or freshly-created target decisions.
    ///
    /// Plan is loaded from a JSON file: an array of entries with either
    /// ``target_decision_id`` or ``new_decision`` (object with
    /// decision_title etc.) plus ``edges`` ([{kind, target_id, rel_type}]).
    Split {
        /// Source OIDecision id
        source_id: String,
        /// Path to a JSON file describing the split entries
        #[arg(long = "file")]
        splits_file: String,
        /// Skip archiving the source decision after splitting
        #[arg(long = "keep-source")]
        keep_source: bool,
        /// Print the plan without modifying Neo4j
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Collapse contradictory rel-types on the same (decision, target) pair
    NormalizeEdges {
        /// Restrict to a single decision id; omit to scan every live decision
        #[arg(long = "decision")]
        decision_id: Option<String>,
        /// Preference order for sub-topic rel-types
        /// (default: RATIFIES,REJECTS,DEFERS).  Omitted types are never kept.
        #[arg(long = "sub-topic-prefer", value_delimiter = ',')]
        sub_topic_prefer: Vec<String>,
        /// Preference order for task rel-types
        /// (default: SPAWNS,MODIFIES,BLOCKS,CANCELS).  Omitted types are never kept.
        #[arg(long = "task-prefer", value_delimiter = ',')]
        task_prefer: Vec<String>,
        /// Print the plan without modifying Neo4j
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum IntentsCmd {
    /// List intents
    List {
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Show uncapped supporting decisions for one intent
    Inspect {
        /// OIIntent id
        id: String,
    },
    /// Update intent scalar fields
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        statement: Option<String>,
        #[arg(long)]
        scope: Option<String>,
        /// Comma-separated list of signals (e.g. "ARR,churn,growth")
        #[arg(long)]
        signals: Option<String>,
    },
    /// Soft-archive an intent
    Archive {
        id: String,
    },
    /// Merge a duplicate source intent into a target intent
    Merge {
        /// Source intent id (will be re-supported then archived)
        source_id: String,
        /// Target intent id (absorbs every incoming SUPPORTS edge)
        #[arg(long = "into")]
        target_id: String,
    },
    /// Split an intent by re-pointing groups of supporting decisions to
    /// existing and/or freshly-created target intents (undo a false merge).
    ///
    /// Plan is loaded from a JSON file: an array of entries with either
    /// ``target_intent_id`` or ``new_intent`` (object with intent_title,
    /// statement, scope, signals) plus ``supporting_decision_ids``.
    Split {
        /// Source OIIntent id
        source_id: String,
        /// Path to a JSON file describing the split entries
        #[arg(long = "file")]
        splits_file: String,
        /// Skip archiving the source intent after splitting
        #[arg(long = "keep-source")]
        keep_source: bool,
        /// Print the plan without modifying Neo4j
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Add one or more signals to an intent (deduped, case-insensitive)
    AddSignal {
        /// OIIntent id
        id: String,
        /// Signal phrase (repeatable)
        #[arg(long = "signal", required = true)]
        signals: Vec<String>,
    },
    /// Remove one or more signals from an intent (matched case-insensitively)
    RemoveSignal {
        /// OIIntent id
        id: String,
        /// Signal phrase to remove (repeatable)
        #[arg(long = "signal", required = true)]
        signals: Vec<String>,
    },
    /// Set the scope on an intent (pass an empty string to clear)
    SetScope {
        /// OIIntent id
        id: String,
        /// New scope value
        scope: String,
    },
    /// Consolidate multiple source intents into a target intent
    Consolidate {
        /// Target OIIntent id (will absorb the sources)
        #[arg(long = "into")]
        target_id: String,
        /// Source OIIntent id (repeatable, one or more)
        #[arg(long = "source", required = true)]
        sources: Vec<String>,
        /// Statement merge strategy: keep_target (default) | first_non_empty
        #[arg(long = "statement-strategy", default_value = "keep_target")]
        statement_strategy: String,
        /// Scope merge strategy: keep_target (default) | first_non_empty
        #[arg(long = "scope-strategy", default_value = "keep_target")]
        scope_strategy: String,
        /// Signals merge strategy: union (default) | keep_target
        #[arg(long = "signals-strategy", default_value = "union")]
        signals_strategy: String,
        /// Print the plan without modifying Neo4j
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum DiagnoseCmd {
    /// Find OIDecisions with the most outgoing sub-topic / task / intent edges
    Decisions {
        /// Sort dimension: sub_topics (default) | tasks | intents
        #[arg(long = "by")]
        by: Option<String>,
        /// Top N rows to return (default 20, max 200)
        #[arg(long)]
        top: Option<i64>,
        /// Only include rows with at least this many total edges
        #[arg(long = "min-edges")]
        min_edges: Option<i64>,
        /// Skip the first N rows for paging
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Find OITopics with the most sub-topic versions or incident decisions
    Topics {
        /// Sort dimension: sub_topics (default) | decisions
        #[arg(long = "by")]
        by: Option<String>,
        #[arg(long)]
        top: Option<i64>,
        #[arg(long = "min-edges")]
        min_edges: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Find OITopicVersion sub-topics with the most incoming decision edges
    Subtopics {
        #[arg(long)]
        top: Option<i64>,
        #[arg(long = "min-edges")]
        min_edges: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Find OIIntents with the most supporting decisions (SUPPORTS fan-in)
    Intents {
        #[arg(long)]
        top: Option<i64>,
        #[arg(long = "min-edges")]
        min_edges: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Find single-lineage roots (OIDecision/OIIntent/OITask) whose version
    /// chain has forked into multiple heads (breaks head resolution downstream)
    VersionChains {
        /// Comma-separated subset of OIDecision,OIIntent,OITask
        #[arg(long)]
        labels: Option<String>,
        /// Max rows to return (default 100, max 1000)
        #[arg(long)]
        limit: Option<i64>,
    },
}

#[derive(Subcommand)]
enum RepairCmd {
    /// Re-linearize forked version chains (OIDecision/OIIntent/OITask)
    VersionChains {
        /// Comma-separated subset of OIDecision,OIIntent,OITask
        #[arg(long)]
        labels: Option<String>,
        /// Explicit root ids to repair (comma-separated). Omit to repair every
        /// forked root in the requested labels.
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
        /// Print the roots that would be repaired without writing
        #[arg(long = "dry-run", default_value_t = false)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum EntityCmd {
    /// Get detailed info for one or more entities (max 20)
    Get {
        /// Entity IDs to retrieve
        #[arg(required = true, num_args = 1..=20)]
        ids: Vec<String>,
        /// Also return soft-deleted entities as a minimal payload
        /// (use 'entity restore' to recover them)
        #[arg(long = "include-deleted", default_value_t = false)]
        include_deleted: bool,
    },
    /// List soft-deleted OI entities (restore candidates)
    ListDeleted {
        /// Comma-separated subset of OITopic,OIDecision,OIIntent,OITask
        #[arg(long)]
        labels: Option<String>,
        /// Substring filter on title/description (case-insensitive)
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        limit: Option<i64>,
        #[arg(long)]
        offset: Option<i64>,
    },
    /// Restore a single soft-deleted entity (and its versions). Re-enqueues
    /// pgvector embeddings so semantic search picks it up again.
    Restore {
        /// Entity ID
        id: String,
        /// Root label of the entity: OITopic | OIDecision | OIIntent | OITask
        #[arg(long)]
        label: String,
    },
}

#[derive(Subcommand)]
enum TeamsCmd {
    /// List teams
    List,
}

#[derive(Subcommand)]
enum ProjectsCmd {
    /// List projects
    List {
        #[arg(long)]
        team: Option<String>,
    },
    /// Create a new project
    Create {
        #[arg(long)]
        name: String,
        /// Team ID (required)
        #[arg(long)]
        team: String,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
enum StatusesCmd {
    /// List statuses
    List {
        #[arg(long)]
        team: Option<String>,
    },
}

#[derive(Subcommand)]
enum EmbeddingsCmd {
    /// Read-only drift report: Neo4j vs pgvector per entity type
    Status,
    /// Realign pgvector with Neo4j. Default is synchronous; use --no-wait
    /// to background.
    Sync {
        /// Entity-type scope: 'all' (default) | OITopic | OIIntent | OITask |
        /// OIDecision | OIProject | ExternalSyncJob | OITopicVersion
        #[arg(long)]
        scope: Option<String>,
        /// Explicit entity IDs (comma-separated). Mutually exclusive with --since.
        #[arg(long)]
        ids: Option<String>,
        /// ISO-8601 datetime — only re-embed entities updated_at >= this.
        #[arg(long)]
        since: Option<String>,
        /// Re-embed even when the v3 hash matches the stored embedded_text_hash
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Report the plan without writing to pgvector
        #[arg(long = "dry-run", default_value_t = false)]
        dry_run: bool,
        /// Enqueue on the in-process background queue and return immediately
        #[arg(long = "no-wait", default_value_t = false)]
        no_wait: bool,
        /// Synchronous-mode timeout in seconds (default 120, max 900)
        #[arg(long)]
        timeout: Option<i64>,
    },
}

#[derive(Subcommand)]
enum CypherCmd {
    /// Set or rotate the per-owner Cypher passphrase (min 12 chars)
    SetPassphrase,
    /// Run a Cypher file after prompting for the passphrase
    Run {
        /// Path to a .cypher file; blocks separated by lines containing only ';'
        file: String,
        /// Validate guardrails without executing any block
        #[arg(long = "dry-run", default_value_t = false)]
        dry_run: bool,
    },
}

async fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Configure { api_key, api_url } => {
            commands::auth::configure(&api_key, api_url.as_deref()).await
        }
        Commands::Auth { command } => match command {
            AuthCmd::Status => commands::auth::status().await,
        },
        Commands::Topics { command } => match command {
            TopicsCmd::List { limit, offset, search, category } => {
                commands::topics::list(limit, offset, search.as_deref(), category).await
            }
            TopicsCmd::Inspect { id } => commands::topics::inspect(&id).await,
            TopicsCmd::Update { id, title, description, category, primary_contributor } => {
                commands::topics::update(
                    &id,
                    title.as_deref(),
                    description.as_deref(),
                    category,
                    primary_contributor.as_deref(),
                )
                .await
            }
            TopicsCmd::Archive { id } => commands::topics::archive(&id).await,
            TopicsCmd::Merge { source_id, target_id } => {
                commands::topics::merge(&source_id, &target_id).await
            }
            TopicsCmd::Split {
                source_id,
                splits_file,
                keep_source,
                dry_run,
            } => {
                commands::topics::split(
                    &source_id,
                    &splits_file,
                    !keep_source,
                    dry_run,
                )
                .await
            }
        },
        Commands::Subtopics { command } => match command {
            SubtopicsCmd::List { type_filter, topic, limit, offset } => {
                commands::subtopics::list(type_filter.as_deref(), topic.as_deref(), limit, offset).await
            }
            SubtopicsCmd::Inspect { id } => commands::subtopics::inspect(&id).await,
            SubtopicsCmd::Move { id, target_topic_id } => {
                commands::subtopics::move_to(&id, &target_topic_id).await
            }
            SubtopicsCmd::Archive { id } => commands::subtopics::archive(&id).await,
            SubtopicsCmd::Update {
                id,
                conclusion,
                conclusion_type,
                primary_contributor,
            } => {
                commands::subtopics::update(
                    &id,
                    conclusion.as_deref(),
                    conclusion_type.as_deref(),
                    primary_contributor.as_deref(),
                )
                .await
            }
        },
        Commands::Tasks { command } => match command {
            TasksCmd::List { team, project, status, assignee, priority, search, topic, intent, topic_category, limit, offset } => {
                commands::tasks::list(
                    team.as_deref(), project.as_deref(), status.as_deref(),
                    assignee.as_deref(), priority.as_deref(), search.as_deref(),
                    topic.as_deref(), intent.as_deref(), topic_category.as_deref(),
                    limit, offset,
                ).await
            }
            TasksCmd::Update { id, title, description, priority, assignee, due_date, status, team, project, user_notes, blocked_by_reason, task_type, parent, clear_parent } => {
                commands::tasks::update(
                    &id, title.as_deref(), description.as_deref(), priority.as_deref(),
                    assignee.as_deref(), due_date.as_deref(), status.as_deref(),
                    team.as_deref(), project.as_deref(), user_notes.as_deref(),
                    blocked_by_reason.as_deref(), task_type.as_deref(),
                    parent.as_deref(), clear_parent,
                ).await
            }
            TasksCmd::Archive { id } => commands::tasks::archive(&id).await,
            TasksCmd::Merge { source_id, target_id } => {
                commands::tasks::merge(&source_id, &target_id).await
            }
        },
        Commands::Decisions { command } => match command {
            DecisionsCmd::List { search, limit, offset } => {
                commands::decisions::list(search.as_deref(), limit, offset).await
            }
            DecisionsCmd::Inspect { id } => commands::decisions::inspect(&id).await,
            DecisionsCmd::Update {
                id,
                title,
                description,
                rationale,
                status,
                decision_maker,
                decision_type,
                priority,
            } => {
                commands::decisions::update(
                    &id,
                    title.as_deref(),
                    description.as_deref(),
                    rationale.as_deref(),
                    status.as_deref(),
                    decision_maker.as_deref(),
                    decision_type.as_deref(),
                    priority.as_deref(),
                )
                .await
            }
            DecisionsCmd::Archive { id } => commands::decisions::archive(&id).await,
            DecisionsCmd::Merge { source_id, target_id } => {
                commands::decisions::merge(&source_id, &target_id).await
            }
            DecisionsCmd::Link { id, sub_topic, task, intent, rel_type } => {
                commands::decisions::link(
                    &id,
                    sub_topic.as_deref(),
                    task.as_deref(),
                    intent.as_deref(),
                    rel_type.as_deref(),
                )
                .await
            }
            DecisionsCmd::Unlink { id, sub_topic, task, intent, rel_type } => {
                commands::decisions::unlink(
                    &id,
                    sub_topic.as_deref(),
                    task.as_deref(),
                    intent.as_deref(),
                    rel_type.as_deref(),
                )
                .await
            }
            DecisionsCmd::NormalizeEdges {
                decision_id,
                sub_topic_prefer,
                task_prefer,
                dry_run,
            } => {
                commands::decisions::normalize_edges(
                    decision_id.as_deref(),
                    &sub_topic_prefer,
                    &task_prefer,
                    dry_run,
                )
                .await
            }
            DecisionsCmd::Split {
                source_id,
                splits_file,
                keep_source,
                dry_run,
            } => {
                commands::decisions::split(
                    &source_id,
                    &splits_file,
                    !keep_source,
                    dry_run,
                )
                .await
            }
        },
        Commands::Intents { command } => match command {
            IntentsCmd::List { limit, offset } => {
                commands::intents::list(limit, offset).await
            }
            IntentsCmd::Inspect { id } => commands::intents::inspect(&id).await,
            IntentsCmd::Update { id, title, statement, scope, signals } => {
                commands::intents::update(
                    &id,
                    title.as_deref(),
                    statement.as_deref(),
                    scope.as_deref(),
                    signals.as_deref(),
                )
                .await
            }
            IntentsCmd::Archive { id } => commands::intents::archive(&id).await,
            IntentsCmd::Merge { source_id, target_id } => {
                commands::intents::merge(&source_id, &target_id).await
            }
            IntentsCmd::Split {
                source_id,
                splits_file,
                keep_source,
                dry_run,
            } => {
                commands::intents::split(
                    &source_id,
                    &splits_file,
                    !keep_source,
                    dry_run,
                )
                .await
            }
            IntentsCmd::AddSignal { id, signals } => {
                commands::intents::add_signal(&id, &signals).await
            }
            IntentsCmd::RemoveSignal { id, signals } => {
                commands::intents::remove_signal(&id, &signals).await
            }
            IntentsCmd::SetScope { id, scope } => {
                commands::intents::set_scope(&id, &scope).await
            }
            IntentsCmd::Consolidate {
                target_id,
                sources,
                statement_strategy,
                scope_strategy,
                signals_strategy,
                dry_run,
            } => {
                commands::intents::consolidate(
                    &target_id,
                    &sources,
                    &statement_strategy,
                    &scope_strategy,
                    &signals_strategy,
                    dry_run,
                )
                .await
            }
        },
        Commands::Diagnose { command } => match command {
            DiagnoseCmd::Decisions { by, top, min_edges, offset } => {
                commands::diagnose::decisions(by.as_deref(), top, min_edges, offset).await
            }
            DiagnoseCmd::Topics { by, top, min_edges, offset } => {
                commands::diagnose::topics(by.as_deref(), top, min_edges, offset).await
            }
            DiagnoseCmd::Subtopics { top, min_edges, offset } => {
                commands::diagnose::subtopics(top, min_edges, offset).await
            }
            DiagnoseCmd::Intents { top, min_edges, offset } => {
                commands::diagnose::intents(top, min_edges, offset).await
            }
            DiagnoseCmd::VersionChains { labels, limit } => {
                commands::diagnose::version_chains(labels.as_deref(), limit).await
            }
        },
        Commands::Repair { command } => match command {
            RepairCmd::VersionChains { labels, ids, dry_run } => {
                commands::repair::version_chains(labels.as_deref(), &ids, dry_run).await
            }
        },
        Commands::Entity { command } => match command {
            EntityCmd::Get { ids, include_deleted } => commands::entity::get(ids, include_deleted).await,
            EntityCmd::ListDeleted { labels, search, limit, offset } => {
                commands::entity::list_deleted(
                    labels.as_deref(),
                    search.as_deref(),
                    limit,
                    offset,
                )
                .await
            }
            EntityCmd::Restore { id, label } => commands::entity::restore(&id, &label).await,
        },
        Commands::Teams { command } => match command {
            TeamsCmd::List => commands::teams::list().await,
        },
        Commands::Projects { command } => match command {
            ProjectsCmd::List { team } => commands::projects::list(team.as_deref()).await,
            ProjectsCmd::Create { name, team, key, description } => {
                commands::projects::create(&name, &team, key.as_deref(), description.as_deref()).await
            }
        },
        Commands::Statuses { command } => match command {
            StatusesCmd::List { team } => commands::statuses::list(team.as_deref()).await,
        },
        Commands::Embeddings { command } => match command {
            EmbeddingsCmd::Status => commands::embeddings::status().await,
            EmbeddingsCmd::Sync {
                scope,
                ids,
                since,
                force,
                dry_run,
                no_wait,
                timeout,
            } => {
                let id_list = ids.as_deref().map(|s| {
                    s.split(',')
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty())
                        .collect::<Vec<String>>()
                });
                commands::embeddings::sync(
                    scope.as_deref(),
                    id_list,
                    since.as_deref(),
                    force,
                    dry_run,
                    no_wait,
                    timeout,
                )
                .await
            }
        },
        Commands::Cypher { command } => match command {
            CypherCmd::SetPassphrase => commands::cypher::set_passphrase().await,
            CypherCmd::Run { file, dry_run } => commands::cypher::run(&file, dry_run).await,
        },
        Commands::Search { query } => {
            commands::search::search(&query).await
        }
        Commands::Context { max_tokens } => {
            commands::context::context(max_tokens).await
        }
    }
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = run(cli).await;
    match result {
        Ok(()) => std::process::ExitCode::from(0),
        Err(e) => {
            output::print_error(&e);
            e.exit_code().into()
        }
    }
}

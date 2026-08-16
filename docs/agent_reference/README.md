# Agent Design References

This note records external projects that inform the future KimPeanut Research
agent runtime. They are design references only. KimPeanut will not vendor,
copy, or make its architecture depend on these projects.

The selection deliberately combines a durable-workflow project, a professional
typed agent SDK, a small readable implementation, and a production coding-agent
control surface. Repository activity and public popularity were checked on
2026-08-15; popularity is only a discovery signal, not an adoption criterion.

## Chosen references

| Project | Why it is useful | Core idea to retain | Boundary for KimPeanut |
| --- | --- | --- | --- |
| [LangGraph](https://github.com/langchain-ai/langgraph) | Mature, widely used low-level orchestration framework | Treat a run as a state machine with persisted checkpoints, explicit transitions, interrupts, and resumability. | Implement run state and checkpoints in SQLite; do not add a graph-runtime dependency or a server. |
| [PydanticAI](https://github.com/pydantic/pydantic-ai) | Professional SDK focused on validated tools and structured results | A tool is a typed contract: validated input, explicit dependencies, bounded output, and observable execution. | Model Rust tool schemas and results with enums/structs; model output never writes source facts directly. |
| [smolagents](https://github.com/huggingface/smolagents) | Intentionally small implementation whose core agent logic stays readable | Keep the loop, tool interface, and execution record small enough to inspect and test. Sandboxing is a separate capability. | Begin with one sequential loop and a narrow registry; no arbitrary code execution, shell, or filesystem tool. |
| [OpenHands](https://github.com/OpenHands/OpenHands) | Production-oriented agent control environment with local and sandboxed operating modes | Runtime environment is a security decision. Capabilities, isolation, and user-visible warnings must be first-class. | Consequential actions require policy approval and audit events; future repository/experiment execution must use a dedicated sandbox. |

## Derived architecture for KimPeanut

The application architecture remains the authority:

```text
Vue agent UI
  -> Tauri command
  -> Rust agent runtime
  -> tool registry -> policy decision -> domain service -> SQLite / local storage
                                          \-> audit event
```

The runtime is a client of the research core. It may create a derived
interpretation, but it may not bypass repository rules, mutate source evidence,
or obtain ambient file, shell, database, or network access.

### Minimum viable runtime

Build Phase 4 as a small, inspectable Rust module:

- `ResearchRun`: immutable run identity, task, model/parser identifiers,
  lifecycle status, and timestamps.
- `RunStep`: ordered model, tool, policy, and user-review events; each stores
  input/output summaries and references to full artifacts where appropriate.
- `ToolDefinition`: stable name, typed request/response, declared capability,
  and an owning Rust service. No generic `sql`, `shell`, `filesystem`, or
  unrestricted `http` tool.
- `PolicyDecision`: allow, deny, or require user approval, with the evaluated
  capability, reason, and decision timestamp.
- `Checkpoint`: enough state to resume a paused run without reconstructing
  hidden model context. Store it as a versioned derived artifact linked to the
  run.

The first useful tools should be read-only and source-aware:
`search_papers`, `get_paper_detail`, `get_evidence`, `find_related_papers`, and
`draft_claim`. `draft_claim` creates an interpretation requiring user review;
it never upgrades itself to evidence or a source fact.

### State machine, not an unbounded loop

Use an explicit lifecycle such as:

```text
queued -> running -> waiting_for_approval -> running -> completed
                  \-> cancelled | failed
```

Persist after every state transition and every tool result. A restart can then
resume only from a known checkpoint, and a user can see exactly why a run is
waiting. Retrying a tool needs an idempotency key or an explicit new attempt so
that a failure cannot silently duplicate mutations.

### Tool and data invariants

- Validate tool requests before a domain service is called; validate results
  before the model sees them.
- Scope every read by explicit paper, collection, or run identifiers. Return
  reference IDs and excerpts, not broad database access.
- Attach paper/source location, producer version, and creation time to derived
  records. Preserve `Source -> Evidence -> Claim -> Interpretation ->
  Hypothesis`.
- Keep working context transient and bounded. Durable research data belongs in
  SQLite/local artifacts; model conversation history is neither evidence nor
  authoritative memory.
- Record policy denials and approval decisions as carefully as successful calls.
- Treat PDF text, webpages, repository files, metadata, and model output as
  untrusted data, never as executable instructions.

### Deferred capabilities

Do not import the following ideas until there is a concrete product need:

- Multi-agent delegation and graph scheduling.
- Remote agent servers, queues, and cloud deployment.
- General code execution, unrestricted network fetches, or repository writes.
- Vector databases and autonomous long-term memory.

When Phase 5 needs code or experiment execution, add a dedicated sandboxed
executor behind a separate policy capability. It must expose a narrow command
set, explicit workspace boundaries, captured outputs, and reviewable mutations.

## How to use this reference

Before adding an agent feature, answer these questions in its design note and
tests:

1. Which narrow tool owns the operation, and which Rust domain service executes it?
2. What capability and approval level are required?
3. Which source/evidence/provenance records are read or created?
4. At which transition is progress checkpointed, and how is retry idempotent?
5. What does the user inspect before a consequential mutation is applied?

If a proposed feature cannot answer these questions, keep it outside the agent
runtime until the required domain and policy foundations exist.

## Source notes

- LangGraph describes durable execution, human-in-the-loop interruption,
  persistence, and run visibility in its project overview.
- PydanticAI demonstrates typed tool inputs, validated structured output, and
  explicit dependency context.
- smolagents makes its intentionally minimal agent core and optional sandboxed
  code execution visible in its README.
- OpenHands documents both local operation and a warning that an unsandboxed
  agent server receives full filesystem access; this is a useful negative
  constraint for KimPeanut.

Read the repositories and their licenses directly before borrowing a concrete
algorithm, data model, or implementation detail.

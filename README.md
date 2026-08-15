# KimPeanut Research

> A lightweight, local-first research environment for computer science.
>
> **Read papers. Understand methods. Work with code. Run experiments. Build knowledge. Let an agent drive the research loop.**

KimPeanut Research is a desktop application designed for computer science research. Its long-term goal is to provide an agent that can progressively drive a research task—from discovering and reading literature, to analyzing methods, working with code, running experiments, and maintaining a traceable research knowledge base.

The project starts with a simpler and more immediately useful foundation: a **Research Library** and **Paper Analysis System**. The agent is built on top of these capabilities rather than being tightly coupled to them.

## Vision

Research should be a continuous loop rather than a collection of disconnected chat sessions:

```text
Research Question
      ↓
Search Literature
      ↓
Collect & Organize Papers
      ↓
Read & Analyze
      ↓
Extract Evidence / Methods / Results
      ↓
Identify Gaps & Hypotheses
      ↓
Find or Write Code
      ↓
Run Experiments
      ↓
Analyze Results
      ↓
Update Knowledge
      ↓
Search Again
```

KimPeanut Research aims to make this loop persistent, searchable, auditable, and eventually agent-driven.

## Core Principles

### 1. The Research Library comes first

The library is the durable foundation of the system. It should remain useful even without an autonomous agent.

The same library can support:

- manual research
- AI-assisted paper analysis
- literature review
- knowledge discovery
- coding workflows
- experiment tracking
- autonomous research agents

### 2. The agent is a user of the system

The LLM should not own the research state or directly manipulate internal storage. It should operate through well-defined tools and services.

```text
              Research Agent
                    │
                    ▼
               Tool Registry
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
    Paper Tools  Search Tools  Code Tools
        │           │           │
        └───────────┼───────────┘
                    ▼
             Research Core
                    │
                    ▼
             Research Library
```

### 3. Evidence must be traceable

AI-generated interpretations should never be silently treated as established facts.

The system distinguishes between:

```text
Source
  ↓
Evidence
  ↓
Claim
  ↓
Interpretation
  ↓
Hypothesis
```

A claim should be able to point back to its source paper and, where possible, the relevant page, section, table, figure, or citation context.

### 4. Untrusted research content stays untrusted

Papers, webpages, repositories, and other external content may contain malicious or misleading instructions. Retrieved content is treated as **data**, not as authority over the agent.

The agent operates through controlled tools, permission checks, provenance tracking, and audit logs.

### 5. Local-first and lightweight

KimPeanut Research is intended to be a desktop application with a lightweight native core. Local storage and local processing should work without requiring a permanent backend service.

Cloud APIs and local models can be used where appropriate, but they should be optional components rather than fundamental dependencies.

## Main Components

### Research Library

The persistent research knowledge base.

- Papers
- Authors
- Venues
- Topics
- Tags and collections
- Sections
- Claims
- Evidence
- Methods
- Datasets
- Experiments
- Citations
- Code repositories
- Research tasks
- Notes

### Paper Manager

Handles the paper lifecycle.

```text
Import
  ↓
Identify
  ↓
Fetch Metadata
  ↓
Store PDF
  ↓
Parse
  ↓
Index
  ↓
Analyze
```

Typical capabilities include:

- PDF import
- DOI / arXiv / URL lookup
- metadata retrieval
- PDF storage and caching
- full-text search
- tagging and grouping
- citation relationships
- related-paper discovery

### Paper Analysis

Provides structured analysis instead of only generating a long summary.

Example analysis targets:

```text
Problem
Contribution
Method
Architecture / Algorithm
Dataset
Training / Experimental Setup
Metrics
Results
Limitations
Claims
Evidence
```

### Research Knowledge Graph

Papers are connected to the concepts and artifacts they describe.

```text
Paper
 ├── cites → Paper
 ├── uses → Dataset
 ├── proposes → Method
 ├── reports → Experiment
 ├── supports → Claim
 └── implements → Repository
```

The initial implementation can use a relational database and relationship tables. A dedicated graph database is not required for the first versions.

### Research Tools

Tools are the stable interface between the agent and the research environment.

Examples:

```text
search_papers()
fetch_paper()
import_paper()
parse_paper()
summarize_paper()
extract_method()
extract_experiments()
extract_claims()
compare_papers()
find_related_papers()
search_library()
create_research_task()
```

### Research Agent

The final goal is an agent capable of driving research tasks through the available tools.

A typical research run may look like:

```text
Question
  ↓
Plan
  ↓
Search
  ↓
Screen
  ↓
Read
  ↓
Extract Evidence
  ↓
Compare
  ↓
Find Research Gaps
  ↓
Plan Experiment
  ↓
Work with Code
  ↓
Run Experiment
  ↓
Analyze Result
  ↓
Update Research State
```

## Architecture

The project is planned as a Rust + Tauri desktop application.

```text
┌─────────────────────────────────────────────────────────────┐
│                         Tauri UI                            │
│                  Vue / TypeScript frontend                  │
└────────────────────────────┬────────────────────────────────┘
                             │ IPC
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                       Rust Core                             │
│                                                             │
│  Research Library                                           │
│  Paper Service                                              │
│  Search Service                                             │
│  Analysis Engine                                            │
│  Tool Registry                                              │
│  Agent Runtime                                               │
│  Permission / Policy                                        │
│  Provenance / Audit                                         │
└───────────────┬─────────────────────────┬───────────────────┘
                │                         │
                ▼                         ▼
        Local Research DB          External Services
             SQLite               OpenAlex / Semantic Scholar
                                  Crossref / Unpaywall / arXiv
                │
                ▼
          Local Research Files
        PDFs / extracted text / cache
```

The agent runtime is intentionally separated from the application UI and the research data layer. This allows the core library to remain useful with no agent at all.

## Security Model

Security is part of the architecture rather than a later feature.

### Untrusted content

Treat external material as untrusted input:

```text
Web / PDF / Repository
        ↓
     Ingestion
        ↓
   Parse / Extract
        ↓
   Provenance Store
        ↓
   Research State
```

Content from a paper must not be able to redefine agent instructions or obtain arbitrary tool permissions.

### Capability-based tools

Tools should expose narrow capabilities instead of unrestricted access.

For example:

```text
search_papers
    network: yes
    database_write: no
    process_execution: no

create_claim
    network: no
    database_write: yes
    process_execution: no

run_experiment
    network: policy-controlled
    database_write: yes
    process_execution: yes
```

High-impact operations should support explicit approval and policy checks.

### Auditability

Agent runs should be inspectable:

```text
Research Run
 ├── search actions
 ├── retrieved sources
 ├── analysis steps
 ├── tool calls
 ├── mutations
 ├── experiments
 └── final conclusions
```

The system should be able to answer **why** a conclusion exists and **which evidence supports it**.

## Technology Direction

The initial technology direction is:

| Area | Technology |
|---|---|
| Desktop | Tauri 2 |
| UI | Vue 3 + TypeScript |
| Core | Rust |
| Database | SQLite |
| Full-text search | SQLite FTS5 |
| PDF / scientific parsing | GROBID or specialized parsers |
| Scholarly metadata | OpenAlex / Semantic Scholar / Crossref |
| Open-access discovery | Unpaywall / repositories |
| Local LLM | llama.cpp or another local inference runtime |
| Vector retrieval | Add when needed |

The project should avoid unnecessary infrastructure. A local SQLite database is sufficient for the early library and analysis phases.

## Development Roadmap

### Phase 1 — Research Library

- [ ] Rust application core
- [ ] SQLite schema and migrations
- [ ] Paper model
- [ ] PDF import and storage
- [ ] Metadata management
- [ ] Tags and collections
- [ ] Full-text search
- [ ] Basic citation relationships

### Phase 2 — Paper Analysis

- [ ] Structured paper parsing
- [ ] Abstract / section extraction
- [ ] Quick paper screening
- [ ] Method extraction
- [ ] Experiment extraction
- [ ] Claim and evidence extraction
- [ ] Paper comparison

### Phase 3 — Research Intelligence

- [ ] Topic clustering
- [ ] Citation graph views
- [ ] Related-paper discovery
- [ ] Research questions
- [ ] Research notes
- [ ] Research task state

### Phase 4 — Agent Runtime

- [ ] Tool registry
- [ ] Agent state
- [ ] Planning loop
- [ ] Tool permissions
- [ ] Provenance checks
- [ ] Audit log
- [ ] Human approval workflow

### Phase 5 — Computer Science Research Agent

- [ ] Repository inspection
- [ ] Code search
- [ ] Code modification
- [ ] Build / test execution
- [ ] Experiment execution
- [ ] Result analysis
- [ ] Autonomous literature ↔ code ↔ experiment loop

## Repository Philosophy

KimPeanut Research should make the following distinction explicit:

```text
Research Data
    ≠
Model Memory
    ≠
Agent Interpretation
```

The research library is the persistent source of truth for the application. Models are reasoning components that operate on that state; they are not the state itself.

This makes it possible to replace or upgrade models, agent frameworks, and external services without losing the accumulated research knowledge.

## Status

Early architecture / initial development.

The first target is a practical paper management and paper analysis system. Autonomous research is the long-term goal, but the library should remain independently useful throughout development.

## License

TBD.

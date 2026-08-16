# KimPeanut frontend guide

This directory records durable UI decisions so frontend work stays coherent across sessions.

KimPeanut is a focused research workspace: clean, modern, calm, and optimized for reading and scanning literature. Favor neutral surfaces, restrained accent colors, typographic hierarchy, and compact metadata over dashboard decoration.

Use `App.vue` as a composition root. Put page features under `src/features/<feature>/` and split meaningful regions into typed Vue components. Vue presents state and calls narrow Tauri-facing services; it does not own persistence, database access, or research-file operations.

Page references:

- [Paper manager](paper-manager.md) — current library page layout and visual decisions.

Keep the matching page reference synchronized while coding. Update it when page structure, visual language, component boundaries, or interaction policy changes; do not wait until a later cleanup pass.

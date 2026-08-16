# Paper manager

The paper manager is the first useful research-library surface. It supports fast collection scanning while keeping the selected paper's source context visible.

Regions:

- Sidebar: library navigation, collections, local-storage status, a bottom
  settings control, and user identity.
- Library pane: collection context, search/filter affordances, paper rows, and lightweight metadata.
- Detail pane: selected paper cover, bibliographic metadata, PDF affordance, abstract, notes, tags, and source provenance.

Visual decisions:

- Use a dark navigation rail against warm off-white content surfaces.
- Use serif display titles and compact sans/mono text for metadata and labels.
- Use muted lavender, sand, blue, green, and mauve accents.
- Keep rows scannable: title, authors, venue/recency, then tags.
- Keep source/provenance visible; imported material is distinct from interpretation.

The **Import papers** control opens a native PDF-only picker. It delegates the
selected path to `src/features/library/services/localPdfImport.ts`, which calls
the Rust `import_local_pdf` command; Vue never copies or reads the research
file. It is intentionally unavailable from a normal browser visiting the Vite
development URL. The button reports copying, success, cancellation, or failure inline.
It uses a pointer cursor and a hover hint describing the local-PDF action.
This first import slice only creates the managed PDF and does not yet add it to
the fixture-backed list. Fixtures live in `src/features/library/mockPaperData.ts`,
models in `src/features/library/types.ts`, and page regions in
`src/features/library/components/`.

The bottom-left **Settings** control opens a General settings dialog. Its data
storage card reads the backend-owned data location and lets a user select an
empty destination folder. The backend copies managed files, retains the
previous location for recovery, and switches future imports to the new one.
The migration runs off the UI thread; after it succeeds, the dialog closes and
returns the user to the library. The dialog is mounted only while open; its
close button and a click on the backdrop both return to the library.
The dialog lives in `src/features/settings/components/SettingsDialog.vue`,
while `App.vue` owns its open/close state.

<script setup lang="ts">
import { ref } from "vue";

import { collectionDescription, collectionTitle, papers } from "../mockPaperData";
import { selectAndImportLocalPdf } from "../services/localPdfImport";
import PaperList from "./PaperList.vue";

const importState = ref<"idle" | "importing" | "success" | "error">("idle");
const importMessage = ref("");

async function importPdf() {
  importState.value = "importing";
  importMessage.value = "Copying PDF into your local library…";

  try {
    const imported = await selectAndImportLocalPdf();
    if (!imported) {
      importState.value = "idle";
      importMessage.value = "";
      return;
    }

    importState.value = "success";
    importMessage.value = `${imported.sourceFileName} copied to local library.`;
  } catch (error) {
    importState.value = "error";
    importMessage.value = error instanceof Error ? error.message : "Could not import this PDF.";
  }
}
</script>

<template>
  <section class="library-pane">
    <header class="topbar"><div class="crumb"><span>Library</span><span class="crumb-arrow">/</span><strong>{{ collectionTitle }}</strong></div><div class="top-actions"><button class="icon-button" aria-label="Help">?</button><button class="icon-button" aria-label="Settings">⚙</button></div></header>
    <div class="library-content">
      <div class="page-heading"><div><p class="eyebrow">PERSONAL RESEARCH LIBRARY</p><h1>{{ collectionTitle }}</h1><p class="subtitle">{{ collectionDescription }}</p></div><button class="import-button" title="Select a local PDF to copy into your library" :disabled="importState === 'importing'" @click="importPdf"><span>+</span>{{ importState === "importing" ? "Importing…" : "Import papers" }}</button></div>
      <p v-if="importMessage" class="import-status" :class="`is-${importState}`" role="status">{{ importMessage }}</p>
      <div class="toolbar"><label class="search"><span>⌕</span><input aria-label="Search papers" placeholder="Search this collection" /></label><button class="filter-button">All papers <span>⌄</span></button><button class="view-button active-view" aria-label="List view">☷</button><button class="view-button" aria-label="Grid view">▦</button></div>
      <PaperList :papers="papers" />
    </div>
  </section>
</template>

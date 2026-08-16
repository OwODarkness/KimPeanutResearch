<script setup lang="ts">
import { ref } from "vue";

import AppSidebar from "./features/library/components/AppSidebar.vue";
import LibraryPane from "./features/library/components/LibraryPane.vue";
import PaperDetailPanel from "./features/library/components/PaperDetailPanel.vue";
import { selectedPaper } from "./features/library/mockPaperData";
import SettingsDialog from "./features/settings/components/SettingsDialog.vue";

import "./styles/paper-manager.css";

const settingsOpen = ref(false);

function openSettings() {
  settingsOpen.value = true;
}

function closeSettings() {
  settingsOpen.value = false;
}
</script>

<template>
  <main class="app-shell">
    <AppSidebar @open-settings="openSettings" />
    <LibraryPane />
    <PaperDetailPanel :paper="selectedPaper" />
    <SettingsDialog v-if="settingsOpen" @close="closeSettings" />
  </main>
</template>

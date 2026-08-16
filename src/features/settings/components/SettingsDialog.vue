<script setup lang="ts">
import { onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { getStorageLocation, migrateStorageLocation, type StorageLocation } from "../services/storageSettings";

const emit = defineEmits<{ close: [] }>();

const storageNotice = ref("");
const storageLocation = ref<StorageLocation | null>(null);
const changingStorage = ref(false);

async function loadStorageLocation() {
  try { storageLocation.value = await getStorageLocation(); } catch { storageNotice.value = "Storage settings are available only in the desktop app."; }
}

onMounted(() => { void loadStorageLocation(); });

function closeSettings() {
  emit("close");
}

async function chooseStorageFolder() {
  const destination = await open({ directory: true, multiple: false, title: "Choose an empty folder for KimPeanut Research" });
  if (!destination || Array.isArray(destination)) return;
  if (!window.confirm("Copy the managed library to this empty folder and use it for future imports? The previous location will be kept as a recovery copy.")) return;
  changingStorage.value = true;
  storageNotice.value = "Copying managed library data to the new location…";
  try {
    const receipt = await migrateStorageLocation(destination);
    storageLocation.value = { dataDir: receipt.dataDir, layoutVersion: 1, usesCustomLocation: true };
    storageNotice.value = `Storage moved to the selected folder. ${receipt.copiedEntries} file${receipt.copiedEntries === 1 ? "" : "s"} copied; the previous location was retained as a recovery copy.`;
    closeSettings();
  } catch (error) { storageNotice.value = error instanceof Error ? error.message : "Could not change the storage location."; }
  finally { changingStorage.value = false; }
}
</script>

<template>
  <div class="settings-backdrop" @click.self="closeSettings">
    <section class="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title">
      <header class="settings-header">
        <div>
          <p class="eyebrow">WORKSPACE SETTINGS</p>
          <h2 id="settings-title">General</h2>
        </div>
        <button type="button" class="settings-close" aria-label="Close settings" @click.stop="closeSettings">×</button>
      </header>

      <div class="settings-content">
        <section class="settings-card">
          <div class="settings-card-icon">⌂</div>
          <div class="settings-card-copy">
            <h3>Data storage</h3>
            <p>Keep papers, extracted text, and library records in a location you control.</p>
            <span class="settings-location">{{ storageLocation?.dataDir ?? "Loading storage location…" }}</span>
          </div>
          <button class="settings-secondary-button" :disabled="changingStorage" @click="chooseStorageFolder">{{ changingStorage ? "Moving…" : "Choose folder" }}</button>
        </section>
        <p v-if="storageNotice" class="settings-notice" role="status">{{ storageNotice }}</p>

        <section class="settings-card settings-card-muted">
          <div class="settings-card-icon">◌</div>
          <div class="settings-card-copy">
            <h3>About this library</h3>
            <p>Storage layout version {{ storageLocation?.layoutVersion ?? 1 }}. Your research data stays local to this device.</p>
          </div>
        </section>
      </div>
    </section>
  </div>
</template>

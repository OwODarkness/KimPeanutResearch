<script setup lang="ts">
import type { PaperListItem } from "../types";
import PaperListItemRow from "./PaperListItemRow.vue";

defineProps<{ papers: PaperListItem[] }>();
</script>

<template>
  <div class="paper-list">
    <div class="list-heading"><span>32 PAPERS</span><span>ADDED</span></div>
    <PaperListItemRow v-for="paper in papers" :key="paper.title" :paper="paper" />
  </div>
</template>

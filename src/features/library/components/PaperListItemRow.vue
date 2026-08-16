<script setup lang="ts">
import type { PaperListItem } from "../types";

defineProps<{ paper: PaperListItem }>();
</script>

<template>
  <article class="paper-row" :class="{ selected: paper.isSelected }">
    <div class="paper-cover" :style="{ '--cover-color': paper.coverColor }"><span>PDF</span><i></i><i></i><i></i></div>
    <div class="paper-copy"><h2>{{ paper.title }}</h2><p class="authors">{{ paper.authors }}</p><div class="paper-meta"><span>{{ paper.venue }}</span><span class="meta-dot">·</span><span>Added 2 days ago</span><span class="meta-dot">·</span><span class="note-icon">▱</span><span>3 notes</span></div><div class="tag-row"><span v-for="tag in paper.tags" :key="tag" class="tag">{{ tag }}</span></div></div>
    <button class="more-button" aria-label="More options">•••</button>
  </article>
</template>

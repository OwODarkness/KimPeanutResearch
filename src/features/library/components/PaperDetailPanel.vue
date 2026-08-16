<script setup lang="ts">
import type { PaperDetail } from "../types";

defineProps<{ paper: PaperDetail }>();
</script>

<template>
  <aside class="detail-pane">
    <header class="detail-top"><span class="detail-label">PAPER DETAILS</span><button class="close-button" aria-label="Close details">×</button></header>
    <div class="detail-scroll">
      <div class="hero-cover"><div class="hero-paper">ATTENTION<br />IS ALL YOU<br />NEED<span></span><small>Vaswani et al.</small></div></div>
      <div class="detail-title-row"><h2>{{ paper.title }}</h2><button class="save-button" aria-label="Saved">⌑</button></div>
      <p class="detail-authors">{{ paper.authors.replace(' · et al.', '') }}</p><p class="detail-venue">{{ paper.venue }} <span>•</span> {{ paper.citationCount }}</p>
      <div class="detail-actions"><button class="read-button">Open PDF <span>↗</span></button><button class="outline-button">•••</button></div>
      <section class="detail-section abstract"><h3>Abstract</h3><p>{{ paper.abstract }}</p><a href="#">Read abstract <span>→</span></a></section>
      <section class="detail-section"><h3>Research notes <span class="section-badge">3</span></h3><div class="note-card"><span class="note-pin">✦</span><p>{{ paper.note }}</p><small>Today · Introduction</small></div><button class="add-note">+ Add a note</button></section>
      <section class="detail-section"><h3>Tags</h3><div class="tag-row"><span v-for="tag in paper.tags" :key="tag" class="tag">{{ tag }}</span><button class="tag-add">+</button></div></section>
      <section class="detail-section provenance"><h3>Source</h3><p><span class="source-dot"></span> Imported PDF <span>·</span> {{ paper.pageCount }} pages</p></section>
    </div>
  </aside>
</template>

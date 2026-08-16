<script setup lang="ts">
defineEmits<{ "open-settings": [] }>();
</script>

<template>
  <aside class="sidebar">
    <div class="brand"><span class="brand-mark">K</span><span>KimPeanut</span></div>
    <nav class="primary-nav" aria-label="Main navigation">
      <a class="nav-item active" href="#"><span class="nav-icon">⌘</span>Library <span class="nav-count">248</span></a>
      <a class="nav-item" href="#"><span class="nav-icon">◫</span>Collections</a>
      <a class="nav-item" href="#"><span class="nav-icon">⌁</span>Reading queue <span class="nav-count">12</span></a>
      <a class="nav-item" href="#"><span class="nav-icon">✦</span>Insights</a>
    </nav>
    <div class="sidebar-section">
      <div class="section-label"><span>COLLECTIONS</span><button aria-label="Add collection">+</button></div>
      <a class="collection active-collection" href="#"><span class="collection-dot indigo"></span>Foundation models <span>32</span></a>
      <a class="collection" href="#"><span class="collection-dot orange"></span>Retrieval systems <span>18</span></a>
      <a class="collection" href="#"><span class="collection-dot green"></span>Methods to revisit <span>9</span></a>
      <a class="collection" href="#"><span class="collection-dot blue"></span>Thesis references <span>41</span></a>
    </div>
    <div class="sidebar-footer"><div class="sync-state"><span class="sync-dot"></span>Library stored locally</div><button class="sidebar-settings" aria-label="Open settings" @click="$emit('open-settings')"><span>⚙</span> Settings</button><div class="sidebar-user"><div class="avatar">YL</div><span>Yilin</span><span class="chevron">⌄</span></div></div>
  </aside>
</template>

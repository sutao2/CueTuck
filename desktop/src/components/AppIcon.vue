<template>
  <svg class="app-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
    <path :d="paths[name] || paths.library" />
  </svg>
</template>

<script setup>
defineProps({ name: { type: String, default: "library" } });
const paths = {
  search: "M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0",
  square: "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z",
  library: "M4 4h5v16H4zM9 4h5v16H9zM16 5l4-1 3 15-4 1z",
  folder: "M3 7V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7z",
  settings: "M4 7h16M4 17h16M8 4v6M16 14v6",
  plus: "M12 5v14M5 12h14",
  refresh: "M20 7v5h-5M20 12a8 8 0 1 0-2 5",
  grid: "M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z",
  list: "M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01",
  moon: "M20 14A8 8 0 0 1 10 4a8 8 0 1 0 10 10z",
  sun: "M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1 1M18 18l1 1M5 19l1-1M18 6l1-1M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0",
  blue: "M8 6l-6 6 6 6M16 6l6 6-6 6M14 3l-4 18",
  violet: "M4 3h16a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1zM3 17l6-6 4 4 3-3 5 5M8 7h.01",
  coral: "M3 5h12v14H3zM15 10l6-4v12l-6-4",
  amber: "M8 6V3h8v3M3 6h18v14H3zM3 11h18M10 11v3h4v-3",
  green: "M4 20l1-5L17 3l4 4L9 19zM14 6l4 4M4 20h16",
  teal: "M4 4h16v16H4zM4 10h16M10 10v10",
  rose: "M3 10v4h5l10 5V5L8 10zM8 14l2 7h3M21 9v6",
  cyan: "M4 3v18h17M8 16v-5M13 16V7M18 16V4",
  gold: "M2 9l10-6 10 6-10 6zM6 12v6l6 3 6-3v-6M22 9v8",
  lime: "M3 10l9-7 9 7M5 9v12h14V9M9 21v-8h6v8",
};
</script>

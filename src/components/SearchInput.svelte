<script lang="ts">
  import { Search } from "@lucide/svelte";

  let { value = $bindable(""), placeholder, width = "240px" }: { value?: string; placeholder: string; width?: string } = $props();
</script>

<div class="search" style:width>
  <Search size={14} />
  <input class="input" aria-label={placeholder} {placeholder} bind:value />
</div>

<style>
  .search {
    position: relative;
    display: flex;
    align-items: center;
    min-width: 0;
    max-width: 100%;
  }
  .search :global(svg) {
    position: absolute;
    left: 8px;
    color: var(--text-3);
    pointer-events: none;
  }
  .input { padding-left: 28px; }
</style>

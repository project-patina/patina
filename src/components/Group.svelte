<script lang="ts">
  import type { Snippet } from "svelte";

  let { title = "", children }: { title?: string; children: Snippet } = $props();
</script>

<section class="group">
  {#if title}<h3>{title}</h3>{/if}
  {@render children()}
</section>

<style>
  .group { margin-top: 24px; }
  .group:first-child { margin-top: 0; }
  h3 {
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
    color: var(--text-2);
    font-size: var(--fs-13);
    font-weight: 600;
  }
</style>

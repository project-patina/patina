<script lang="ts">
  import type { Snippet } from "svelte";

  // One setting: title + description on the left, control on the right.
  let { title, desc = "", children, stacked = false }: { title: string; desc?: string; children: Snippet; stacked?: boolean } = $props();
</script>

<div class="row" class:stacked>
  <div class="text">
    <div class="title">{title}</div>
    {#if desc}<div class="desc">{desc}</div>{/if}
  </div>
  <div class="control">{@render children()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    min-height: 48px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child { border-bottom: 0; }
  .row.stacked {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 12px 0;
  }
  .text { flex: 1 1 0; min-width: 0; }
  .stacked .text { flex: none; }
  .title { font-size: var(--fs-13); font-weight: 500; }
  .desc {
    color: var(--text-3);
    font-size: var(--fs-12);
    margin-top: 2px;
    line-height: 1.4;
  }
  .control {
    display: flex;
    align-items: center;
    gap: var(--control-gap);
    flex-shrink: 0;
    max-width: 100%;
    min-width: 0;
  }
  .control :global(input), .control :global(select) { max-width: 100%; }
  .stacked .control { flex-shrink: 1; flex-wrap: wrap; }
</style>

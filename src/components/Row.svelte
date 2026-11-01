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
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child {
    border-bottom: 0;
  }
  .row.stacked {
    flex-direction: column;
    align-items: stretch;
    gap: 10px;
  }
  .title {
    font-weight: 560;
  }
  .desc {
    color: var(--text-3);
    font-size: 0.86rem;
    margin-top: 3px;
    line-height: 1.45;
    max-width: 520px;
  }
  .control {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }
  .stacked .control {
    flex-shrink: 1;
    flex-wrap: wrap;
  }
</style>

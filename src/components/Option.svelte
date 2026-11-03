<script lang="ts">
  import Toggle from "./Toggle.svelte";

  let { checked = $bindable(false), label, title = "" }: { checked?: boolean; label: string; title?: string } = $props();
</script>

<label class="option" title={title || undefined}>
  <Toggle bind:checked {label} />
  <span>{label}</span>
</label>

<style>
  .option {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
    font-size: var(--fs-13);
    white-space: nowrap;
    cursor: pointer;
  }
</style>

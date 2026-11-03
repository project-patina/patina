<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  let { checked = $bindable(false), disabled = false, label = "", onchange }: { checked?: boolean; disabled?: boolean; label?: string; onchange?: (v: boolean) => void } = $props();
</script>

<button
  class="toggle"
  aria-label={label || t("common.toggle")}
  class:on={checked}
  role="switch"
  aria-checked={checked}
  {disabled}
  onclick={() => {
    checked = !checked;
    onchange?.(checked);
  }}
>
  <span></span>
</button>

<style>
  .toggle {
    width: 32px;
    height: 18px;
    border-radius: 18px;
    background: var(--panel-3);
    border: 1px solid var(--border-2);
    position: relative;
    flex-shrink: 0;
    transition: background 0.12s, border-color 0.12s;
  }
  .toggle span {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--text-2);
    transition: transform 0.14s var(--ease), background 0.12s;
  }
  .toggle.on {
    background: var(--accent);
    border-color: transparent;
  }
  .toggle.on span {
    transform: translateX(14px);
    background: var(--accent-fg);
  }
  .toggle:disabled { opacity: 0.45; }
</style>

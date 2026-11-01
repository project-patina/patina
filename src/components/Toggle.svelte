<script lang="ts">
  let { checked = $bindable(false), disabled = false, label = "Przełącz", onchange }: { checked?: boolean; disabled?: boolean; label?: string; onchange?: (v: boolean) => void } = $props();
</script>

<button
  class="toggle"
  aria-label={label}
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
    width: 40px;
    height: 23px;
    border-radius: 999px;
    background: var(--panel-3);
    border: 1px solid var(--border-2);
    position: relative;
    flex-shrink: 0;
    transition: background 0.25s var(--ease), border-color 0.25s var(--ease);
  }
  .toggle span {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: var(--text-2);
    transition: transform 0.3s var(--ease-spring), background 0.25s;
  }
  .toggle.on {
    background: var(--accent);
    border-color: transparent;
  }
  .toggle.on span {
    transform: translateX(17px);
    background: white;
  }
  .toggle:disabled {
    opacity: 0.45;
  }
</style>

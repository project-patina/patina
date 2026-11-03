<script lang="ts">
  import { parseWhole } from "../lib/numbers";

  let { value, min, max, label, width = 88, onchange }: {
    value: number;
    min: number;
    max: number;
    label: string;
    width?: number;
    onchange: (value: number) => void;
  } = $props();

  let input: HTMLInputElement;
  let invalid = $state(false);

  function check() {
    const typed = Number(input.value);
    invalid = input.value.trim() === "" || !Number.isInteger(typed) || typed < min || typed > max;
  }

  function commit() {
    const next = parseWhole(input.value, min, max, value);
    input.value = String(next);
    invalid = false;
    if (next !== value) onchange(next);
  }
</script>

<input bind:this={input} class="input" type="number" {min} {max} step="1" aria-label={label} aria-invalid={invalid} style:width="{width}px" {value} oninput={check} onchange={commit} />

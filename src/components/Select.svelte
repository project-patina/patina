<script lang="ts" generics="T extends string">
  let { value, options, label, width = 180, onchange }: {
    value: T;
    options: readonly { value: T; label: string }[];
    label: string;
    width?: number;
    onchange: (value: T) => void;
  } = $props();
</script>

<select class="input" aria-label={label} style:width="{width}px" {value} onchange={(e) => onchange(e.currentTarget.value as T)}>
  {#each options as option (option.value)}<option value={option.value}>{option.label}</option>{/each}
</select>

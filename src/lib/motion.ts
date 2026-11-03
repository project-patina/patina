import { tick } from "svelte";
import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";
import { dur } from "./state.svelte";

const STAGGER_MS = 30;
const STAGGER_ITEMS = 6;

function rise(duration: number, delay = 0): TransitionConfig {
  return {
    duration: dur(duration), delay: dur(delay), easing: cubicOut,
    css: (t, u) => `opacity:${t};transform:translateY(${6 * u}px);`,
  };
}

export const page = (_node: Element) => rise(180);

/** Dialogs and popovers; pass a shorter duration for the exit. */
export const dialog = (_node: Element, { duration = 160 }: { duration?: number } = {}) => rise(duration);

/** List items: only the first few are staggered. */
export const reveal = (_node: Element, { index = 0 }: { index?: number } = {}) => rise(180, Math.min(index, STAGGER_ITEMS - 1) * STAGGER_MS);

/**
 * The sliding selection shared by every nav and segmented control: the container holds one
 * `[data-indicator]` child, which follows the element marked `aria-current`. Pass the selected key.
 */
export function indicator(node: HTMLElement, _key: unknown) {
  const mark = node.querySelector<HTMLElement>(":scope > [data-indicator]")!;
  let placed = false;

  function place() {
    const item = node.querySelector<HTMLElement>("[aria-current]");
    if (!item) {
      mark.style.opacity = "0";
      placed = false;
      return;
    }
    const box = node.getBoundingClientRect();
    const rect = item.getBoundingClientRect();
    const transform = `translate(${rect.left - box.left + node.scrollLeft}px, ${rect.top - box.top + node.scrollTop}px)`;
    mark.style.width = `${rect.width}px`;
    mark.style.height = `${rect.height}px`;
    if (placed) mark.style.transform = transform;
    else {
      mark.style.transition = "none";
      mark.style.transform = transform;
      void mark.offsetWidth;
      mark.style.transition = "";
    }
    mark.style.opacity = "1";
    placed = true;
  }

  const observer = new ResizeObserver(place);
  observer.observe(node);
  place();
  return {
    update() { void tick().then(place); },
    destroy() { observer.disconnect(); },
  };
}

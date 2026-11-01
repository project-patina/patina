/** Debounced, sequential persistence. Failed changes remain available for retry. */
export function createSettingsWriter<T>(options: {
  save: (value: T) => Promise<T>;
  onSaved: (saved: T, submitted: T) => void;
  onError: (error: unknown) => void;
  delay?: number;
}) {
  let pending: { value: T } | undefined;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let running: Promise<void> | undefined;

  function schedule(value: T) {
    pending = { value: structuredClone(value) };
    clearTimeout(timer);
    timer = setTimeout(() => { void flush().catch(() => {}); }, options.delay ?? 350);
  }

  function flush(): Promise<void> {
    clearTimeout(timer);
    if (running) return running;
    if (!pending) return Promise.resolve();
    running = (async () => {
      while (pending) {
        const submitted = pending;
        pending = undefined;
        try {
          const saved = await options.save(submitted.value);
          options.onSaved(saved, submitted.value);
        } catch (error) {
          pending ??= submitted;
          options.onError(error);
          throw error;
        }
      }
    })().finally(() => { running = undefined; });
    return running;
  }

  return { schedule, flush, hasPending: () => !!pending };
}

/** Tauri awaits this handler and destroys the window unless closing is prevented. */
export function createCloseHandler(save: () => Promise<void>) {
  return async (event: { preventDefault: () => void }) => {
    try {
      await save();
    } catch {
      // The settings writer reports the error; keep the window open for retry.
      event.preventDefault();
    }
  };
}

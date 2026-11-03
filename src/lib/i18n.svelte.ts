// Usage: import { t } from "../lib/i18n.svelte", then t("nav.home") or t("news.count", { n: 3 }).
// t() is reactive in templates and $derived. Add each key to locales/pl.ts first; en.ts then fails to
// compile until it has the same key. A value with "|" is a plural, see locales/resolve.ts.
import { app } from "./state.svelte";
import { en } from "./locales/en";
import { pl, type Key } from "./locales/pl";
import { format, resolveLanguage, type Lang, type Params } from "./locales/resolve";

const dictionaries: Record<Lang, Record<Key, string>> = { pl, en };

export function lang(): Lang {
  return resolveLanguage(app.settings?.language, typeof navigator === "undefined" ? "" : navigator.language);
}

/** BCP 47 tag for Intl formatters. */
export function locale(): string {
  return lang() === "pl" ? "pl-PL" : "en-GB";
}

export function t(key: Key, params?: Params): string {
  const active = lang();
  return format(dictionaries[active][key], active, params);
}

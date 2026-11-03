export type Lang = "pl" | "en";
export type Params = Record<string, string | number>;

/** A saved "pl"/"en" wins; anything else follows the system language. */
export function resolveLanguage(setting: string | null | undefined, system: string | null | undefined): Lang {
  if (setting === "pl" || setting === "en") return setting;
  return system?.toLowerCase().startsWith("pl") ? "pl" : "en";
}

const FORMS: Record<Lang, Record<string, number>> = {
  pl: { one: 0, few: 1, many: 2, other: 2 },
  en: { one: 0, other: 1 },
};

/** "a|b|c" holds plural forms: pl is one|few|many, en is one|other. `{n}` selects the form. */
export function format(template: string, lang: Lang, params?: Params): string {
  let text = template;
  if (text.includes("|") && typeof params?.n === "number") {
    const forms = text.split("|");
    text = forms[FORMS[lang][new Intl.PluralRules(lang).select(params.n)] ?? forms.length - 1] ?? forms[forms.length - 1];
  }
  return params ? text.replace(/\{(\w+)\}/g, (all, name) => (name in params ? String(params[name]) : all)) : text;
}

import DOMPurify from "dompurify";
import { marked } from "marked";

/** Modrinth descriptions are Markdown with some HTML; sanitise before rendering. */
export function renderMarkdown(md: string): string {
  const html = marked.parse(md ?? "", { async: false, gfm: true, breaks: false }) as string;
  return DOMPurify.sanitize(html, { FORBID_TAGS: ["style", "iframe", "form", "input"], FORBID_ATTR: ["style"] });
}

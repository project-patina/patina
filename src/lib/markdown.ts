import DOMPurify from "dompurify";
import { marked } from "marked";

/** Modrinth descriptions are Markdown with some HTML; sanitise before rendering. */
export function renderMarkdown(md: string): string {
  const html = marked.parse(md ?? "", { async: false, gfm: true, breaks: false }) as string;
  return DOMPurify.sanitize(html, { FORBID_TAGS: ["style", "iframe", "form", "input"], FORBID_ATTR: ["style"] });
}

/** Notices are text only: embedded HTML cannot load trackers or active content. */
export function renderAnnouncementMarkdown(md: string): string {
  const html = marked.parse(md, { async: false, gfm: true }) as string;
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["h1", "h2", "h3", "h4", "p", "a", "strong", "em", "del", "ul", "ol", "li", "blockquote", "code", "pre", "hr", "br"],
    ALLOWED_ATTR: ["href", "title"], ALLOW_DATA_ATTR: false,
  });
}

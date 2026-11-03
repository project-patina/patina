# Home announcements

The separate NEWS page has been replaced by small image cards on Home. Each card
opens a sanitized Markdown message. A dot marks unread messages; read state stays
on the device. These are in-app messages, not desktop push notifications.

## Source

`resources/announcements-source.json` names the public content repository,
`project-patina/patina-news`, branch `main`, directory `updates`. It is compiled in:
changing the source needs a rebuild, but later edits to that repository's Markdown,
images and index do not. Setting `repository` to `null` makes **no feed network
requests** and shows the bundled message only. The branch must be a single name,
without slashes; directory and filenames use ASCII letters, digits, dots,
underscores, hyphens and `/` separators. Private repos and tokens are not supported
or needed.

## Content repository format

```text
updates/
  index.json
  first-version.pl.md
  first-version.en.md
  images/
    first-version.png
```

`index.json` is an array (up to 30 entries). The three newest dates appear on Home.
Equal dates keep the order in the file. An empty array intentionally hides the cards.
Every entry carries both languages; the launcher downloads only the Markdown of its own.

```json
[
  {
    "id": "first-version",
    "date": "2026-11-01",
    "image": "images/first-version.png",
    "pl": {
      "title": "Patina: pierwsza wersja",
      "summary": "Co potrafi launcher.",
      "markdown": "first-version.pl.md"
    },
    "en": {
      "title": "Patina: first version",
      "summary": "What the launcher does.",
      "markdown": "first-version.en.md"
    }
  }
]
```

IDs are unique and stable. Dates use `YYYY-MM-DD`; update the date when an edited
message should become unread again. Titles are at most 140 UTF-8 bytes and
summaries at most 300 bytes. Images use PNG, JPEG, WebP or SVG, ideally under
500 KB, with a roughly 4:3 composition. Broken or offline images use bundled art.
Keep important text in Markdown rather than inside an image.

Markdown supports headings, lists, emphasis, blockquotes, code and HTTPS links.
Scripts, forms, inline styling, embedded images and iframes are stripped. HTTPS
links open in the system browser; relative links do not navigate the launcher.
`resources/announcements/` is a working example of this format; its image is
`public/announcements/first-version.png`.

## Offline and failure behavior

The Rust backend fetches public raw GitHub files without account credentials.
Only paths within the configured repository are accepted; redirects and external
file URLs are rejected. Limits: 32 KiB index, 96 KiB Markdown per card, three cards,
eight seconds for a complete refresh. Fetching never blocks launcher startup.

A successful feed is cached per language, under `cache/announcements-pl.json` and
`cache/announcements-en.json` in the launcher's data directory, for 15 minutes. On a later refresh failure, the last successful
feed remains readable. Without a cache, the bundled message appears instead. The
frontend keeps the loaded feed for the current session. No notification prompt,
GitHub login, telemetry or new settings control is added to this flow.

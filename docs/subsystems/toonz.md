---
title: Toonz template (en.toonz, en.toonx, en.lewdmanhwa)
type: subsystem
created: 2026-11-19
updated: 2026-11-21
sources:
  - code: templates/toonz/src/lib.rs
  - code: templates/toonz/src/imp.rs
  - code: templates/toonz/src/helpers.rs
  - code: templates/toonz/src/home.rs
  - code: templates/toonz/src/models.rs
  - code: templates/toonz/src/transport.rs
  - code: scripts/tests/test_toonz_transport.mjs
  - code: templates/toonz/src/settings.rs
  - code: templates/toonz/src/test.rs
  - code: sources/en.toonz/src/lib.rs
  - code: sources/en.toonz/src/test.rs
  - code: sources/en.toonz/res/filters.json
  - code: sources/en.toonz/res/settings.json
  - code: sources/en.toonx/src/lib.rs
  - code: sources/en.toonx/src/test.rs
  - code: sources/en.toonx/res/settings.json
  - code: sources/en.lewdmanhwa/src/lib.rs
  - code: sources/en.lewdmanhwa/src/test.rs
  - code: sources/en.lewdmanhwa/res/settings.json
  - ext: https://toonz.to/comics
  - ext: https://github.com/keiyoushi/extensions-source/tree/main/src/en/toonz
verified_at: afbc5da
---

Toonz (`toonz.to`), Toonx (`toonx.to`) and LewdManhwa (`lewdmanhwa.com`) are one Next.js
engine, which keiyoushi serves from a single extension with three sources. The `toonz` template
holds all of it; each consumer is a `Params` block naming its catalogue (`comics` or
`webtoons`), the path that resolves every series (`comic` or `webtoon`), the paths series are
filed under, its rating default, type count and home rows (`templates/toonz/src/lib.rs`).
Everything below was probed live on 2026-11-19 unless marked.

## Requests

- Search and listings: `/{catalog}?search=&sort=&status=&genres=&excludeGenres=&author=&page=`
  (`helpers.rs` `SearchParams::page`), the same on all three; `q` is ignored. Sorts are the
  site's Latest, Popular, Top Rated, A-Z (`latest`, `popular`, `rating`, `a-z`). `author`
  takes the site's slug; names are slugified the site's way (punctuation dropped, spaces to
  hyphens).
- Cards: each `img[data-cover]`, walked up to the nearest ancestor that is or holds a series
  link, as keiyoushi's `closest` does (`helpers.rs` `card`). Toonz and Toonx wrap the cover in
  the link; LewdManhwa puts an empty overlay link beside it.
- Details: the series path (`/comic/{slug}` redirects to `/{type}/{slug}` on Toonz and to
  `/adult/{slug}` on Toonx). Toonz and Toonx list Type, Status and Author in a `dl`;
  LewdManhwa shows badges and `/author/` links instead (`helpers.rs` `details`).
- Chapters: `/api/comics/{id}/chapters`, so a chapters-only update still fetches the series
  page for the id. Chapter keys are the slug minus `chapter-` (`2.1`, `45-wad21`).
- Pages: the chapter route with `RSC: 1`, reading the `"images"` array (`{filename}` under
  `/uploads/`), as keiyoushi's client does.
- Home: Toonz reads `/manhwa`, `/manga` and the list view, Toonx `/` and the list view, and
  LewdManhwa `/home` (hero slides and Latest Updates; its Trending Now repeats the slides), with Toonz publishing rows as each page loads; the other two use one `send_all`.

> ⚠️ LewdManhwa's hero renders only its first slide's cover (a `<picture>`); the other slides
> arrive with an empty cover box, so a slide takes its cover from the same title's card on the
> page (`templates/toonz/src/home.rs`). Requiring the picture showed one Featured title.

> ⚠️ The content setting is the `reader_prefs` cookie (`{"rating","types","blockedGenres"}`),
> and every page reads it, home included. The site's own URL `rating` and `types` can only
> narrow the cookie, never widen it, and an included genre the cookie blocks returns nothing. So
> the source never sends those URL parameters: each request writes the cookie itself from the
> effective values, with search filters replacing the settings, which lets a filter widen as
> well as narrow (`helpers.rs` `Content::request`).

> ⚠️ `rating` is a level ("Erotica and below"), not a set: Safe, Suggestive, Erotica, Explicit
> (`pornographic`). The setting and the Content Rating filter are both selects, as the site's
> own control is single-choice; the filter's first option, Any, is omitted by the app, so the
> setting applies until a level is picked (`imp.rs`). Toonz defaults to Suggestive, as a source
> rated 1 must; LewdManhwa defaults to Explicit, the whole catalogue, and is rated 2.

> ⚠️ Toonx's catalogue is all Explicit comics: any lower rating or another type in the cookie
> returns nothing, so it has no rating or type controls and always sends `pornographic`
> (`sources/en.toonx/src/lib.rs`). Its excluded genres still apply.

> ⚠️ The list view prints a chapter number but no chapter link, so Recently Updated keys its
> chapter by that number. A slug with a suffix (`chapter-45-wad21`) answers 404 for the bare
> number (probed on LewdManhwa, 2026-11-20), so `get_page_list` then reads the series' chapter
> list once and retries with the real key (`imp.rs`); the LewdManhwa suite reads chapter 45 by
> its number. The reader takes the first `"images"` array that parses as pages, not merely the
> first one named so (`helpers.rs` `reader_images`).

> ⚠️ A cleared Types setting sends `types:["none"]`, which the site answers with nothing; a
> full selection sends `[]` (`helpers.rs` `narrowed`).

> ⚠️ Next.js streams the page into hidden `S:` chunks after `<main>`, so selectors scoped to
> `main` find nothing. Listing and details selectors take the whole document; home sections
> are self-contained and need no chunk swap.

> ⚠️ Only the list view (`view=list`) prints update times; some of its cards print none, so
> Recently Updated keeps those rows undated. LewdManhwa has no list view, so its Latest
> Updates row names the chapter without a date. Times are the site's `timeAgo` units: just
> now, m, h, d, w, mo, y.

> ⚠️ Home rows read each page's own section: Featured (Top rated this month) merged by rating,
> New Titles merged by age, and the visible Today panel of Trending and Popular Updates. An
> absent section drops its row rather than borrowing another page's cards. LewdManhwa's Most
> Read This Week, Top Rated, Uncensored and Yaoi carousels carry covers without series links
> in the server HTML, so they cannot be rows.

> ⚠️ Toonz series and chapter pages answer `403` with `cf-mitigated: challenge`, a Cloudflare
> managed challenge observed in the earlier sandbox; automatic recovery in the app was not
> established. `helpers.rs` `check` turns an unsolved challenge into "The site answered HTTP 403"
> rather than a misleading "Series not found". Toonx and LewdManhwa pages answer the sandbox.

> ⚠️ The app appends a source's `Cookie` header after its stored cookies, so the Cloudflare
> clearance survives; a `reader_prefs` stored by a web view visit would come first and win
> (inferred from `AidokuRunner.swift` `modify`).

Genres are static per consumer, generated from each site's `/api/genres` (Toonz 199, Toonx
607 including X-Ray, LewdManhwa 78). Merged `en.toonkotsu` shows they could be fetched instead
(`DynamicFilters` and `DynamicSettings`), which would drop about 3,400 JSON lines (unverified
whether Toonz's `/api/genres` answers through Cloudflare). The live
`details_chapters_and_pages` test fails on Toonz in the sandbox on the challenge page and needs
an in-app check; Toonx and LewdManhwa run their whole suites.

## Toonz request timeouts

Device logs for Toonz v4 showed `NSURLErrorDomain -1001` on `/manga`, `/manhwa`, and
`/comics?sort=latest&page=1`, while both Safari and the app's browser loaded the site.
These logs establish plain-request timeouts, not a refused HTTP response.

`Params::browser_requests` opts only `en.toonz` into same-origin asynchronous browser
requests (`templates/toonz/src/transport.rs`). A local blank document supplies
the origin without waiting for a remote navigation. Readiness polling is bounded; XHR has
a 15-second timeout and Rust polls for at most 20 iterations. Status and body remain separate,
including the reader's 404 fallback. Home and search write the effective `reader_prefs`
cookie before sending, retaining the existing setting/filter override behavior. Toonz home
publishes partial components after each page; failed pages do not discard successful rows.

Native WebKit probes returned HTTP 200 for both type home pages, browse, and text search
on 2026-11-21. The first browser-request build still received 403 for series pages; real
navigation was needed to establish the title session.

> ⚠️ Toonz series pages must navigate in the browser and wait for the parsed document's
> `data-comic-id` marker. `templates/toonz/src/transport.rs` returns the redirected URL with
> the HTML. Navigation is limited to 25 polling iterations. An API or reader-data 403 gets
> one normal navigation to its series/chapter before one retry, never recursive retries.

A fresh, non-persistent native WebKit session loaded Solo Leveling's title, its chapter API
(263 chapters), and chapter 2's reader data (26 image filenames). The user then installed v6
and confirmed it works in Aidoku, following their earlier confirmation of faster home loading.
This is device confirmation of the reported fix, not a comprehensive multi-title benchmark.

`node --test scripts/tests/test_toonz_transport.mjs` covers asynchronous request setup,
preferences, RSC/404, network errors, timeout, cancellation, refused/missing-marker pages,
document readiness, redirect URL preservation, and chapter-image readiness. The existing six
template parser tests and three Toonz `review_` tests pass. The full Toonz live suite still
fails with `JsError(MissingResult)` because the Rust runner stubs WebView; it cannot replace
native WebKit and device evidence for this path.

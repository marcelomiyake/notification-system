# Lighthouse and SEO META Verification

This record audits the built notification web console and checks its browser metadata against the published SEO META in 1 CLICK field list. Lighthouse measures this browser run; it is separate from SonarQube analysis and JEV readiness evaluation.

## Contents

- [Scope](#scope)
- [Lighthouse results](#lighthouse-results)
- [SEO META checklist](#seo-meta-checklist)
- [Findings](#findings)
- [Related documentation](#related-documentation)

## Scope

- **Date/time:** 2026-09-25 12:31:32 America/Sao_Paulo.
- **Source:** dirty worktree based on revision 41d9dac.
- **Environment:** Linux; Google Chrome 154.0.8037.57; Lighthouse CLI 13.5.0; default mobile emulation and simulated throttling.
- **Target:** production Vite preview at http://127.0.0.1:4173/; one app preview ran at a time and was stopped before the next.
- **Commands:** `pnpm run build`; `npx --yes lighthouse http://127.0.0.1:4173/ --output=json --output-path=/tmp/lighthouse-notification.json --only-categories=performance,accessibility,best-practices,seo --chrome-flags='--headless --no-sandbox' --quiet`.
- Notification API services were not deployed during this frontend-only audit; recipients and templates requests returned HTTP 500 through the local preview proxy. This is not a live-system smoke.

## Lighthouse results

| Performance | Accessibility | Best practices | SEO |
| ---: | ---: | ---: | ---: |
| 98 | 92 | 96 | 63 |

Scores range from 0 to 100. The SEO score reflects the intentional noindex directive; the two other lower categories are described under Findings.

## SEO META checklist

| Field | Result |
| --- | --- |
| HTML language | English is declared. |
| Title and length | Present: “Notification Console” (20 characters). |
| Description and length | Present: “Local notification system operations console” (44 characters). |
| Robots metadata | noindex, nofollow is intentional for this local demo. A valid robots.txt is served with Allow: / so crawlers can read the page directive. |
| Canonical URL | Omitted because the local preview/deployment origin is not a stable public URL. |
| Headings | One H1 with a semantic H2/H3 sequence. |
| Images and alt text | No HTML image elements; this check is not applicable. |
| Links | Four anchors in the default view: two internal and two external, three unique targets; none is unlabeled. |
| Favicon | SVG favicon is declared and served. |
| Open Graph, Twitter, and sitemap | Not provided because the app has no public share URL or public indexing target. |

The SEO META in 1 CLICK extension was not installed in the browser. This is a manual checklist audit against its published fields, not a claim that the extension itself ran.

## Findings

- Lighthouse confirmed the title, description, valid robots.txt, and page metadata. It reports the page as not crawlable by design because of the noindex directive.
- Accessibility scored 92; Lighthouse found low-contrast text and a footer link that is not sufficiently distinguished from surrounding text.
- Best practices scored 96 because the preview proxy could not reach the API services. Test the complete UI against a deployed API before treating this as an application-flow result.

## Related documentation

- [Verification index](README.md)
- [Notification frontend guide](../../notification-frontend/README.md)
- [SonarQube verification](sonarqube.md)
- [JEV readiness assessment](jev-readiness.md)
- [Chrome Lighthouse overview](https://developer.chrome.com/docs/lighthouse/overview)
- [SEO META in 1 CLICK listing](https://chromewebstore.google.com/detail/seo-meta-in-1-click/bjogjfinolnhfhkbipphpdlldadpnmhc?hl=en-GB)

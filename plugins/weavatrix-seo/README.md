# Weavatrix SEO plugin

<img src="assets/logo.svg" alt="Weavatrix SEO logo" width="72" align="right">

Weavatrix SEO gives Cursor, Codex, Claude, and Grok Build **15 read-only
search-intelligence tools** through one native MCP server. It covers crawl,
GSC, logs, AI citations, bounded query, retrieve, and a plan DAG. It never
writes source.

The plugin starts the published `weavatrix-seo@0.6.2` npm package. That package
already contains the matching prebuilt native binaries; it has no lifecycle
scripts or runtime dependencies. Node.js 18 or newer is required.

The bundled skill is optional. It activates for search-surface questions, not
for ordinary source edits.

## Install

### Codex

```text
codex plugin marketplace add Weavatrix/weavatrix-seo --sparse .agents/plugins plugins/weavatrix-seo
codex plugin add weavatrix-seo@weavatrix-seo
```

### Claude Code

```text
claude plugin marketplace add Weavatrix/weavatrix-seo --sparse .claude-plugin plugins
claude plugin install weavatrix-seo@weavatrix-seo
```

### Grok Build

```text
grok plugin marketplace add Weavatrix/weavatrix-seo
```

Open `/marketplace` and install Weavatrix SEO.

### Cursor

Search for **Weavatrix SEO** after marketplace approval, or:

```bash
claude mcp add weavatrix-seo -- npx -y weavatrix-seo mcp --allow-root .
```

## Safety

Weavatrix SEO is local and read-only. It fetches public HTTP for a bounded
crawl and reads files inside `--allow-root`. It does not edit source. `seo_plan`
can hand a path to Weavatrix Refactor; SEO still does not apply the patch.

## Source and support

- Website: https://weavatrix.com/seo
- Repository: https://github.com/Weavatrix/weavatrix-seo
- Issues: https://github.com/Weavatrix/weavatrix-seo/issues

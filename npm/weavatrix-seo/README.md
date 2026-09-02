# weavatrix-seo

Native [Weavatrix SEO](https://github.com/Weavatrix/weavatrix-seo) CLI and MCP host.

This package is a zero-dependency Node launcher. It contains prebuilt `weavatrix-seo` binaries for win32, darwin, and linux on x64 and arm64. Installation does not compile, download, or run lifecycle scripts.

```bash
npm install -g weavatrix-seo
weavatrix-seo --version
weavatrix-seo audit --site https://example.com --json
weavatrix-seo-mcp --allow-root PATH
```

`weavatrix-seo-mcp` is the same native binary with `mcp` as the first argument.

Rust install:

```bash
cargo install weavatrix-seo-cli --locked
```

The engine stays read-only. Missing evidence is `unmeasured`, never green.

MIT. See [LICENSE](LICENSE).

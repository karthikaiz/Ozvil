# Ozvil Update Server

The Tauri updater calls this endpoint to check for new versions:

```
GET https://update.ozvil.app/{target}/{arch}/{current_version}
```

Where:
- `{target}` — `windows`
- `{arch}` — `x86_64`
- `{current_version}` — the installed semver, e.g. `0.1.0`

## Response: no update available

Return **HTTP 204 No Content**.

## Response: update available

Return **HTTP 200** with `Content-Type: application/json` and the body from `update.json.example`.

## Getting the `.sig` file

After running `pnpm tauri build`, Tauri produces `.sig` files alongside each installer:

```
src-tauri/target/release/bundle/nsis/Ozvil_0.2.0_x64-setup.exe
src-tauri/target/release/bundle/nsis/Ozvil_0.2.0_x64-setup.exe.sig   ← paste contents here
```

The `.sig` file contents (base64) go into `platforms.windows-x86_64.signature`.

## Minimal update server (Node.js example)

```js
import express from "express";
import fs from "fs";

const app = express();

// Current latest release info
const LATEST = JSON.parse(fs.readFileSync("latest.json", "utf8"));

app.get("/:target/:arch/:version", (req, res) => {
  const { version } = req.params;
  const { gt } = await import("semver");

  if (!gt(LATEST.version, version)) {
    return res.status(204).end(); // no update
  }

  res.json(LATEST);
});

app.listen(3000);
```

## CI: automatic update manifest generation

The GitHub Actions release workflow (`.github/workflows/release.yml`) uploads
the `.sig` files as release artifacts. After publishing the GitHub Release,
update `latest.json` on your server with the new version, notes, pub_date,
and the `.sig` contents.

A future v1.1 improvement: automate update-manifest generation as part of the
release workflow using a GitHub Actions step that writes `latest.json` to an
S3 bucket or Cloudflare Worker.

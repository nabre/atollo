// The download button points at the visitor's system, and the file links at
// the latest release's assets (names as in .github/workflows/app.yml, «Release
// assets»). Without JavaScript, or before the first release, every link opens
// the releases page.
(function () {
  "use strict";
  const repo = "nabre/atollo";
  const latest = "https://github.com/" + repo + "/releases/latest";
  const names = {
    "win32-x64": "Windows x64",
    "win32-arm64": "Windows ARM64",
    "darwin-arm64": "macOS Apple Silicon",
    "darwin-x64": "macOS Intel",
    "linux-x64": "Linux x64",
    "linux-arm64": "Linux ARM64",
  };
  // The file the main button downloads for each target.
  const main = {
    "win32-x64": "Atollo-x64.exe",
    "win32-arm64": "Atollo-arm64.exe",
    "darwin-arm64": "Atollo-darwin-arm64.app.zip",
    "darwin-x64": "Atollo-darwin-x64.app.zip",
    "linux-x64": "Atollo-linux-x64.AppImage",
    "linux-arm64": "Atollo-linux-arm64.AppImage",
  };

  async function detect() {
    const ua = navigator.userAgent;
    let os = /Windows/.test(ua) ? "win32" : /Mac OS X|Macintosh/.test(ua) ? "darwin" : /Linux|X11/.test(ua) && !/Android/.test(ua) ? "linux" : null;
    if (!os) return null;
    let arch = null;
    const data = navigator.userAgentData;
    if (data && data.getHighEntropyValues) {
      try {
        const v = await data.getHighEntropyValues(["architecture", "bitness"]);
        if (v.architecture === "arm") arch = "arm64";
        else if (v.architecture === "x86") arch = "x64";
      } catch (e) {
        /* not allowed: guess below */
      }
    }
    if (!arch) {
      if (/aarch64|arm64/i.test(ua)) arch = "arm64";
      // Safari says "Intel Mac" on Apple Silicon too: most Macs sold since 2020 are ARM.
      else arch = os === "darwin" ? "arm64" : "x64";
    }
    return os + "-" + arch;
  }

  async function release() {
    try {
      const r = await fetch("https://api.github.com/repos/" + repo + "/releases/latest", {
        headers: { Accept: "application/vnd.github+json" },
      });
      if (r.status === 404) return { none: true };
      if (!r.ok) return null;
      return await r.json();
    } catch (e) {
      return null;
    }
  }

  async function run() {
    const cta = document.getElementById("download");
    const button = document.getElementById("dl-main");
    const note = document.getElementById("dl-note");
    if (!cta || !button) return;
    const [target, rel] = await Promise.all([detect(), release()]);

    if (target) {
      // The card in «Sistemi» and the column of «Strumenti per sistema».
      document.querySelectorAll('[data-target="' + target + '"]').forEach((el) => el.classList.add("here"));
      button.textContent = cta.dataset.for + " " + names[target];
    }
    // Rate limit or network error: the links stay on the releases page.
    if (!rel) return;
    if (rel.none) {
      button.textContent = cta.dataset.generic;
      note.textContent = cta.dataset.none;
      return;
    }

    const assets = new Map((rel.assets || []).map((a) => [a.name, a.browser_download_url]));
    document.querySelectorAll("[data-asset]").forEach((a) => {
      const url = assets.get(a.dataset.asset);
      if (url) a.href = url;
      else a.classList.add("missing");
    });
    if (target && assets.has(main[target])) button.href = assets.get(main[target]);
    else button.href = rel.html_url || latest;

    const version = String(rel.tag_name || "").replace(/^app-v/, "");
    if (version) {
      const link = document.createElement("a");
      link.href = rel.html_url || latest;
      link.textContent = cta.dataset.version + " " + version;
      note.prepend(link, " · ");
    }
  }

  // The other download buttons ([data-download], the page's finale) follow
  // the hero's, whatever run() decided.
  function follow() {
    const button = document.getElementById("dl-main");
    if (!button) return;
    document.querySelectorAll("[data-download]").forEach((a) => {
      a.href = button.href;
      a.textContent = button.textContent;
    });
  }

  run().finally(follow);
})();

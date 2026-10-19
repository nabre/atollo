// The download button points at the visitor's system, and the file links at
// the latest release's assets (names as in .github/workflows/app.yml, «Release
// assets»: Atollo-<version>-<data-asset>). Without JavaScript, or before the
// first release, every link opens the releases page.
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
  // The file the main button downloads for each target, after the version.
  const main = {
    "win32-x64": "win32-x64.exe",
    "win32-arm64": "win32-arm64.exe",
    "darwin-arm64": "darwin-arm64.app.zip",
    "darwin-x64": "darwin-x64.app.zip",
    "linux-x64": "linux-x64.AppImage",
    "linux-arm64": "linux-arm64.AppImage",
  };

  async function detect() {
    // ?os=<target> shows the page as another system sees it.
    const forced = new URLSearchParams(location.search).get("os");
    if (forced && names[forced]) return forced;
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

  // The newest app release (app-v*): Apache and PHP have their own
  // releases in this repository, and GitHub's «latest» may be one of them.
  async function release() {
    try {
      const r = await fetch("https://api.github.com/repos/" + repo + "/releases?per_page=30", {
        headers: { Accept: "application/vnd.github+json" },
      });
      if (!r.ok) return null;
      const app = (await r.json()).find((x) => !x.draft && !x.prerelease && /^app-v/.test(x.tag_name || ""));
      return app || { none: true };
    } catch (e) {
      return null;
    }
  }

  // The hero's kit folder as it looks on the visitor's system: the window
  // bars (site.css, html[data-os]: the kit's and the gallery's), the app's file under its release name
  // (the macOS bundle keeps its name), the folder on a stick.
  function kit(target, version) {
    const card = document.querySelector(".kit");
    if (!card) return;
    const os = target.split("-")[0];
    const v = version || card.dataset.version;
    const look = {
      win32: { root: "E:\\Atollo\\", app: "Atollo-" + v + "-" + target + ".exe", sep: "\\" },
      darwin: { root: "/Volumes/KIT/Atollo/", app: "Atollo.app", sep: "/" },
      linux: { root: "/media/kit/Atollo/", app: "Atollo-" + v + "-" + target + ".AppImage", sep: "/" },
    }[os];
    if (!look) return;
    document.documentElement.dataset.os = os;
    const text = { root: look.root, app: look.app, projects: "projects" + look.sep, data: "data" + look.sep };
    card.querySelectorAll("[data-kit]").forEach((el) => {
      el.textContent = text[el.dataset.kit];
    });
  }

  async function run() {
    const cta = document.getElementById("download");
    const button = document.getElementById("dl-main");
    const note = document.getElementById("dl-note");
    if (!cta || !button) return;
    const [target, rel] = await Promise.all([detect(), release()]);

    if (target) {
      kit(target);
      // The card in «Sistemi» and the column of «Strumenti per sistema».
      document.querySelectorAll('[data-target="' + target + '"]').forEach((el) => el.classList.add("here"));
      button.textContent = cta.dataset.for + " " + names[target];
    }
    // Rate limit or network error: the links stay on the releases page.
    if (!rel) return;
    if (rel.none) {
      button.textContent = cta.dataset.generic;
      button.href = "https://github.com/" + repo + "/releases";
      note.textContent = cta.dataset.none;
      return;
    }

    const version = String(rel.tag_name || "").replace(/^app-v/, "");
    if (target && version) kit(target, version);
    const assets = new Map((rel.assets || []).map((a) => [a.name, a.browser_download_url]));
    const file = (rest) => "Atollo-" + version + "-" + rest;
    document.querySelectorAll("[data-asset]").forEach((a) => {
      const name = file(a.dataset.asset);
      const url = assets.get(name);
      if (!url) {
        a.classList.add("missing");
        return;
      }
      a.href = url;
      // A full name shown (not ".AppImage") follows the release's version.
      if (a.textContent.startsWith("Atollo-")) a.textContent = name;
    });
    if (target && assets.has(file(main[target]))) button.href = assets.get(file(main[target]));
    else button.href = rel.html_url || latest;

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

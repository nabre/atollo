// The page's motion: the header's background once the hero is scrolled past,
// sections revealed as they enter the view, the numbers counted up, the
// glow under the pointer on the feature tiles, the theme button and the
// language menu. Everything is visible without it (the .js class on <html>
// gates the hidden states).
(function () {
  "use strict";
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  // The theme: "system" (no html[data-theme]), "light" or "dark", kept in
  // localStorage (the <head> sets it before the first paint). What a media
  // query picks by itself (the gallery's <source>, theme-color) follows the
  // choice: its own query for the system, all or none of it otherwise.
  const root = document.documentElement;
  const media = Array.from(document.querySelectorAll("[media*='prefers-color-scheme']"));
  media.forEach((el) => (el.dataset.media = el.media));
  const cycle = document.querySelector(".theme-cycle");
  const order = ["system", "light", "dark"];
  function theme(pick) {
    if (pick === "system") delete root.dataset.theme;
    else root.dataset.theme = pick;
    media.forEach((el) => {
      const scheme = /light/.test(el.dataset.media) ? "light" : "dark";
      el.media = pick === "system" ? el.dataset.media : pick === scheme ? "all" : "not all";
    });
    if (cycle) {
      const label = `${cycle.dataset.label}: ${cycle.dataset[pick]}`;
      cycle.setAttribute("aria-label", label);
      cycle.title = label;
    }
  }
  theme(root.dataset.theme || "system");
  if (cycle) {
    cycle.addEventListener("click", () => {
      const pick = order[(order.indexOf(root.dataset.theme || "system") + 1) % order.length];
      theme(pick);
      try {
        if (pick === "system") localStorage.removeItem("atollo-theme");
        else localStorage.setItem("atollo-theme", pick);
      } catch (e) {
        // Storage blocked: the choice lasts for this page.
      }
    });
  }

  // The language menu closes on a click outside it and on Escape.
  const langMenu = document.querySelector(".lang-menu");
  if (langMenu) {
    document.addEventListener("click", (e) => {
      if (langMenu.open && !langMenu.contains(e.target)) langMenu.open = false;
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && langMenu.open) {
        langMenu.open = false;
        langMenu.querySelector("summary").focus();
      }
    });
  }

  const top = document.querySelector(".top");
  const onScroll = () => top && top.classList.toggle("scrolled", window.scrollY > 24);
  onScroll();
  window.addEventListener("scroll", onScroll, { passive: true });

  function count(el) {
    const end = Number(el.dataset.count);
    if (!end || reduced) return;
    const start = performance.now();
    const step = (now) => {
      const p = Math.min(1, (now - start) / 900);
      el.textContent = String(Math.round(end * (1 - Math.pow(1 - p, 3))));
      if (p < 1) requestAnimationFrame(step);
    };
    el.textContent = "0";
    requestAnimationFrame(step);
  }

  // The hero's numbers count up as its stats rise in (--d:6).
  setTimeout(() => document.querySelectorAll(".hero [data-count]").forEach(count), 780);

  const items = document.querySelectorAll(".reveal");
  if (!("IntersectionObserver" in window)) {
    items.forEach((el) => el.classList.add("in"));
  } else {
    // Siblings revealed together come in one after the other.
    items.forEach((el) => {
      const peers = Array.from(el.parentElement.children).filter((c) => c.classList.contains("reveal"));
      el.style.setProperty("--rd", Math.min(peers.indexOf(el), 5) * 60 + "ms");
    });
    const io = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (!e.isIntersecting) return;
          e.target.classList.add("in");
          e.target.querySelectorAll("[data-count]").forEach(count);
          io.unobserve(e.target);
        });
      },
      { rootMargin: "0px 0px -8% 0px", threshold: 0.12 },
    );
    items.forEach((el) => io.observe(el));
  }

  document.querySelectorAll(".tile").forEach((tile) => {
    tile.addEventListener("pointermove", (e) => {
      const r = tile.getBoundingClientRect();
      tile.style.setProperty("--mx", e.clientX - r.left + "px");
      tile.style.setProperty("--my", e.clientY - r.top + "px");
    });
  });
})();

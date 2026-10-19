// The page's motion: the header's background once the hero is scrolled past,
// sections revealed as they enter the view, the numbers counted up, the
// glow under the pointer on the feature tiles. Everything is visible
// without it (the .js class on <html> gates the hidden states).
(function () {
  "use strict";
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

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

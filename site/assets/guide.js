// The Guide's pages: the search over the topics (the app's rule: every
// word typed in the title, subtitle, code or keywords, guide/core/search.ts),
// the copy buttons of the code blocks, the zoom of the screenshots and the
// filters of the sources table and of the problems. Without JavaScript the
// page reads the same: the chapters' list stays, the code can be selected,
// a screenshot's link opens the image.
(function () {
  "use strict";

  // On a phone the chapters start folded, under the title.
  const toc = document.querySelector(".g-toc");
  if (toc && window.matchMedia("(max-width: 900px)").matches) toc.open = false;

  // The search: the language's index, read the first time the box gets focus.
  const box = document.querySelector(".g-search");
  const chapters = document.querySelector(".g-chapters");
  const results = document.querySelector(".g-results");
  const none = document.querySelector(".g-noresult");
  let index = null;
  function show() {
    const words = box.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
    const searching = words.length > 0 && index !== null;
    chapters.hidden = searching;
    results.hidden = !searching;
    none.hidden = true;
    if (!searching) return;
    const found = index.filter((t) => words.every((w) => t.hay.includes(w)));
    results.replaceChildren(
      ...found.map((t) => {
        const li = document.createElement("li");
        const a = document.createElement("a");
        a.href = box.dataset.base + t.href;
        const code = document.createElement("span");
        code.className = "g-code";
        code.textContent = t.code;
        const small = document.createElement("small");
        small.textContent = t.chapter;
        a.append(code, " " + t.title, small);
        li.append(a);
        return li;
      }),
    );
    none.hidden = found.length > 0;
  }
  if (box && chapters && results && none) {
    box.addEventListener("focus", () => {
      if (index) return;
      fetch(box.dataset.index)
        .then((r) => r.json())
        .then((list) => {
          index = list;
          show();
        })
        .catch(() => {
          // Without the index the chapters' list stays.
        });
    });
    box.addEventListener("input", show);
  }

  // The copy buttons of the code blocks.
  document.querySelectorAll(".g-copy").forEach((button) => {
    const label = button.textContent;
    button.addEventListener("click", () => {
      const code = button.closest(".g-codeblock").querySelector("pre").textContent;
      navigator.clipboard.writeText(code).then(() => {
        button.textContent = button.dataset.copied;
        setTimeout(() => (button.textContent = label), 1500);
      });
    });
  });

  // A screenshot in a dialog over the page; Esc or a click outside closes it.
  document.querySelectorAll(".g-zoom").forEach((link) => {
    link.addEventListener("click", (e) => {
      if (typeof HTMLDialogElement !== "function") return;
      e.preventDefault();
      const img = link.querySelector("img");
      const dialog = document.createElement("dialog");
      dialog.className = "g-zoom-dialog";
      const bar = document.createElement("form");
      bar.method = "dialog";
      const title = document.createElement("b");
      title.textContent = img.alt;
      const close = document.createElement("button");
      close.textContent = "×";
      close.setAttribute("aria-label", "Close");
      bar.append(title, close);
      const big = document.createElement("img");
      big.src = img.src;
      big.alt = img.alt;
      dialog.append(bar, big);
      dialog.addEventListener("click", (ev) => {
        if (ev.target === dialog) dialog.close();
      });
      dialog.addEventListener("close", () => dialog.remove());
      document.body.append(dialog);
      dialog.showModal();
    });
  });

  // The sources table: a kind and a text filter together.
  document.querySelectorAll(".g-sources").forEach((table) => {
    const input = table.querySelector(".g-filter");
    const buttons = table.querySelectorAll("button[data-group]");
    let group = "";
    function filter() {
      const q = input.value.trim().toLowerCase();
      table.querySelectorAll("tbody tr").forEach((tr) => {
        tr.hidden = (group !== "" && tr.dataset.group !== group) || (q !== "" && !tr.dataset.text.includes(q));
      });
    }
    buttons.forEach((b) =>
      b.addEventListener("click", () => {
        group = b.dataset.group;
        buttons.forEach((x) => x.setAttribute("aria-pressed", String(x === b)));
        filter();
      }),
    );
    input.addEventListener("input", filter);
  });

  // The problems: the questions whose text has the words typed.
  document.querySelectorAll(".g-filterlist").forEach((listEl) => {
    const input = listEl.querySelector(".g-filter");
    input.addEventListener("input", () => {
      const q = input.value.trim().toLowerCase();
      listEl.querySelectorAll(".g-section").forEach((section) => {
        let any = false;
        section.querySelectorAll(".g-qa").forEach((qa) => {
          qa.hidden = q !== "" && !qa.dataset.text.includes(q);
          any = any || !qa.hidden;
        });
        section.hidden = !any;
      });
    });
  });
})();

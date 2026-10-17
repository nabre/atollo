// The focus of the modal boxes (useModalFocus): it goes inside when the box
// opens, Tab and Shift+Tab stay inside, Esc closes, and once closed it goes
// back to the control that opened the box. The same for ConfirmDialog and
// the zoomed screenshot of the Guide. The yes/no questions in the page
// (InlineConfirm) are alertdialogs named by their question, with the focus
// on the safe answer.
import { useRef, useState } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { ConfirmDialog, InlineConfirm, useModalFocus } from "../../../ui/src/workbench";
import { GuideShot } from "../../../ui/src/guide/widgets";
import { DatabaseCopies } from "../../../ui/src/backups";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function Box({ onClose, empty = false }: { onClose: () => void; empty?: boolean }) {
  const box = useRef<HTMLDivElement>(null);
  useModalFocus(box, onClose);
  return (
    <div ref={box} role="dialog" aria-label="Box" tabIndex={-1}>
      {!empty && (
        <>
          <button type="button">First</button>
          <button type="button" disabled>
            Off
          </button>
          <button type="button">Last</button>
        </>
      )}
    </div>
  );
}

/** A page with the button that opens the box, and the box. */
function Page({ empty = false, onClose = () => undefined }: { empty?: boolean; onClose?: () => void }) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button
        type="button"
        onClick={() => {
          setOpen(true);
        }}
      >
        Open
      </button>
      {open && (
        <Box
          empty={empty}
          onClose={() => {
            onClose();
            setOpen(false);
          }}
        />
      )}
    </>
  );
}

function open() {
  const opener = screen.getByRole("button", { name: "Open" });
  opener.focus();
  fireEvent.click(opener);
  return opener;
}

function tab(shiftKey = false) {
  fireEvent.keyDown(window, { key: "Tab", shiftKey });
}

describe("useModalFocus", () => {
  it("puts the focus on the first control and keeps Tab inside", () => {
    render(<Page />);
    open();
    const first = screen.getByRole("button", { name: "First" });
    const last = screen.getByRole("button", { name: "Last" });
    expect(document.activeElement).toBe(first);

    tab(true);
    expect(document.activeElement).toBe(last);
    tab();
    expect(document.activeElement).toBe(first);

    // A focus that escaped the box comes back on the next Tab.
    screen.getByRole("button", { name: "Open" }).focus();
    tab();
    expect(document.activeElement).toBe(first);
  });

  it("closes with Esc and gives the focus back to the opener", () => {
    const onClose = vi.fn();
    render(<Page onClose={onClose} />);
    const opener = open();
    expect(document.activeElement).not.toBe(opener);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(opener);
  });

  it("focuses the box itself when it has no control, and Tab does not leave it", () => {
    render(<Page empty />);
    open();
    const box = screen.getByRole("dialog", { name: "Box" });
    expect(document.activeElement).toBe(box);
    const event = new KeyboardEvent("keydown", { key: "Tab", cancelable: true });
    window.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });
});

describe("ConfirmDialog", () => {
  it("is an alertdialog named by its title and described by its text, with the focus inside", () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog title="Chiudere?" cancelLabel="Annulla" onCancel={onCancel} actions={null}>
        <p>Le attività in corso si fermano.</p>
      </ConfirmDialog>,
    );
    const dialog = screen.getByRole("alertdialog", { name: "Chiudere?" });
    expect(dialog.getAttribute("aria-describedby")).toBeTruthy();
    expect(document.getElementById(dialog.getAttribute("aria-describedby") ?? "")?.textContent).toBe(
      "Le attività in corso si fermano.",
    );
    expect(dialog.contains(document.activeElement)).toBe(true);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});

describe("InlineConfirm", () => {
  it("is an alertdialog named by its question, described by its text, with the focus on the safe answer", () => {
    const onCancel = vi.fn();
    render(
      <InlineConfirm
        row
        question="Ripristinare copia.sql?"
        onCancel={onCancel}
        actions={
          <>
            <button type="button">Ripristina</button>
            <button type="button">Annulla</button>
          </>
        }
      >
        I dati attuali vengono sostituiti.
      </InlineConfirm>,
    );
    const dialog = screen.getByRole("alertdialog", { name: "Ripristinare copia.sql?" });
    expect(document.getElementById(dialog.getAttribute("aria-describedby") ?? "")?.textContent).toBe(
      "I dati attuali vengono sostituiti.",
    );
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Annulla" }));
    fireEvent.keyDown(document.activeElement ?? dialog, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("gives the focus back to the button it replaced once it closes", () => {
    function Asker() {
      const [asking, setAsking] = useState(false);
      return (
        <div>
          {asking ? (
            <InlineConfirm
              question="Eliminare?"
              onCancel={() => {
                setAsking(false);
              }}
              actions={
                <button
                  type="button"
                  onClick={() => {
                    setAsking(false);
                  }}
                >
                  Annulla
                </button>
              }
            />
          ) : (
            <button
              type="button"
              onClick={() => {
                setAsking(true);
              }}
            >
              Elimina
            </button>
          )}
        </div>
      );
    }
    render(<Asker />);
    const ask = () => {
      const trigger = screen.getByRole("button", { name: "Elimina" });
      trigger.focus();
      fireEvent.click(trigger);
      expect(document.activeElement).toBe(screen.getByRole("button", { name: "Annulla" }));
    };
    ask();
    fireEvent.keyDown(document.activeElement ?? document.body, { key: "Escape" });
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Elimina" }));
    ask();
    fireEvent.click(screen.getByRole("button", { name: "Annulla" }));
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Elimina" }));
  });

  it("has no description when its text is empty", () => {
    render(
      <InlineConfirm question="Applicare?" actions={<button type="button">Annulla</button>}>
        {[false, false]}
      </InlineConfirm>,
    );
    expect(screen.getByRole("alertdialog", { name: "Applicare?" }).hasAttribute("aria-describedby")).toBe(false);
  });

  it("takes its name from `label` when the question is in the text, without a description of its own", () => {
    render(
      <InlineConfirm label="Uscire?" actions={<button type="button">Annulla</button>}>
        {null}
      </InlineConfirm>,
    );
    const dialog = screen.getByRole("alertdialog", { name: "Uscire?" });
    expect(dialog.hasAttribute("aria-describedby")).toBe(false);
  });
});

describe("DatabaseCopies", () => {
  it("asks before a restore with an alertdialog, and Esc gives up", async () => {
    invoke.mockImplementation((cmd) =>
      cmd === "project_backups"
        ? Promise.resolve([{ file: "shop-1.sql", engine: "mariadb", size: 2048 }])
        : Promise.reject(new Error(`unexpected ${cmd}`)),
    );
    render(<DatabaseCopies project="shop" engine="mariadb" />);
    fireEvent.click(await screen.findByRole("button", { name: "Ripristina…" }));
    const dialog = screen.getByRole("alertdialog", { name: "Ripristinare shop-1.sql?" });
    expect(dialog.getAttribute("aria-describedby")).toBeTruthy();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Annulla" }));
    fireEvent.keyDown(document.activeElement ?? dialog, { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("project_db_restore", expect.anything());
  });
});

describe("GuideShot zoom", () => {
  it("opens a modal with the focus inside, Esc closes it and the focus goes back", () => {
    render(<GuideShot src="shot.webp" title="Progetti" caption="La pagina dei progetti" />);
    const zoom = screen.getByTitle("Clicca per ingrandire l'immagine");
    zoom.focus();
    fireEvent.click(zoom);
    const dialog = screen.getByRole("dialog", { name: "Progetti" });
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(dialog.contains(document.activeElement)).toBe(true);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(zoom);
  });
});

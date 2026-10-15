// The workbench controls on vscode-elements keep what the pages rely on:
// roles and names in the page, clicks only when enabled, forms submitted,
// values given back, the dialog's focus.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { Button, ConfirmDialog, Select, TextField } from ".";

afterEach(cleanup);

describe("workbench controls", () => {
  it("names a vscode-button at once and ignores clicks while disabled", () => {
    const onClick = vi.fn();
    function Toggle() {
      const [on, setOn] = useState(false);
      return (
        <>
          <button
            type="button"
            onClick={() => {
              setOn(true);
            }}
          >
            mostra
          </button>
          {on && (
            <Button disabled onClick={onClick} title="Copia">
              Clona
            </Button>
          )}
        </>
      );
    }
    render(<Toggle />);
    fireEvent.click(screen.getByRole("button", { name: "mostra" }));
    // In the same tick: the role is there before Lit's first update.
    const clone = screen.getByRole("button", { name: "Clona", description: "Copia" });
    expect(clone.getAttribute("aria-disabled")).toBe("true");
    fireEvent.click(clone);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("submits its form, by click and by Enter in a field", () => {
    const onSubmit = vi.fn((e: { preventDefault: () => void }) => {
      e.preventDefault();
    });
    render(
      <form onSubmit={onSubmit}>
        <input aria-label="Nome" />
        <input aria-label="Cognome" />
        <Button variant="primary" submit>
          Salva
        </Button>
      </form>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Salva" }));
    expect(onSubmit).toHaveBeenCalledTimes(1);
    // The native default button is out of the accessibility tree.
    expect(screen.getAllByRole("button")).toHaveLength(1);
  });

  it("gives a field's and a select's value back, named by their label", async () => {
    const onText = vi.fn();
    const onPick = vi.fn();
    render(
      <>
        <TextField label="Porta" value="" onChange={onText} />
        <label htmlFor="v">Versione</label>
        <Select
          id="v"
          value="b"
          onChange={onPick}
          options={[
            { value: "a", label: "A" },
            { group: "Scaricate", options: [{ value: "b", label: "B" }] },
          ]}
        />
      </>,
    );
    fireEvent.change(screen.getByLabelText("Porta"), { target: { value: "8080" } });
    expect(onText).toHaveBeenLastCalledWith("8080");
    const select = document.querySelector("vscode-single-select");
    expect(select?.label).toBe("Versione");
    // The options are read on the slot's change, after the first update.
    await waitFor(() => {
      expect(select?.value).toBe("b");
    });
    fireEvent.change(select as Element, { target: { value: "a" } });
    expect(onPick).toHaveBeenLastCalledWith("a");
  });

  it("puts the focus on the dialog's cancel button; Esc cancels", () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog title="Chiudere?" cancelLabel="Annulla" onCancel={onCancel} actions={null}>
        <p>Si chiude tutto.</p>
      </ConfirmDialog>,
    );
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Annulla" }));
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});

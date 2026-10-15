// The workbench controls on vscode-elements keep what the pages rely on:
// roles and names in the page, clicks only when enabled, forms submitted
// (and not while busy), controlled values, the dialog's focus and Esc.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { Button, Checkbox, CheckboxLabel, ConfirmDialog, Radio, SearchBox, Select, TextField } from ".";

afterEach(cleanup);

/** The input inside a vscode-textfield, once drawn. */
async function innerInput(host: Element) {
  const field = host as HTMLElementTagNameMap["vscode-textfield"];
  await field.updateComplete;
  return field.wrappedElement;
}

/** Types into a component's inner input as the browser does: the value,
 * then a composed "input" that reaches the host. */
function type(input: HTMLInputElement, text: string) {
  input.value = text;
  input.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
}

function submitForm(busy: boolean, onSubmit: () => void) {
  return render(
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
    >
      <TextField label="Nome" value="" onChange={() => undefined} />
      <Button variant="primary" submit busy={busy}>
        Salva
      </Button>
    </form>,
  );
}

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

  it("clicks once per key press, without scrolling on Space", async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Salva</Button>);
    const button = screen.getByRole("button", { name: "Salva" });
    await (button as HTMLElementTagNameMap["vscode-button"]).updateComplete;
    expect(fireEvent.keyDown(button, { key: " " })).toBe(false);
    expect(onClick).toHaveBeenCalledTimes(1);
    fireEvent.keyDown(button, { key: "Enter" });
    fireEvent.keyDown(button, { key: "Enter", repeat: true });
    expect(onClick).toHaveBeenCalledTimes(2);
  });

  it("submits its form by click and by Enter in a TextField", async () => {
    const onSubmit = vi.fn();
    submitForm(false, onSubmit);
    fireEvent.click(screen.getByRole("button", { name: "Salva" }));
    expect(onSubmit).toHaveBeenCalledTimes(1);
    const input = await innerInput(screen.getByLabelText("Nome"));
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSubmit).toHaveBeenCalledTimes(2);
    // The native default button is out of the accessibility tree.
    expect(screen.getAllByRole("button")).toHaveLength(1);
  });

  it("doesn't submit while its submit button is busy, by click or by Enter", async () => {
    const onSubmit = vi.fn();
    submitForm(true, onSubmit);
    fireEvent.click(screen.getByRole("button", { name: "Salva" }));
    const input = await innerInput(screen.getByLabelText("Nome"));
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("gives a field's typed value back and shows the value the page keeps", async () => {
    function Port() {
      const [port, setPort] = useState("12");
      return (
        <>
          <TextField
            label="Porta"
            value={port}
            invalid={port === ""}
            inputMode="numeric"
            onChange={(v) => {
              setPort(v.replace(/\D/g, ""));
            }}
          />
          <output>{port}</output>
        </>
      );
    }
    render(<Port />);
    const host = screen.getByLabelText<HTMLElementTagNameMap["vscode-textfield"]>("Porta");
    const input = await innerInput(host);
    expect(input.getAttribute("aria-label")).toBe("Porta");
    expect(input.getAttribute("inputmode")).toBe("numeric");
    act(() => {
      type(input, "123");
    });
    expect(screen.getByRole("status").textContent).toBe("123");
    // Rejected: nothing changes in the page, the field shows its value again.
    act(() => {
      type(input, "123a");
    });
    expect(screen.getByRole("status").textContent).toBe("123");
    expect(host.value).toBe("123");
    expect(input.value).toBe("123");
    // The error state is on the input that has the focus.
    act(() => {
      type(input, "");
    });
    await waitFor(() => {
      expect(input.getAttribute("aria-invalid")).toBe("true");
    });
  });

  it("filters as one types in a SearchBox", async () => {
    const onChange = vi.fn();
    render(<SearchBox placeholder="Cerca" value="" onChange={onChange} />);
    const input = await innerInput(screen.getByLabelText("Cerca"));
    act(() => {
      type(input, "nod");
    });
    expect(onChange).toHaveBeenLastCalledWith("nod");
  });

  it("gives a select's value back, named by its label, picked from its list", async () => {
    const onPick = vi.fn();
    render(
      <>
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
    const select = document.querySelector("vscode-single-select");
    if (!select) throw new Error("no select");
    expect(select.label).toBe("Versione");
    // The options are read on the slot's change, after the first update.
    await waitFor(() => {
      expect(select.value).toBe("b");
    });
    // jsdom has no popover, which the list opens in.
    const proto = HTMLElement.prototype as Partial<Pick<HTMLElement, "showPopover" | "hidePopover">>;
    proto.showPopover = () => undefined;
    proto.hidePopover = () => undefined;
    try {
      // A click on an option of the open list.
      fireEvent.keyDown(select, { key: " " });
      await select.updateComplete;
      expect(select.open).toBe(true);
      const first = select.shadowRoot?.querySelector<HTMLElement>('li.option[data-index="0"]');
      if (!first) throw new Error("no option");
      fireEvent.click(first);
      expect(onPick).toHaveBeenLastCalledWith("a");
      // Enter on the open list picks the active option.
      fireEvent.keyDown(select, { key: " " });
      await select.updateComplete;
      fireEvent.keyDown(select, { key: "ArrowDown" });
      fireEvent.keyDown(select, { key: "ArrowDown" });
      fireEvent.keyDown(select, { key: "Enter" });
      expect(onPick).toHaveBeenLastCalledWith("b");
      await select.updateComplete;
      expect(select.open).toBe(false);
    } finally {
      delete proto.showPopover;
      delete proto.hidePopover;
    }
  });

  it("shows the first option, not a group's header, for a value no option has", async () => {
    render(
      <Select
        label="Linea"
        value="zz"
        onChange={() => undefined}
        options={[{ group: "G1", options: [{ value: "x", label: "X" }] }]}
      />,
    );
    const select = document.querySelector("vscode-single-select");
    await waitFor(() => {
      expect(select?.value).toBe("x");
    });
  });

  it("gives back a checkbox's and a radio's choice", () => {
    const onBox = vi.fn();
    const onLabelled = vi.fn();
    const onRadio = vi.fn();
    render(
      <>
        <Checkbox label="Attivo" checked={false} onChange={onBox} />
        <CheckboxLabel checked onChange={onLabelled}>
          Avvio automatico
        </CheckboxLabel>
        <Radio name="r" checked={false} onSelect={onRadio}>
          Locale
        </Radio>
      </>,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Attivo" }));
    expect(onBox).toHaveBeenLastCalledWith(true);
    fireEvent.click(screen.getByText("Avvio automatico"));
    expect(onLabelled).toHaveBeenLastCalledWith(false);
    fireEvent.click(screen.getByRole("radio", { name: "Locale" }));
    expect(onRadio).toHaveBeenCalledTimes(1);
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

  it("lets Esc cancel a dialog from a closed select inside it", async () => {
    const onCancel = vi.fn();
    render(
      <ConfirmDialog title="Chiudere?" cancelLabel="Annulla" onCancel={onCancel} actions={null}>
        <Select
          label="Linea"
          value="a"
          onChange={() => undefined}
          options={[
            { value: "a", label: "A" },
            { value: "b", label: "B" },
          ]}
        />
      </ConfirmDialog>,
    );
    const select = document.querySelector("vscode-single-select");
    if (!select) throw new Error("no select");
    await select.updateComplete;
    fireEvent.keyDown(select, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});

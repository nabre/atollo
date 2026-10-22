// The workbench controls on vscode-elements keep what the pages rely on:
// roles and names in the page, clicks only when enabled, forms submitted
// (and not while busy), controlled values, the dialog's focus and Esc,
// suggestions taken without submitting, toggles' and meters' states.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { codicon } from "@/workbench/codicon";
import {
  Button,
  Checkbox,
  CheckboxLabel,
  ConfirmDialog,
  IconButton,
  Meter,
  Radio,
  SearchBox,
  Select,
  SuggestField,
  TextField,
  ToggleButton,
} from "@/workbench";
// VSCodium 1.135's workbench, vendored by scripts/vscodium-css/extract.mjs.
import referenceCss from "../../../docs/brand/vscodium-1.135/workbench-ref.css?raw";

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

  it("keeps Tab and Shift+Tab inside the dialog, past a hidden submit button", () => {
    render(
      <ConfirmDialog
        title="Chiudere?"
        cancelLabel="Annulla"
        onCancel={() => undefined}
        actions={
          <form>
            <Button variant="primary" submit>
              Conferma
            </Button>
          </form>
        }
      >
        <p>Si chiude tutto.</p>
      </ConfirmDialog>,
    );
    const cancel = screen.getByRole("button", { name: "Annulla" });
    const confirm = screen.getByRole("button", { name: "Conferma" });
    expect(document.activeElement).toBe(cancel);
    fireEvent.keyDown(window, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(confirm);
    // The last reachable control is Conferma, not the hidden native submit.
    fireEvent.keyDown(window, { key: "Tab" });
    expect(document.activeElement).toBe(cancel);
  });

  it("gives the focus back to what opened the dialog once it closes", () => {
    function Opener() {
      const [open, setOpen] = useState(false);
      return (
        <>
          <button
            type="button"
            onClick={() => {
              setOpen(true);
            }}
          >
            Elimina
          </button>
          {open && (
            <ConfirmDialog
              title="Eliminare?"
              cancelLabel="Annulla"
              onCancel={() => {
                setOpen(false);
              }}
              actions={null}
            >
              <p>Non si torna indietro.</p>
            </ConfirmDialog>
          )}
        </>
      );
    }
    render(<Opener />);
    const opener = screen.getByRole("button", { name: "Elimina" });
    opener.focus();
    fireEvent.click(opener);
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Annulla" }));
    act(() => {
      fireEvent.keyDown(window, { key: "Escape" });
    });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(document.activeElement).toBe(opener);
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

  it("says that an icon action opens a menu, and which one while it is open", () => {
    const { rerender } = render(<IconButton icon={codicon("add")} label="Aggiungi" onClick={() => undefined} />);
    const plain = screen.getByRole("button", { name: "Aggiungi" });
    // Its glyph, the 16px codicon, hidden: the label names the button.
    const glyph = plain.querySelector(".codicon.codicon-add");
    expect(glyph?.classList.contains("text-vsc-codicon")).toBe(true);
    expect(glyph?.getAttribute("aria-hidden")).toBe("true");
    expect(plain.hasAttribute("aria-haspopup")).toBe(false);
    expect(plain.hasAttribute("aria-expanded")).toBe(false);
    const draw = (open: boolean) => (
      <IconButton icon={codicon("add")} label="Aggiungi" menu={{ open, id: "m" }} onClick={() => undefined} />
    );
    rerender(draw(false));
    const button = screen.getByRole("button", { name: "Aggiungi" });
    expect(button.getAttribute("aria-haspopup")).toBe("menu");
    expect(button.getAttribute("aria-expanded")).toBe("false");
    expect(button.hasAttribute("aria-controls")).toBe(false);
    rerender(draw(true));
    expect(button.getAttribute("aria-expanded")).toBe("true");
    expect(button.getAttribute("aria-controls")).toBe("m");
  });

  it("gives autocomplete to a TextField again when it is drawn again", () => {
    const field = () => (
      <TextField label="Chiave" type="password" autoComplete="off" value="" onChange={() => undefined} />
    );
    const { rerender } = render(field());
    const first = document.querySelector("vscode-textfield");
    expect(first?.autocomplete).toBe("off");
    rerender(<></>);
    rerender(field());
    const second = document.querySelector("vscode-textfield");
    expect(second).not.toBe(first);
    expect(second?.autocomplete).toBe("off");
  });

  it("says whether a toggle with text is pressed, and ignores clicks while disabled", () => {
    const onClick = vi.fn();
    const draw = (pressed: boolean, disabled = false) => (
      <ToggleButton pressed={pressed} disabled={disabled} title="Cartella vuota" onClick={onClick}>
        vuoto
      </ToggleButton>
    );
    const { rerender } = render(draw(false));
    const chip = screen.getByRole("button", { name: "vuoto", pressed: false, description: "Cartella vuota" });
    fireEvent.click(chip);
    expect(onClick).toHaveBeenCalledTimes(1);
    rerender(draw(true));
    expect(screen.getByRole("button", { name: "vuoto", pressed: true })).toBe(chip);
    // Pressed: the toggle colors of VSCodium's input options.
    expect(chip.className).toContain("bg-(--vscode-inputOption-activeBackground)");
    expect(chip.className).not.toMatch(ONLY_TOKENS);
    rerender(draw(true, true));
    fireEvent.click(chip);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("gives a meter its value, its range and its text, kept inside the range", () => {
    const { rerender } = render(<Meter label="Sessione" value={40} valueText="40% usato" tone="warning" />);
    const meter = screen.getByRole("meter", { name: "Sessione" });
    expect(meter.getAttribute("aria-valuenow")).toBe("40");
    expect(meter.getAttribute("aria-valuemin")).toBe("0");
    expect(meter.getAttribute("aria-valuemax")).toBe("100");
    expect(meter.getAttribute("aria-valuetext")).toBe("40% usato");
    const bar = meter.firstElementChild as HTMLElement;
    expect(bar.style.width).toBe("40%");
    expect(bar.className).toContain("bg-vsc-warning");
    expect(meter.className).not.toMatch(ONLY_TOKENS);
    rerender(<Meter label="Sessione" value={130} max={200} />);
    expect(meter.getAttribute("aria-valuenow")).toBe("130");
    expect(bar.style.width).toBe("65%");
    rerender(<Meter label="Sessione" value={-5} />);
    expect(meter.getAttribute("aria-valuenow")).toBe("0");
    expect(bar.style.width).toBe("0%");
  });

  it("suggests values as one types, taken with the arrows and Enter, without submitting", async () => {
    const onSubmit = vi.fn();
    function Add() {
      const [id, setId] = useState("");
      return (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit(id);
          }}
        >
          <SuggestField
            label="Estensione"
            value={id}
            onChange={setId}
            suggestions={[
              { value: "ms-python.python", label: "Python" },
              { value: "charliermarsh.ruff", label: "Ruff" },
              { value: "esbenp.prettier-vscode", label: "Prettier" },
            ]}
          />
          <Button variant="primary" submit>
            Aggiungi
          </Button>
        </form>
      );
    }
    render(<Add />);
    const host = screen.getByLabelText<HTMLElementTagNameMap["vscode-textfield"]>("Estensione");
    const input = await innerInput(host);
    expect(screen.queryByRole("listbox")).toBeNull();
    await waitFor(() => {
      expect(input.getAttribute("role")).toBe("combobox");
    });
    expect(input.getAttribute("aria-expanded")).toBe("false");
    // Matched in the value or in the label.
    act(() => {
      type(input, "py");
    });
    const list = screen.getByRole("listbox", { name: "Estensione" });
    expect(screen.getAllByRole("option").map((o) => o.textContent)).toEqual(["ms-python.pythonPython"]);
    act(() => {
      type(input, "r");
    });
    expect(screen.getAllByRole("option")).toHaveLength(2);
    await waitFor(() => {
      expect(input.getAttribute("aria-expanded")).toBe("true");
    });
    expect(list.isConnected).toBe(true);
    // Down twice, Enter: the second one, and the form is not submitted.
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    expect(screen.getByRole("option", { selected: true }).textContent).toContain("esbenp.prettier-vscode");
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSubmit).not.toHaveBeenCalled();
    expect(host.value).toBe("esbenp.prettier-vscode");
    expect(screen.queryByRole("listbox")).toBeNull();
    // With the list closed, Enter submits the text as typed.
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onSubmit).toHaveBeenCalledWith("esbenp.prettier-vscode");
  });

  it("takes a suggestion by click, keeps any text, and closes the list with Esc alone", async () => {
    const onChange = vi.fn();
    const onCancel = vi.fn();
    function Field() {
      const [id, setId] = useState("");
      return (
        <ConfirmDialog title="Aggiungere?" cancelLabel="Annulla" onCancel={onCancel} actions={null}>
          <SuggestField
            label="Estensione"
            value={id}
            onChange={(v) => {
              onChange(v);
              setId(v);
            }}
            suggestions={[{ value: "ms-python.python" }, { value: "charliermarsh.ruff" }]}
          />
        </ConfirmDialog>
      );
    }
    render(<Field />);
    const input = await innerInput(screen.getByLabelText("Estensione"));
    act(() => {
      type(input, "nessuna.corrispondenza");
    });
    // Nothing matches: no list, the text stays.
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(onChange).toHaveBeenLastCalledWith("nessuna.corrispondenza");
    act(() => {
      type(input, "ruff");
    });
    fireEvent.keyDown(input, { key: "Escape" });
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(onCancel).not.toHaveBeenCalled();
    // The arrow opens it again; a click takes one.
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.click(screen.getByRole("option", { name: "charliermarsh.ruff" }));
    expect(onChange).toHaveBeenLastCalledWith("charliermarsh.ruff");
    expect(screen.queryByRole("listbox")).toBeNull();
    // Closed, Esc is the dialog's again.
    fireEvent.keyDown(input, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});

/** A property of a selector in VSCodium's reference (one rule per line). */
function reference(selector: string, property: string) {
  const pattern = new RegExp(`[{;]${property}:([^;}]+)`);
  for (const line of referenceCss.split("\n")) {
    const value = line.startsWith(`${selector}{`) ? pattern.exec(line)?.[1] : undefined;
    if (value !== undefined) return value.trim();
  }
  return undefined;
}

/** The CSS a component's shadow root has: the sheets Lit adopts. */
async function shadowCss(el: Element | null) {
  const host = el as HTMLElementTagNameMap["vscode-textfield"] | null;
  await host?.updateComplete;
  const sheets = host?.shadowRoot?.adoptedStyleSheets ?? [];
  return sheets.flatMap((s) => [...s.cssRules].map((r) => r.cssText)).join("\n");
}

/** Spacing utilities and VSCodium variables only: no measure in pixels. */
const ONLY_TOKENS = /\d+px|-\[\d/;

describe("VSCodium 1.135's measures", () => {
  it("draws a text button as .monaco-text-button, and the small one as .small", async () => {
    expect(reference(".monaco-text-button", "padding")).toBe("4px 8px");
    expect(reference(".monaco-text-button", "line-height")).toBe("16px");
    expect(reference(".monaco-text-button", "font-size")).toBe("12px");
    expect(reference(".monaco-text-button.small", "padding")).toBe("3px 6px");
    render(
      <>
        <Button>Apri</Button>
        <Button small>Aggiungi</Button>
      </>,
    );
    const text = screen.getByRole("button", { name: "Apri" }).className;
    for (const c of ["px-vsc-size80", "py-vsc-size40", "text-vsc-body-small", "rounded-vsc-small"]) {
      expect(text).toContain(`[&::part(base)]:${c}`);
    }
    expect(text).toContain("[&::part(base)]:leading-(--vscode-spacing-size160)");
    const small = screen.getByRole("button", { name: "Aggiungi" }).className;
    for (const c of ["px-vsc-size60", "py-vsc-size20", "text-vsc-body-xsmall"]) {
      expect(small).toContain(`[&::part(base)]:${c}`);
    }
    for (const name of [text, small]) {
      expect(name).not.toMatch(ONLY_TOKENS);
      // The component's own focus ring (1px focusBorder, 2px out, as
      // VSCodium's) stays: no class draws another.
      expect(name).not.toContain("outline");
    }
    expect(reference(".monaco-text-button:focus", "outline-offset")).toBe("2px!important");
    const own = await shadowCss(screen.getByRole("button", { name: "Apri" }));
    expect(own).toMatch(
      /:host\(:focus\) \.base \{[^}]*outline: 1px solid var\(--vscode-focusBorder[^}]*outline-offset: 2px/,
    );
  });

  it("gives fields and dropdowns the input box's padding, and the dropdown its corners and shadow", async () => {
    expect(reference(".monaco-inputbox>.ibwrapper>.input,.monaco-inputbox>.ibwrapper>.mirror", "padding")).toBe(
      "4px 6px",
    );
    expect(reference(".monaco-select-box", "border-radius")).toBe("var(--vscode-cornerRadius-small)");
    expect(
      reference(
        ".settings-editor>.settings-body .settings-tree-container .setting-item-contents .monaco-select-box",
        "height",
      ),
    ).toBe("26px");
    expect(reference(".monaco-select-box-dropdown-container", "box-shadow")).toBe("var(--vscode-shadow-lg)");
    expect(reference(".monaco-select-box-dropdown-container", "border-radius")).toBe(
      "var(--vscode-cornerRadius-large)",
    );
    render(
      <>
        <TextField label="Nome" value="" onChange={() => undefined} />
        <Select
          label="Linea"
          value="a"
          onChange={() => undefined}
          options={[
            { value: "a", label: "A" },
            { value: "b", label: "B" },
          ]}
        />
      </>,
    );
    const field = await shadowCss(document.querySelector("vscode-textfield"));
    expect(field).toMatch(
      /input:not\(\[type="file"\]\) \{\s*padding: var\(--vscode-spacing-size40\) var\(--vscode-spacing-size60\);/,
    );
    const select = await shadowCss(document.querySelector("vscode-single-select"));
    expect(select).toMatch(
      /\.select-face \{\s*padding: var\(--vscode-spacing-size40\) var\(--vscode-spacing-size60\);/,
    );
    expect(select).toMatch(/\.dropdown \{[^}]*border-radius: var\(--vscode-cornerRadius-large\)/);
    expect(select).toMatch(/\.dropdown \{[^}]*box-shadow: var\(--vscode-shadow-lg\)/);
    // The focus still turns the border to focusBorder, the component's own.
    expect(field).toContain("border-color: var(--vscode-focusBorder");
  });
});

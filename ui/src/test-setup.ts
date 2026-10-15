// What jsdom lacks for the Lit components of @vscode-elements/elements;
// the app's WebView2 has all of it. Lit itself falls back from
// adoptedStyleSheets to <style> elements on its own.

// Form-associated custom elements: jsdom's ElementInternals has the ARIA
// part only. The form part, as a browser gives it to an element that
// reports no errors: the value is kept, validity follows setValidity.
const proto = ElementInternals.prototype as Partial<ElementInternals>;
if (!("setFormValue" in proto)) {
  const hosts = new WeakMap<ElementInternals, HTMLElement>();
  const flags = new WeakMap<ElementInternals, { message: string; invalid: boolean }>();
  const states = new WeakMap<ElementInternals, Set<string>>();
  const attach = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "attachInternals")?.value as (
    this: HTMLElement,
  ) => ElementInternals;
  HTMLElement.prototype.attachInternals = function (this: HTMLElement) {
    const internals = attach.call(this);
    hosts.set(internals, this);
    return internals;
  };
  const state = (i: ElementInternals) => flags.get(i) ?? { message: "", invalid: false };
  Object.defineProperties(ElementInternals.prototype, {
    setFormValue: { value: () => undefined },
    setValidity: {
      value(this: ElementInternals, f: ValidityStateFlags = {}, message = "") {
        flags.set(this, { message, invalid: Object.values(f).some(Boolean) });
      },
    },
    form: {
      get(this: ElementInternals) {
        return hosts.get(this)?.closest("form") ?? null;
      },
    },
    labels: {
      get: () => [],
    },
    states: {
      get(this: ElementInternals) {
        const set = states.get(this) ?? new Set<string>();
        states.set(this, set);
        return set;
      },
    },
    validity: {
      get(this: ElementInternals) {
        return { valid: !state(this).invalid } as ValidityState;
      },
    },
    validationMessage: {
      get(this: ElementInternals) {
        return state(this).message;
      },
    },
    willValidate: { get: () => true },
    checkValidity: {
      value(this: ElementInternals) {
        return !state(this).invalid;
      },
    },
    reportValidity: {
      value(this: ElementInternals) {
        return !state(this).invalid;
      },
    },
  });
}

// The text of an element as drawn (vscode-single-select reads its options'
// labels so): without layout, the plain text.
if (!("innerText" in HTMLElement.prototype)) {
  Object.defineProperty(HTMLElement.prototype, "innerText", {
    configurable: true,
    get(this: HTMLElement) {
      return this.textContent;
    },
    set(this: HTMLElement, v: string) {
      this.textContent = v;
    },
  });
}

// Layout observers (vscode-scrollable, vscode-form-container): jsdom has no
// layout, so nothing ever resizes.
if (!("ResizeObserver" in globalThis)) {
  Object.defineProperty(globalThis, "ResizeObserver", {
    configurable: true,
    writable: true,
    value: class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  });
}

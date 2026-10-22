// The guard of freeze.ts, without its effect: tested on objects of its own.
// A frozen prototype makes `obj.toString = f` throw on any object (the
// "override mistake"): xterm.js does it, and the terminal panel did not
// load (D-46, D-148). Each method becomes an accessor first, as SES does:
// reading gives the method, assigning it on an object defines the object's
// own property, assigning it on the prototype itself still throws.

/** Makes the data properties of `proto` overridable on the objects that
 * inherit them, then freezes it. */
export function freezeOverridable(proto: object): void {
  for (const key of Reflect.ownKeys(proto)) {
    const desc = Object.getOwnPropertyDescriptor(proto, key);
    if (!desc || !("value" in desc) || !desc.configurable) continue;
    const value: unknown = desc.value;
    Object.defineProperty(proto, key, {
      get() {
        return value;
      },
      set(this: object, v: unknown) {
        if (this === proto) throw new TypeError(`Cannot assign to read only property '${String(key)}'`);
        Object.defineProperty(this, key, { value: v, writable: true, enumerable: true, configurable: true });
      },
      enumerable: desc.enumerable ?? false,
      configurable: false,
    });
  }
  Object.freeze(proto);
}

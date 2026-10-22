// The prototype guard of the app window (freeze.ts): the prototype cannot
// change, its methods can still be overridden on other objects (xterm.js
// assigns `toString` to its own: with a plain freeze the terminal panel
// did not load, D-148).
import { describe, expect, it } from "vitest";
import { freezeOverridable } from "@/freezeproto";

function makeProto() {
  const proto = {
    toString() {
      return "proto";
    },
    shared: 1,
  };
  freezeOverridable(proto);
  return proto;
}

describe("freezeOverridable", () => {
  it("lets an object override an inherited method", () => {
    const proto = makeProto();
    const obj = Object.create(proto) as { toString: () => string };
    expect(obj.toString()).toBe("proto");
    obj.toString = () => "own";
    expect(obj.toString()).toBe("own");
    expect(Object.prototype.hasOwnProperty.call(obj, "toString")).toBe(true);
    // Another object still sees the prototype's.
    expect((Object.create(proto) as { toString: () => string }).toString()).toBe("proto");
  });

  it("keeps the prototype itself unchangeable", () => {
    const proto = makeProto() as { toString: () => string; shared: number; added?: number };
    expect(() => {
      proto.toString = () => "polluted";
    }).toThrow(TypeError);
    expect(() => {
      proto.added = 1;
    }).toThrow(TypeError);
    expect(Object.isFrozen(proto)).toBe(true);
    expect(proto.toString()).toBe("proto");
    expect(proto.shared).toBe(1);
  });
});

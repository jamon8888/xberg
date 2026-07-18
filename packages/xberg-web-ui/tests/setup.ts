// tests/setup.ts
import "@testing-library/jest-dom/vitest";
import "fake-indexeddb/auto";

// jsdom doesn't implement ResizeObserver; stub it for components (e.g.
// ScrollArea) that observe element size in an effect.
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}

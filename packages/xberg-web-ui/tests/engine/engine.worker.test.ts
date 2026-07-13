import { describe, it, expect, vi, beforeEach } from "vitest";

const ocrMock = vi.fn();

vi.mock("@xberg-io/xberg-wasm", () => ({
  XbergEngine: class {
    constructor(_opts: unknown, _injections: unknown) {}
    async ocr(bytes: Uint8Array, opts: unknown): Promise<unknown> {
      return ocrMock(bytes, opts);
    }
  },
}));

vi.mock("xberg-wasm-runtime", () => ({
  createXbergRuntimeFactory: vi.fn().mockResolvedValue({}),
}));

describe("engine.worker handleOcr", () => {
  beforeEach(() => {
    ocrMock.mockReset();
  });

  it("posts real per-line text/confidence/bbox instead of a newline-split flat string", async () => {
    ocrMock.mockResolvedValue({
      text: "Hello\nWorld",
      lines: [
        { text: "Hello", confidence: 0.97, bbox: { x: 1, y: 2, w: 3, h: 4 } },
        { text: "World", confidence: 0.88, bbox: { x: 1, y: 10, w: 3, h: 4 } },
      ],
    });
    const postMessageSpy = vi.spyOn(self, "postMessage").mockImplementation(() => undefined);

    await import("../../src/engine/engine.worker.js");
    self.dispatchEvent(
      new MessageEvent("message", {
        data: { type: "ocr", requestId: "r1", bytes: new Uint8Array([1, 2, 3]) },
      })
    );

    await vi.waitFor(() => {
      expect(postMessageSpy).toHaveBeenCalled();
    });

    expect(postMessageSpy).toHaveBeenCalledWith(
      {
        type: "ocrResult",
        requestId: "r1",
        lines: [
          { text: "Hello", confidence: 0.97, bbox: { x: 1, y: 2, w: 3, h: 4 } },
          { text: "World", confidence: 0.88, bbox: { x: 1, y: 10, w: 3, h: 4 } },
        ],
      },
      []
    );
  });
});

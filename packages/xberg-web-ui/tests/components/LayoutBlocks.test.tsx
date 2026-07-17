import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { LayoutBlocks } from "../../src/components/LayoutBlocks.js";

// The OcrBlocksPanel branch virtualizes its list (@tanstack/react-virtual),
// which reads offsetWidth/offsetHeight to size its scroll container — jsdom
// always reports 0 for both, so the virtualizer renders nothing unless we
// stub a real size.
function stubNonZeroOffsetSize() {
  const widthDescriptor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "offsetWidth");
  const heightDescriptor = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "offsetHeight");
  Object.defineProperty(HTMLElement.prototype, "offsetWidth", { configurable: true, value: 400 });
  Object.defineProperty(HTMLElement.prototype, "offsetHeight", { configurable: true, value: 400 });
  return () => {
    if (widthDescriptor) Object.defineProperty(HTMLElement.prototype, "offsetWidth", widthDescriptor);
    if (heightDescriptor) Object.defineProperty(HTMLElement.prototype, "offsetHeight", heightDescriptor);
  };
}

describe("LayoutBlocks", () => {
  it("renders one region per OCR line", () => {
    render(
      <LayoutBlocks
        lines={[
          { text: "Hello", confidence: 0.95, bbox: { x: 10, y: 20, w: 100, h: 30 } },
          { text: "World", confidence: 0.6 },
        ]}
        width={200}
        height={120}
      />
    );
    expect(screen.getAllByTestId("layout-block")).toHaveLength(2);
  });

  it("renders geometry-derived blocks via OcrBlocksPanel when file is provided", async () => {
    const restore = stubNonZeroOffsetSize();
    try {
      render(
        <LayoutBlocks
          lines={[
            { text: "Hello", confidence: 0.95, bbox: { x: 10, y: 20, w: 100, h: 30 } },
          ]}
          width={200}
          height={120}
          file="sample.pdf"
        />
      );
      expect(await screen.findByText("Hello")).toBeInTheDocument();
      expect(screen.queryByTestId("layout-block")).not.toBeInTheDocument();
    } finally {
      restore();
    }
  });
});

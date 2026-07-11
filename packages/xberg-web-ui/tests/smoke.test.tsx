// tests/smoke.test.tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import HomePage from "../src/app/page.js";

describe("app shell smoke test", () => {
  it("renders the placeholder home page", () => {
    render(<HomePage />);
    expect(screen.getByText("Xberg — folders")).toBeDefined();
  });
});

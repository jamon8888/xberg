import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { SyncBar } from "../../src/components/SyncBar.js";
import { EngineProvider } from "../../src/providers/EngineProvider.js";

describe("SyncBar", () => {
  it("shows 'All synced' when nothing is pending and there is no error", () => {
    const fakeClient = { ingestFile: vi.fn() };
    render(
      <EngineProvider baseUrl="http://x:8080" workerClient={fakeClient as never}>
        <SyncBar />
      </EngineProvider>
    );
    expect(screen.getByText("All synced")).toBeDefined();
  });
});

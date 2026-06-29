import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import {
  extract,
  extractInputFromUri,
  type ExtractionConfig,
} from "@xberg-io/xberg";

export function registerWebTools(server: McpServer): void {
  server.tool(
    "scrape_url",
    "Fetch and extract content from a URL. Supports single pages, PDFs served over HTTPS, and multi-page crawls. " +
    "JS-rendered pages are handled automatically via headless browser fallback (mode: auto). " +
    "For crawling, set mode='crawl' and max_pages/max_depth.",
    {
      url: z.string().url().describe("HTTPS URL to scrape or crawl"),
      mode: z.enum(["document", "crawl"]).optional().default("document").describe(
        "'document' extracts a single page or file. 'crawl' follows links up to max_pages."
      ),
      max_pages: z.number().int().min(1).max(200).optional().default(1).describe(
        "Maximum pages to crawl (crawl mode only)."
      ),
      max_depth: z.number().int().min(1).max(5).optional().default(2).describe(
        "Maximum link-hop depth from the seed URL (crawl mode only)."
      ),
      js_rendering: z.enum(["auto", "always", "never"]).optional().default("auto").describe(
        "'auto' uses headless browser only when JS is detected. 'always' forces browser. 'never' is plain HTTP."
      ),
      allow_subdomains: z.boolean().optional().default(false).describe(
        "Allow crawling to subdomains of the seed URL."
      ),
    },
    async ({ url, mode, max_pages, max_depth, js_rendering, allow_subdomains }) => {
      try {
        const extractInput = extractInputFromUri(url);

        const config: ExtractionConfig = {
          url: {
            mode: mode as "document" | "crawl",
            crawl: {
              maxDepth: max_depth,
              maxPages: max_pages,
              allowSubdomains: allow_subdomains,
              browser: {
                mode: js_rendering as "auto" | "always" | "never",
              },
            },
          },
        };

        const result = await extract(extractInput, config);
        const docs = result.results ?? [];

        return {
          content: [{
            type: "text" as const,
            text: JSON.stringify({
              pages_extracted: docs.length,
              documents: docs.map((doc) => ({
                url: (doc.metadata as Record<string, unknown>)?.["sourceUrl"] ?? url,
                title: doc.metadata?.title ?? null,
                content: doc.content ?? "",
                content_length: (doc.content ?? "").length,
                mime_type: doc.mimeType,
              })),
            }, null, 2),
          }],
        };
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        return { content: [{ type: "text" as const, text: `scrape_url failed: ${msg}` }], isError: true };
      }
    }
  );
}

"use client";
import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { getHistoryEntry } from "@/lib/ingest-history.js";
import { DocumentViewer } from "@/components/DocumentViewer.js";
import type { IngestHistoryEntry } from "@/lib/types.js";

export default function DocumentPage() {
  const { collection, id } = useParams<{ collection: string; id: string }>();
  const [entry, setEntry] = useState<IngestHistoryEntry | null>(null);

  useEffect(() => {
    void getHistoryEntry(collection, id).then(setEntry);
  }, [collection, id]);

  if (!entry) return <main className="p-6">Loading…</main>;
  return (
    <main className="p-6">
      <DocumentViewer entry={entry} />
    </main>
  );
}

"use client";
import { createContext, useContext, useState, type ReactNode } from "react";
import { cn } from "@/lib/utils.js";

const DialogCtx = createContext<{ open: boolean; setOpen: (v: boolean) => void } | null>(null);

export function Dialog({ children }: { children: ReactNode }) {
  const [open, setOpen] = useState(false);
  return <DialogCtx.Provider value={{ open, setOpen }}>{children}</DialogCtx.Provider>;
}
export function DialogTrigger({ children }: { children: ReactNode; asChild?: boolean }) {
  const ctx = useContext(DialogCtx)!;
  return <span onClick={() => ctx.setOpen(true)}>{children}</span>;
}
export function DialogContent({ className, children }: { className?: string; children: ReactNode }) {
  const ctx = useContext(DialogCtx)!;
  if (!ctx.open) return null;
  return (
    <div role="dialog" className={cn("fixed inset-0 z-50 flex items-center justify-center bg-black/40")}>
      <div className={cn("rounded-lg bg-white p-6 shadow-lg", className)}>{children}</div>
    </div>
  );
}
export function DialogHeader({ children }: { children: ReactNode }) {
  return <div className="mb-4">{children}</div>;
}
export function DialogTitle({ children }: { children: ReactNode }) {
  return <h2 className="text-lg font-semibold">{children}</h2>;
}
export function DialogFooter({ children }: { children: ReactNode }) {
  return <div className="mt-4 flex justify-end gap-2">{children}</div>;
}
export function DialogClose({ children }: { children: ReactNode; asChild?: boolean }) {
  const ctx = useContext(DialogCtx)!;
  return <span onClick={() => ctx.setOpen(false)}>{children}</span>;
}

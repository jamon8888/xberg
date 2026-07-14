export default async function init(): Promise<void> {}

export class XbergEngine {
  constructor(_opts: unknown, _injections: unknown) {}
  async ocr(_bytes: Uint8Array, _opts: unknown): Promise<unknown> {
    throw new Error("Not mocked");
  }
}

// `@xberg-io/xberg-wasm`'s pkg/nodejs output is a wasm-pack "nodejs" target
// (CommonJS, `require()`-based) being loaded into a browser Web Worker via
// webpack -- a target mismatch (see next.config.js). Its generated glue does
// `require("env")` for a WASM import module literally named "env" (wasm32
// binaries from certain C-interop/WASI-adjacent dependencies commonly use
// this generic import namespace). Node.js has no built-in "env" module; this
// stub lets webpack resolve the require() at bundle time. If the compiled
// code path actually calls one of these imports at runtime, WebAssembly
// instantiation will throw a LinkError naming the missing function --
// that's the signal a real implementation is needed here, not a silent
// wrong-answer.
module.exports = new Proxy(
  {},
  {
    get(_target, prop) {
      return () => {
        throw new Error(`xberg-wasm: unimplemented "env" import: ${String(prop)}`);
      };
    },
  }
);

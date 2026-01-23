// Deno entry point for hwpjs
// GitHub: ohsory1324/hwpjs branch: deno-dist

import {
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  getDefaultContext as __emnapiGetDefaultContext,
  WASI as __WASI,
} from "npm:@napi-rs/wasm-runtime";

const __wasi = new __WASI({
  version: "preview1",
});

// Fetch WASM from GitHub raw URL
const __wasmUrl =
  "https://raw.githubusercontent.com/ohsory1324/hwpjs/deno-dist/packages/hwpjs/npm/wasm32-wasi/hwpjs.wasm32-wasi.wasm";

const __emnapiContext = __emnapiGetDefaultContext();

const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
});

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer());

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 0, // Disable workers for Deno compatibility
  wasi: __wasi,
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    };
    return importObject;
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith("__napi_register__")) {
        instance.exports[name]();
      }
    }
  },
});

export default __napiModule.exports;
export const fileHeader = __napiModule.exports.fileHeader;
export const toHtml = __napiModule.exports.toHtml;
export const toJson = __napiModule.exports.toJson;
export const toMarkdown = __napiModule.exports.toMarkdown;

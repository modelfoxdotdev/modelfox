import { setNative } from "./common.ts"

export * from "./common.ts"

setNative(await import("./modelfox_wasm.js"))

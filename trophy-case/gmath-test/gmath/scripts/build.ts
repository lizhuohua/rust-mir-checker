import { encode } from "https://deno.land/std@0.89.0/encoding/base64.ts";

const name = "gmath";

await Deno.run({
  cmd: ["cargo", "build", "--release", "--target", "wasm32-unknown-unknown"],
}).status();

const wasm = await Deno.readFile(
  `./target/wasm32-unknown-unknown/release/${name}.wasm`,
);
const encoded = encode(wasm);
const js = `// deno-fmt-ignore-file\n// deno-lint-ignore-file
import { decode } from "https://deno.land/std@0.89.0/encoding/base64.ts";
export const source = decode("${encoded}");`;

await Deno.writeTextFile("wasm/wasm.js", js);

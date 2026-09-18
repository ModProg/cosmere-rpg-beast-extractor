import type { UserConfig } from 'vite'
import rustWasmLoader from "rust-wasmpack-loader";
export default {
  plugins: [
    {
      name: "include-obsidian-statblock-files",
      async generateBundle() {
        for (let file of ["Style.css", "Layout.json"]) {
          this.emitFile({
            type: "asset",
            fileName: `obsidian-fantasy-statblocks/${file}`,
            source: await this.fs.readFile(`../obsidian-fantasy-statblocks/${file}`)
          })
        }
      },

    },
    rustWasmLoader.rollup({ types: true, logLevel: "info" })
  ]
} satisfies UserConfig;

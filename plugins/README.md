# Generate Plugins

Any language that supports WebAssembly output should work.

## plugins directory structure:
Each plugin has its own directory that contains `manifest.json` and `<plugin>.wasm`

## manifest.json structure:

```json
{
  "manifest_version": "1.0",
  "version": "1.0",
  "name": "toc",
  "tag": "toc",
  "wasm_path": "toc.wasm",
  "func_name": "toc"
}
```

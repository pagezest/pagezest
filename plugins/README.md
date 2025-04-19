# Generate Plugins

## Step-1 : Make sure you have AssemblyScript Installed.

```
npm i assemblyscript
```

## Step-2 : Write TS Plugin file

Write your core plugin code in assemblyscript (typescript .ts) file for example `toc.ts`

## Step-3 : Compile the plugin with assemblyscript

```
npx asc toc.ts --target release --exportRuntime --exportTable --outFile toc.wasm
```

Load and test your .wasm file.


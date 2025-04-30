const fs = require("fs");
const path = require("path");

// Load the WASM binary
const wasmBuffer = fs.readFileSync(path.join(__dirname, "toc.wasm"));

(async () => {
  const wasm = await WebAssembly.instantiate(wasmBuffer, {
    env: {
      'console.log': function() {
      },
      console: {
        log: function() {
        },
      }
    }
  });

  // Get exports
  const { memory, toc } = wasm.instance.exports;

  // Allocate input in memory
  const inputJSON = {"attributes":{},"root":[{"depth":1,"raw":"# This is heading1\n","text":"This is heading1","tokens":[{"escaped":false,"raw":"This is heading1","text":"This is heading1","type":"text"}],"type":"heading"},{"raw":"Contents of Line1\n","text":"Contents of Line1","tokens":[{"escaped":false,"raw":"Contents of Line1","text":"Contents of Line1","type":"text"}],"type":"paragraph"},{"depth":2,"raw":"## This is Heading2\n","text":"This is Heading2","tokens":[{"escaped":false,"raw":"This is Heading2","text":"This is Heading2","type":"text"}],"type":"heading"},{"raw":"Contents of Line2","text":"Contents of Line2","tokens":[{"escaped":false,"raw":"Contents of Line2","text":"Contents of Line2","type":"text"}],"type":"paragraph"},{"raw":"\n\n","type":"space"},{"raw":"[[toc]]\n[[/toc]]","text":"[[toc]]\n[[/toc]]","tokens":[{"escaped":false,"raw":"[[toc]]\n[[/toc]]","text":"[[toc]]\n[[/toc]]","type":"text"}],"type":"paragraph"}],"tag":"toc"};
  const inputStr = JSON.stringify(inputJSON);
  const inputBytes = new TextEncoder().encode(inputStr);

  // Write to WASM memory (starts at offset 0)
  const mem = new Uint8Array(memory.buffer);
  mem.set(inputBytes, 0);

  // Call function: pointer = 0, length = inputBytes.length
  const resultPtr = toc(0, inputBytes.length);

  // Read null-terminated string from resultPtr
  let end = resultPtr;
  while (mem[end] !== 0) end++;

  const resultStr = new TextDecoder().decode(mem.slice(resultPtr, end));
  console.log("Parsed value: result", resultStr);
})();

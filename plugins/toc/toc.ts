import { JSON } from "json-as";

// Add this to your `entry.ts`
@external("env", "abort")
declare function abort(msg: usize, file: usize, line: u32, col: u32): void;

// override abort to do nothing
function abort(_: usize, __: usize, ___: u32, ____: u32): void {}

export function malloc(sz: usize): usize {
  return heap.alloc(sz);
}

export function toc(ptr: usize, len: i32): usize {
  //return ptr;
  const md = String.UTF8.decodeUnsafe(ptr, len, true);
  const obj = JSON.parse<JSON.Obj>(md);
  let tag: string = '???';
  if(obj.has('tag')) {
    tag = obj.get('tag')!.toString();
  }
  if(obj.has('root')) {
    const blocks = obj.get('root')!.get<JSON.Value[]>();
    console.log(`children length: ${blocks.length}`);
    for(let i=0; i<blocks.length; i++) {
      const block = blocks[i];
      const s = blocks[i].get<string>();
      console.log(`${s.length}`);
      //console.log(JSON.stringify(block));
      //const blockStr = JSON.stringify<JSON.Obj>(block);
      //console.log(`blockStr: ${blockStr}`);
      //const type = block.get('type')!.toString();
      console.log('block');
    }
  }
  console.log(`tag: ${tag}`);
  const toc = `TOC [tag=${tag}]`;

  const toc_str = String.UTF8.encode(toc, true);
  return changetype<usize>(toc_str);
  return ptr;
  //const obj = new Input("tag!");
  //const obj = new JSON.Obj();
  //obj.set('tag', 'tag__');
  const toc = "hello";
  return changetype<usize>(toc_str);
}

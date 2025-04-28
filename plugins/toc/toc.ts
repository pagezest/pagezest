import { JSON } from "json-as";

@json
class Input {
    x: i32;
}

export function malloc(sz: usize): usize {
    return heap.alloc(sz);
}

export function toc(ptr: usize, len: i32): usize {
    //console.log('start');
    //const md = "md";
    // Converting the raw string.
    const md = String.UTF8.decodeUnsafe(ptr, len, true);
    //const obj = JSON.parse<JSON.Obj>(md);
    //
    //const obj = JSON.parse<JSON.Obj>('{"toc": true}');
    //const obj = JSON.parse<Input>('{"x": 10}');

    //const toc = `###TOC plugin, ptr: ${ptr}, len: ${len}####`;
    const toc = "X" + md;
    //const toc = JSON.stringify(obj);
    const toc_str = String.UTF8.encode(toc, true);
    //const strBuff = heap.alloc(32);
    //return strBuff;
    return changetype<usize>(toc_str);
    //const numBytes = String.UTF8.encodeUnsafe(toc, 
    //const output_ptr = heap.alloc(4);

    //return 0;
    //return output_ptr;
}

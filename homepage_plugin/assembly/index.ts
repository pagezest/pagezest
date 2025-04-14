import { JSON } from "json-as";

// @ts-ignore: decorator
@external("env", "abort")
declare function abort(
  message?: i32,
  fileName?: i32,
  lineNumber?: i32,
  columnNumber?: i32
): void;

function allocateString(str: string): i32 {
  const length = str.length;
  const ptr = heap.alloc(length + 4);
  store<i32>(ptr, str.length);
  for (let i = 0; i < length; i++) {
    store<u8>(ptr + i + 4, str.charCodeAt(i));
  }
  return ptr as i32;
}

export function malloc(size: i32): i32 {
  return heap.alloc(size) as i32;
}

export function homepage(sz: i32, input_ptr: i32): i32 {
  const input = String.UTF8.decodeUnsafe(input_ptr as usize, sz as usize);
  let output = "<h1>AssemblyScript Homepage Plugin</h1><ul>";
  let arr: Array<Post> = JSON.parse<Array<Post>>(input);
  for(let i=0; i<arr.length; i++) {
    const elm = arr[i];
    output += `<li><a href="/${elm.slug}">${elm.title}</a></li>`;
  }
  output += "</ul>";
  return allocateString(output);
//  let s = "";
//  //if(process.argv.length > 0) s = process.argv.at(0);
  return arr.length;
  //return [sz + 99, input_ptr + 99];

  // Example: build a response
  let count = arr.length;
  let result = "Received " + count.toString() + " items";
  console.log(result);

  // Return pointer to result string
  //return String.UTF8.encode(result, true); // true = null-terminated
  return 42;
}

export function add(a: i32, b: i32): i32 {
  return a + b;
}

@json
class Post {
  title: string;
  content: string;
  author: string;
  slug: string;
  created_at: string;
}

/*
*/

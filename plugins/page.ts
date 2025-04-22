// index.ts
import { JSON } from "assemblyscript-json/assembly";

export function title(ptr: usize, len: i32): usize {
  const title = String.UTF8.decodeUnsafe(ptr, len, true);
  const html = `<h1>${title}</h1><hr>`;
  const html_str = String.UTF8.encode(html, true);
  return changetype<usize>(html_str);
}

export function toc(ptr: usize, len: i32): usize {
  // Converting the raw string.
  const md_json_str = String.UTF8.decodeUnsafe(ptr, len, true);
  const md_json = JSON.parse(md_json_str);

  if (!md_json || !md_json.isObj) {
    return changetype<usize>(
      String.UTF8.encode(`{"html": "", "success": false}`, true)
    );
  }

  const obj = md_json as JSON.Obj;
  const tocLines = new Array<string>();

  const children = obj.get("children");
  if (!children || !children.isArr) {
    return changetype<usize>(
      String.UTF8.encode(`{"html": "", "success": false}`, true)
    );
  }

  const childrenArray = children as JSON.Arr;
  for (let i = 0; i < childrenArray.valueOf().length; i++) {
    const childValue = childrenArray.valueOf()[i];

    if (childValue && childValue.isObj) {
      const child = childValue as JSON.Obj;
      const typeValue = child.get("type");

      if (
        typeValue &&
        typeValue.isString &&
        typeValue.toString() === "heading"
      ) {
        const depthValue = child.get("depth");
        const textValue = child.get("text");

        if (
          depthValue &&
          depthValue.isInteger &&
          textValue &&
          textValue.isString
        ) {
          // Fix: properly handle integer values from JSON
          const depth = depthValue.isInteger
            ? (depthValue as JSON.Integer).valueOf()
            : 1;
          const text = textValue.toString();

          // Fix: ensure depth is used as a string in template literals
          const depthStr = depth.toString();
          tocLines.push(`<h${depthStr}>${text}</h${depthStr}>`);
        }
      }
    }
  }

  const toc_html = tocLines.join(`\\n`);
  const result = `{"html": "${toc_html}", "success": true}`;
  const toc_str = String.UTF8.encode(result, true);
  return changetype<usize>(toc_str);
}

export function toc(ptr: usize, len: i32): usize {
    // Converting the raw string.
    const md = String.UTF8.decodeUnsafe(ptr, len, true);
    const lines = md.split("\n");
    const tocLines = new Array<string>();

    for(let i=0; i< lines.length; i++){
        const line = lines[i];

        if (line.length==0 || line[0]!='#')
            continue;
        
        let heading_level = 0;
        while(heading_level < line.length && line[heading_level] == '#')
            heading_level++;

        const title = line.substring(heading_level).trim()
        let indent = "";
        for(let j=1; j<heading_level; j++){
            indent += "\t";
        }
        tocLines.push(`${indent}${title}`);
    }

    const toc_str = String.UTF8.encode(tocLines.join("\n"), true);
    return changetype<usize>(toc_str);
}
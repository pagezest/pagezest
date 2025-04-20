export function title(ptr: usize, len: i32): usize {
    const title = String.UTF8.decodeUnsafe(ptr, len, true);
    const html = `<h1>${title}</h1><hr>`;
    const html_str = String.UTF8.encode(html, true);
    return changetype<usize>(html_str);
}

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

        if (heading_level>6) continue;
        
        const title = line.substring(heading_level).trim()
        tocLines.push(`<h${heading_level}>${title}</h${heading_level}>`);
    }

    const toc_str = String.UTF8.encode(tocLines.join("\n"), true);
    return changetype<usize>(toc_str);
}

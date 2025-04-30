const std = @import("std");
extern "console.log" fn log(message: [*:0]const u8) void;

var return_buffer: [65536]u8 = undefined;

fn append_to_str(src: []u8, offset: usize, s: []const u8) usize {
    std.mem.copyForwards(u8, src[offset..s.len], s);
    return offset + s.len;
}

export fn toc(ptr: [*]const u8, len: usize) [*:0]const u8 {
    const input = ptr[0..len];

    var fba = std.heap.FixedBufferAllocator.init(&return_buffer);
    const allocator = fba.allocator();
    const parsed = std.json.parseFromSlice(std.json.Value, allocator, input, .{}) catch {
        return "parse error";
    };

    const inputObj = parsed.value;

    if (inputObj != .object)
        return "not an object";

    const obj = inputObj.object;
    const root = obj.get("root") orelse return "no root";

    if (root != .array)
        return "not a array";

    const children = root.array;
    var offset: u32 = 0;

    offset = append_to_str(&return_buffer, offset, "<h1>TOC</h1>");
    offset = append_to_str(&return_buffer, offset, "<ul>");
    for (children.items) |c| {
        const typ = c.object.get("type").?.string;
        const depth = c.object.get("depth").?.integer;
        if (std.mem.eql(u8, typ, "heading") and depth >= 0) {
            const text = c.object.get("text").?.string;
            offset = append_to_str(&return_buffer, offset, "<li>");
            offset = append_to_str(&return_buffer, offset, text);
            offset = append_to_str(&return_buffer, offset, "</li>");
        }
    }
    offset = append_to_str(&return_buffer, offset, "</ul>");
    offset = append_to_str(&return_buffer, offset, "<hr/>");

    return_buffer[offset] = 0;

    return return_buffer[0..offset :0];
}

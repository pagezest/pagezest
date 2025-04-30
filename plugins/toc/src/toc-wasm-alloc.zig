const std = @import("std");
extern "console.log" fn log(message: [*]const u8) void;

//var return_buffer: [16384]u8 = undefined;

// fn append_to_str(src: []u8, offset: usize, s: []const u8) usize {
//     std.mem.copyForwards(u8, src[offset..s.len], s);
//     return offset + s.len;
// }

export fn toc(ptr: [*]const u8, len: usize) [*:0]const u8 {
    const input = ptr[0..len];

    const allocator = std.heap.wasm_allocator;

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

    var output = std.ArrayList(u8).init(allocator);
    var writer = output.writer();
    _ = writer.print("<ul>", .{}) catch null;
    for (children.items) |c| {
        const typ = c.object.get("type").?.string;
        const depth = c.object.get("depth").?.integer;
        _ = writer.print("<li style='margin-left: {d}px'>", .{depth}) catch null;
        _ = writer.print("{s}", .{typ}) catch null;
        _ = writer.print("</li>", .{}) catch null;
    }
    _ = writer.print("<ul>", .{}) catch null;

    _ = writer.writeByte(0) catch null;

    const slice = output.toOwnedSlice() catch "err";
    const o_ptr: [*:0]const u8 = slice;
    return o_ptr;
}

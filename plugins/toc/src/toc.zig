const std = @import("std");
extern "console.log" fn log(message: [*]const u8) void;

export fn toc(ptr: [*]const u8, len: usize) usize {
    const input = ptr[0..len];

    const allocator = std.heap.wasm_allocator;

    const parsed = std.json.parseFromSlice(std.json.Value, allocator, input, .{}) catch {
        return @intFromPtr("parse error".ptr);
    };

    const inputObj = parsed.value;

    if (inputObj != .object)
        return @intFromPtr("not an object".ptr);

    const obj = inputObj.object;
    const root = obj.get("root") orelse return @intFromPtr("no root".ptr);

    if (root != .array)
        return @intFromPtr("not a array");

    const children = root.array;

    var output = std.ArrayList(u8).init(allocator);
    var writer = output.writer();
    _ = writer.print("<h1>TOC</h1>", .{}) catch null;
    _ = writer.print("<ul>", .{}) catch null;
    for (children.items) |c| {
        const typ = c.object.get("type").?.string;
        const depth = c.object.get("depth").?.integer;
        if (depth == 1) {
            _ = writer.print("<li style='margin-left: {d}px'>", .{depth}) catch null;
            _ = writer.print("{s}", .{typ}) catch null;
            _ = writer.print("</li>", .{}) catch null;
        }
    }
    _ = writer.print("</ul>", .{}) catch null;

    _ = writer.writeByte(0) catch null;

    const slice = output.toOwnedSlice() catch "err";
    return @intFromPtr(slice.ptr);
}

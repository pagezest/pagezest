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
    const title_val = obj.get("content") orelse std.json.Value{ .string = "" };
    const title_trimmed = std.mem.trim(u8, title_val.string, " \t\n\r");
    const title = if (title_trimmed.len == 0) "Table of contents" else title_trimmed;

    if (root != .array)
        return @intFromPtr("not a array");

    const children = root.array;

    var output = std.ArrayList(u8).init(allocator);
    var writer = output.writer();
    _ = writer.print("<h1>{s}</h1>", .{title}) catch null;
    _ = writer.print("<ul>", .{}) catch null;
    for (children.items) |c| {
        const typ = c.object.get("type").?.string;
        const depth = c.object.get("depth").?.integer;
        if (std.mem.eql(u8, typ, "heading") and depth == 1) {
            const text = c.object.get("text").?.string;
            _ = writer.print("<li>", .{}) catch null;
            _ = writer.print("{s}", .{text}) catch null;
            _ = writer.print("</li>", .{}) catch null;
        }
    }
    _ = writer.print("</ul>", .{}) catch null;

    _ = writer.writeByte(0) catch null;

    const slice = output.toOwnedSlice() catch "err";
    return @intFromPtr(slice.ptr);
}

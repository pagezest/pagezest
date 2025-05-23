const std = @import("std");

pub fn build(b: *std.Build) void {
    const wasm_target = b.resolveTargetQuery(.{
        .cpu_arch = .wasm32,
        .os_tag = .wasi,
        .cpu_features_add = std.Target.wasm.featureSet(&.{
            .atomics,
            .bulk_memory,
            // .extended_const, not supported by Safari
            .multivalue,
            .mutable_globals,
            .nontrapping_fptoint,
            .reference_types,
            //.relaxed_simd, not supported by Firefox or Safari
            .sign_ext,
            // observed to cause Error occured during wast conversion :
            // Unknown operator: 0xfd058 in Firefox 117
            //.simd128,
            // .tail_call, not supported by Safari
        }),
    });

    const wasm = b.addExecutable(.{
        .name = "toc",
        .root_source_file = b.path("src/toc.zig"),
        .target = wasm_target,
        .optimize = .ReleaseSmall,
        .strip = true,
    });
    wasm.entry = .disabled;
    wasm.rdynamic = true;

    b.installArtifact(wasm);
}

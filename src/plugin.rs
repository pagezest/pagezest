use std::error::Error;
use wasmi::{Caller, Config, Engine, Func, Linker, Module, Store, TypedFunc};

pub fn call_wasm(
    wasm_file_path: &str,
    input: &str,
    plugin_func: &str,
) -> Result<String, Box<dyn Error>> {
    let wasm = std::fs::read(wasm_file_path)?;
    let engine = Engine::new(&Config::default());
    let module = Module::new(&engine, &wasm)?;

    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    let abort_func = Func::wrap(&mut store, abort_stub);
    let console_log_func = Func::wrap(&mut store, console_log_stub);
    linker.define("env", "abort", abort_func)?;
    linker.define("env", "console.log", console_log_func)?;

    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

    let memory = instance
        .get_export(&store, "memory")
        .and_then(|ext| ext.into_memory())
        .unwrap();

    // Loading greet function and passing input as string and receiving output as string.
    let plugin_func: TypedFunc<(u32, u32), u32> =
        instance.get_typed_func(&store, plugin_func)?;

    let input_buffer = input.as_bytes();
    let offset = 40_000u32;
    memory
        .write(&mut store, offset as usize, input_buffer)
        .unwrap();

    let res_ptr = plugin_func
        .call(&mut store, (offset, input.len() as u32))
        .unwrap();
    let mut output = Vec::new();
    let mut curr_ptr = res_ptr;
    loop {
        let mut buf = [0u8, 1];
        memory
            .read(&mut store, curr_ptr as usize, &mut buf)
            .unwrap();
        if buf[0] == 0 {
            break;
        }
        output.push(buf[0]);
        curr_ptr += 1;
    }

    let output_str = String::from_utf8(output).unwrap();
    Ok(output_str)
}

fn abort_stub(_caller: Caller<'_, ()>, _msg_ptr: i32, _file_ptr: i32, _line: i32, _col: i32) {}
fn console_log_stub(_caller: Caller<'_, ()>, _msg_ptr: i32) {
    println!("console.log called");
}

//use std::fmt::Write;
use serde_json::Value;
use eyre::Result;
use wamr_rust_sdk::{
    function::Function, instance::Instance, module::Module as WamrModule, runtime::RuntimeBuilder, value::WasmValue
};
use wasmi::{core::ValType, AsContext, Caller, Engine, Func, Linker, Memory, Module, Store, WasmTyList};

use std::path::PathBuf;

use crate::errors::AppError;


pub fn run_plugin(filename: &str, input: String) -> Result<String, AppError> {
    return run_plugin_wasmi(filename, input);
}

pub fn read_string(store: Store<u32>, memory: Memory, ptr: i32) -> String {
    let mut buffer: &mut [u8] = &mut [0u8; 4];
    memory.read(&store, ptr as usize, buffer);
    let sz = i32::from_le_bytes(buffer.try_into().unwrap());
    let mut buffer: Vec<u8> = vec![0u8; sz as usize];
    let buffer_slice: &mut [u8] = &mut buffer;
    memory.read(&store, (ptr + 4) as usize, buffer_slice);

    return String::from_utf8_lossy(buffer_slice).to_string();
}

pub fn run_plugin_wasmi(filename: &str, input: String) -> Result<String, AppError> {
    let input_sz = input.len();
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push(filename);
    let wasm_bytes = std::fs::read(d.as_path()).expect("read");

    let engine = Engine::default();
    let module = Module::new(&engine, wasm_bytes).expect("module");

    type HostState = u32;
    let mut store = Store::new(&engine, 42);
    let mut linker = <Linker<HostState>>::new(&engine);

    linker.func_wrap("host", "homepage", |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {param} from WebAssembly and my host state is: {}", caller.data());
    }).expect("func_wrap");

    linker
        .define(
            "env",
            "abort",
            Func::wrap(&mut store, |_0: i32, _1: i32, _2: i32, _3: i32| -> () {
                unimplemented!()
            }),
        )
        .unwrap();
    linker
        .define(
            "env",
            "console.log",
            Func::wrap(&mut store, |arg: i32| -> () {
                //let memory: &[u32] = unsafe { std::slice::from_raw_parts(store.data().to_owned() as *const u32, 4) };
                //println!("console.log, {}", arg);
                println!("")
            }),
        )
        .unwrap();
    linker
        .define(
            "env",
            "memory",
            wasmi::Memory::new(&mut store, wasmi::MemoryType::new(2, Some(16)).unwrap())
            .unwrap(),
        )
        .unwrap();



    let instance = linker
        .instantiate(&mut store, &module).expect("init")
        .start(&mut store).expect("start");


    let malloc_func = instance.get_typed_func::<i32, i32>(&store, "malloc")
        .expect("get_typed_func");
    println!("input size: {}", input_sz);
    let input_ptr = malloc_func.call(&mut store, input_sz as i32, ).unwrap();
    println!("input buffer: {}", input_ptr);
    let memory = instance.get_memory(&store, "memory").unwrap();
    memory.write(&mut store, input_ptr as usize, input.as_bytes());
    let func_ref = instance.get_typed_func::<(i32, i32), i32>(&store, "homepage")
        .expect("get_typed_func");
    let res = func_ref.call(&mut store, (input_sz as i32, input_ptr as i32)).expect("call");
    let output = read_string(store, memory, res);
    println!("sz: {}", output);




    //func_ref.call(&mut store, (input.len() as i32, input.as_bytes().as_ptr())).expect("call");
    println!("result: {}", res);

    Ok(output.into())
}

#[allow(dead_code)]
pub fn run_plugin_wamr(filename: &str, input: String) -> Result<String, AppError> {
    //let runtime = Runtime::new().expect("1");
    let memory_pool: Vec<u8> = vec![0; 64 * 1024];
    let runtime = RuntimeBuilder::default()
        .use_memory_pool(memory_pool)
        .build().expect("1");
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push(filename);

    let module = WamrModule::from_file(&runtime, d.as_path()).expect("2");
    //let instance = Instance::new(&runtime, &module, 1024 * 64).expect("3");
    let instance = Instance::new_with_args(&runtime, &module, 1024 * 64, 1204 * 64).expect("3");
    let function = Function::find_export_func(&instance, "homepage").expect("4");

    let input_bytes = input.into_bytes();
    let input_sz = input_bytes.len() as i32;
    let input_ptr = input_bytes.as_ptr() as i32;


    let params: Vec<WasmValue> = vec![WasmValue::I32(input_sz), WasmValue::I32(input_ptr)];
    let fn_result = function.call(&instance, &params).expect("5");
    let result_value = fn_result.get(0).unwrap();
    let result = match result_value  {
        WasmValue::I32(val) => val,
        _ => panic!("invalid value"),
    };

    println!("result: {}", result);
    Ok("OK...".into())
}

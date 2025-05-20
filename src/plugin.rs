use std::error::Error;
use wasmtime::{Caller, Config, Engine, Instance, Linker, Memory, Module, Store, TypedFunc};

#[allow(unused)]
pub struct Plugin {
    pub name: String,
    pub tag: String,
    pub wasm_file_path: String,
    pub plugin_func_name: String,
    pub dynamic: bool,
    instance: Instance,
    memory: Memory,
    store: Store<()>,
    plugin_func: TypedFunc<(u32, u32), u32>,
}

impl Plugin {
    pub fn new(
        name: &str,
        tag: &str,
        wasm_path: &str,
        plugin_func_name: &str,
        dynamic: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let wasm = std::fs::read(wasm_path)?;
        let mut config = Config::new();
        config.async_support(false);
        let engine = Engine::new(&config)?;
        let module = Module::from_binary(&engine, &wasm)?;

        let mut store = Store::new(&engine, ());
        let mut linker = Linker::new(&engine);

        linker.func_wrap("env", "abort", abort_stub)?;
        linker.func_wrap("env", "console.log", console_log_stub)?;

        let instance = linker.instantiate(&mut store, &module)?;

        let memory = instance
            .get_export(&mut store, "memory")
            .and_then(|e| e.into_memory())
            .ok_or("memory export not found")?;

        let plugin_func =
            instance.get_typed_func::<(u32, u32), u32>(&mut store, plugin_func_name)?;

        Ok(Self {
            name: name.to_string(),
            tag: tag.to_string(),
            wasm_file_path: wasm_path.to_string(),
            plugin_func_name: plugin_func_name.to_string(),
            instance,
            memory,
            store,
            plugin_func,
            dynamic,
        })
    }

    pub fn call(&mut self, input: &Vec<u8>) -> Result<String, Box<dyn Error>> {
        let offset = 40_000u32;
        self.memory.write(&mut self.store, offset as usize, input)?;

        let res_ptr = self
            .plugin_func
            .call(&mut self.store, (offset, input.len() as u32))?;

        let mut output = Vec::new();
        let mut curr_ptr = res_ptr;
        loop {
            let mut buf = [0u8; 1];
            self.memory
                .read(&mut self.store, curr_ptr as usize, &mut buf)?;
            if buf[0] == 0 {
                break;
            }
            output.push(buf[0]);
            curr_ptr += 1;
        }

        Ok(String::from_utf8(output)?)
    }
}

impl std::fmt::Debug for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Plugin")
            .field("name", &self.name)
            .field("tag", &self.tag)
            .field("wasm_file_path", &self.wasm_file_path)
            .field("plugin_func_name", &self.plugin_func_name)
            .finish()
    }
}

fn abort_stub(_caller: Caller<'_, ()>, _msg_ptr: i32, _file_ptr: i32, _line: i32, _col: i32) {
    println!(
        "abort: msg: {}, file: {}, line: {}, col: {}",
        _msg_ptr, _file_ptr, _line, _col
    );
}

fn console_log_stub(_caller: Caller<'_, ()>, _msg_ptr: i32) {
    println!("console.log called");
}

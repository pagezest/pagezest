use std::{collections::HashMap, env, error::Error, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use serde_json::Value;
use wasmi::{AsContextMut, Caller, Config, Engine, Func, Instance, Linker, Memory, Module, Store, TypedFunc};

use crate::{errors::AppError, plugin::call_wasm};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Plugin {
    name: String,
    tag: String,
    wasm_file_path: String,
    plugin_func_name: String,
    instance: Instance,
    memory: Memory,
    store: Store<()>,
    plugin_func: TypedFunc<(u32, u32), u32>,
}

#[derive(Debug)]
pub struct PluginManager {
    plugins: HashMap<String, Arc<Mutex<Plugin>>>
}

impl PluginManager {
   pub fn new() -> Self {
        Self { plugins: HashMap::new() }
   }

   pub fn load_plugin(&mut self, plugin: Plugin) {
       //let plugin = Plugin::new(name, tag);
       self.plugins.insert(plugin.tag.to_string(), Arc::new(Mutex::new(plugin)));
   }

   pub fn unload_plugin(&mut self, tag: &str) {
       self.plugins.remove(tag);
   }

   pub fn list_plugins(self) {
       self.plugins.keys();
   }

   pub fn has_plugin_handler(&mut self, tag: &str) -> bool {
       self.plugins.contains_key(tag)
   }

   pub fn get_plugin_by_tag(&mut self, tag: &str) -> Result<&Arc<Mutex<Plugin>>, AppError> {
       match self.plugins.get(tag) {
           Some(plugin) => {
               Ok(plugin)
           },
           _ => {
               Err(AppError::PluginError("Not found".to_string()))
           }
       }

   }

   pub fn load_plugin_dir(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
       if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                match fs::read_to_string(manifest_path) {
                    Ok(manifest_content) => {
                        let json: Value = serde_json::from_str(&manifest_content).expect("could not parse manifest");
                        let name = json.get("name").and_then(|s| s.as_str()).expect("Manifest: name not found").to_string();
                        let tag = json.get("tag").and_then(|s| s.as_str()).expect("Manifest: tag not found {}").to_string();
                        let wasm_path = json.get("wasm_path").and_then(|s| s.as_str()).expect("Manifest: wasm_path not found").to_string();
                        let plugin_func_name = json.get("func_name").and_then(|s| s.as_str()).expect("Manifest: func_name not found").to_string();
                        let wasm_path = path.join(wasm_path);
                        let wasm_path_str = wasm_path.to_str().unwrap();
                        match Plugin::new(&name, &tag, wasm_path_str, &plugin_func_name) {
                            Ok(plugin) => {
                                self.load_plugin(plugin);
                                println!("plugin loaded: {}", path.to_str().unwrap());
                            },
                            Err(e) => {
                                return Err(e.into());
                            }
                        }
                    },
                    Err(e) => {
                        return Err(Box::new(e));
                    }
                }
            } else {
                println!("no manifest file in {}, skipping", path.to_str().unwrap());
            }
       }
       Ok(())
   }

   pub fn scan_plugins(&mut self) -> Result<(), AppError> {
       let plugins_path = env::var("CARGO_MANIFEST_DIR").unwrap_or("./".to_string());
       let path = Path::new(&plugins_path).join("plugins");
       if path.exists() && path.is_dir() {
           println!("plugins path: {}", path.to_str().unwrap());
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.filter_map(Result::ok) {
                    let entry_path = entry.path();
                    if ! entry_path.is_dir() {
                        continue;
                    }
                    match self.load_plugin_dir(entry_path) {
                        Ok(()) => {
                        },
                        Err(e) => {
                            println!("Error loading plugin: {}", e.to_string());
                        }
                    }
                }
            }
       } else {
           println!("no plugins")
       }
       println!("plugins loaded:");
       for (k, _) in self.plugins.clone() {
            println!("{}", k);
       }
       Ok(())
   }
}

impl Plugin {
    pub fn new(name: &str, tag: &str, wasm_path: &str, plugin_func_name: &str) -> Result<Self, Box<dyn Error>> {
        let wasm = std::fs::read(wasm_path)?;
        let engine = Engine::new(&Config::default());
        let module = Module::new(&engine, &wasm)?;

        let mut store = Store::new(&engine, ());
        let mut linker = Linker::new(&engine);
        let abort_func = Func::wrap(&mut store, abort_stub);
        linker.define("env", "abort", abort_func)?;
        let console_log_func = Func::wrap(&mut store, console_log_stub);
        linker.define("env", "console.log", console_log_func)?;
        let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

        let memory = instance
            .get_export(&store, "memory")
            .and_then(|ext| ext.into_memory()).unwrap();
        let plugin_func =
            instance.get_typed_func(&store, &plugin_func_name.to_string())?;
        Ok(
            Self {
                name: name.to_string(),
                tag: tag.to_string(),
                wasm_file_path: wasm_path.to_string(),
                plugin_func_name: plugin_func_name.to_string(),
                plugin_func,
                memory,
                instance,
                store,
            }
        )
    }



    pub fn call(
        &mut self, input: &Vec<u8>
    ) -> Result<String, Box<dyn Error>> {
        let offset = 40_000u32;
        self.memory
            .write(&mut self.store.as_context_mut(), offset as usize, input)
            .unwrap();
        let res_ptr = (&mut self.plugin_func)
            .call(&mut self.store, (offset, input.len() as u32))?;
        let mut output = Vec::new();
        let mut curr_ptr = res_ptr;
        loop {
            let mut buf = [0u8, 1];
            self.memory
                .read(&mut self.store, curr_ptr as usize, &mut buf)
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

    pub fn call_in_new_context(
        &mut self, input: &Vec<u8>
    ) -> Result<String, Box<dyn Error>> {
        println!("call_in_new_context: {}, {}, {}", self.wasm_file_path, "", self.plugin_func_name);
        call_wasm(&self.wasm_file_path, input, &self.plugin_func_name)
    }
}

fn abort_stub(_caller: Caller<'_, ()>, _msg_ptr: i32, _file_ptr: i32, _line: i32, _col: i32) {
    println!("abort: msg: {}, file: {}, line: {}, col: {}", _msg_ptr, _file_ptr, _line, _col);
}

fn console_log_stub(_caller: Caller<'_, ()>, _msg_ptr: i32) {
    println!("console.log called");
}

use crate::{errors::AppError, plugin::Plugin};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, env, error::Error, fs, path::{Path, PathBuf}, sync::{Arc, RwLock}
};

fn default_manifest_version() -> String {
    "1.0".to_string()
}
fn default_manifest_dynamic() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    #[serde(default = "default_manifest_version")]
    manifest_version: String,
    #[serde(default = "default_manifest_dynamic")]
    dynamic: bool,
    name: String,
    tag: String,
    wasm_path: String,
    func_name: String,
}

pub struct PluginManager {
    plugins: HashMap<String, Arc<RwLock<Plugin>>>,
    cache: RwLock<HashMap<String, String>>
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            cache: RwLock::new(HashMap::new())
        }
    }

    pub fn load_plugin(&mut self, plugin: Plugin) {
        //let plugin = Plugin::new(name, tag);
        self.plugins
            .insert(plugin.tag.to_string(), Arc::new(RwLock::new(plugin)));
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

    pub fn get_plugin_by_tag(&mut self, tag: &str) -> Result<&Arc<RwLock<Plugin>>, AppError> {
        match self.plugins.get(tag) {
            Some(plugin) => Ok(plugin),
            _ => Err(AppError::PluginError("Not found".to_string())),
        }
    }

    pub fn run_plugin(&mut self, tag: &str, slug: &str, input: &Vec<u8>) -> Result<String, Box<dyn Error>> {
        let plugin = self.plugins.get(tag).ok_or_else(|| format!("No plugin found for {}", tag))?;
        let mut plugin = plugin.write().map_err(|e| format!("could not lock plugin: {e}"))?;
        let key = format!("{}__{}", tag, slug);
        if !plugin.dynamic {
            let cache = self.cache.read().map_err(|e| format!("cloud not lock cache {e}"))?;
            if cache.contains_key(&key) {
                return Ok(cache.get(&key).unwrap().clone());
            }
        }

        let res = plugin.call(input)?;


        let mut cache_w = self.cache.write().map_err(|e| format!("could not lock cache for write {e}"))?;
        cache_w.insert(key, res.clone());
        Ok(res)
    }

    pub fn load_plugin_dir(&mut self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                match fs::read_to_string(manifest_path) {
                    Ok(manifest_content) => {
                        let manifest: PluginManifest = serde_json::from_str(&manifest_content)?;
                        println!("manifest: {:?}", manifest);
                        let wasm_path = path.join(manifest.wasm_path);
                        let wasm_path_str = wasm_path.to_str().unwrap();
                        match Plugin::new(
                            &manifest.name,
                            &manifest.tag,
                            wasm_path_str,
                            &manifest.func_name,
                            manifest.dynamic,
                        ) {
                            Ok(plugin) => {
                                self.load_plugin(plugin);
                                println!("plugin loaded: {}", path.to_str().unwrap());
                            }
                            Err(e) => {
                                return Err(e.into());
                            }
                        }
                    }
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
                    if !entry_path.is_dir() {
                        continue;
                    }
                    match self.load_plugin_dir(entry_path) {
                        Ok(()) => {}
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

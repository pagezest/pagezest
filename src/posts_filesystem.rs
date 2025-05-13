use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Write, Error as IoError};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::post::BlogPost;

#[derive(Debug)]
pub enum PostsFSError {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    NotFound(String),
}

impl From<std::io::Error> for PostsFSError {
    fn from(err: IoError) -> Self {
        PostsFSError::Io(err)
    }
}

impl From<serde_json::Error> for PostsFSError {
    fn from(err: serde_json::Error) -> Self {
        PostsFSError::SerdeJson(err)
    }
}

pub struct PostsFS {
    base_dir: PathBuf,
    index: RwLock<HashMap<String, String>>,
}

impl PostsFS {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, PostsFSError> {
        let dir = base_dir.as_ref();
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        Ok(Self { base_dir: dir.to_path_buf(), index: RwLock::new(HashMap::new()) })
    }

    pub fn default() -> Result<Self, PostsFSError> {
        let dir = Path::new("posts");
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        Ok(Self { base_dir: dir.to_path_buf(), index: RwLock::new(HashMap::new()) })
    }

    pub fn build_index(&self) -> Result<(), PostsFSError> {
        let entries = self.list()?;
        if let Ok(mut index) = self.index.write() {
            for e in entries {
                index.insert(e.slug, e.id);
            }
        }
        Ok(())
    }

    fn entry_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(id)
    }

    fn metadata_path(&self, id: &str) -> PathBuf {
        self.entry_path(id).join("metadata.json")
    }

    fn content_path(&self, id: &str) -> PathBuf {
        self.entry_path(id).join("content.bin")
    }

    pub fn list(&self) -> Result<Vec<BlogPost>, PostsFSError> {
        let mut entries = vec![];
        for entry in fs::read_dir(&self.base_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                if let Some(id) = path.file_name().and_then(|n| n.to_str()) {
                    if let Ok(meta) = self.get_metadata(id) {
                        entries.push(meta);
                    }
                }
            }
        }
        Ok(entries)
    }

    pub fn get(&self, id: &str) -> Result<(BlogPost, Vec<u8>), PostsFSError> {
        let metadata = self.get_metadata(id)?;
        let content = fs::read(self.content_path(id))?;
        Ok((metadata, content))
    }

    pub fn get_by_slug(&self, slug: &str) -> Result<Option<(BlogPost, Vec<u8>)>, PostsFSError> {
        if let Ok(index) = self.index.read() {
            if ! index.contains_key(slug) {
                return Ok(None);
            }
            let id = index.get(slug).expect("get_by_slug::Unknow error");
            let metadata = self.get_metadata(id)?;
            let content = fs::read(self.content_path(id))?;
            Ok(Some((metadata, content)))
        } else {
            Err(PostsFSError::NotFound("".to_string()))
        }
    }



    fn get_metadata(&self, id: &str) -> Result<BlogPost, PostsFSError> {
        let file = File::open(self.metadata_path(id))?;
        let metadata = serde_json::from_reader(file)?;
        Ok(metadata)
    }

    pub fn insert(&self, metadata: &BlogPost, content: &[u8]) -> Result<(), PostsFSError> {
        let path = self.entry_path(&metadata.id);
        fs::create_dir_all(&path)?;

        let meta_path = self.metadata_path(&metadata.id);
        let mut meta_file = File::create(meta_path)?;
        serde_json::to_writer_pretty(&mut meta_file, metadata)?;

        let mut content_file = File::create(self.content_path(&metadata.id))?;
        content_file.write_all(content)?;
        if let Ok(mut index) = self.index.write() {
            index.insert(metadata.slug.clone(), metadata.id.clone());
        }
        Ok(())
    }

    pub fn update_metadata(&self, id: &str, update: &BlogPost) -> Result<(), PostsFSError> {
        let path = self.entry_path(id);
        let mut update = update.clone();
        update.id = id.to_string();
        if path.exists() {
            let mut file = File::create(self.metadata_path(id))?;
            serde_json::to_writer_pretty(&mut file, &update)?;
            if let Ok(mut index) = self.index.write() {
                index.insert(update.slug.clone(), id.to_string());
            }
            Ok(())
        } else {
            Err(PostsFSError::NotFound(id.to_string()))
        }
    }

    pub fn update_content(&self, id: &str, content: &[u8]) -> Result<(), PostsFSError> {
        let path = self.entry_path(id);
        if path.exists() {
            let mut file = File::create(self.content_path(id))?;
            file.write_all(content)?;
            Ok(())
        } else {
            Err(PostsFSError::NotFound(id.to_string()))
        }
    }

    pub fn delete(&self, id: &str) -> Result<(), PostsFSError> {
        let path = self.entry_path(id);
        if path.exists() {
            fs::remove_dir_all(path)?;
            if let Ok(mut index) = self.index.write() {
                let key = index.iter().find(|(_, v)| *v == id).map(|(k, _)| k.clone());
                match key {
                    Some(key) => {
                        index.remove(&key);
                    },
                    _ => {}
                }
                Ok(())
            } else {
                Err(PostsFSError::NotFound(id.to_string()))
            }
        } else {
            Err(PostsFSError::NotFound(id.to_string()))
        }
    }

    pub fn filter<F>(&self, predicate: F) -> Result<Vec<BlogPost>, PostsFSError>
    where
        F: Fn(&BlogPost) -> bool,
    {
        let all = self.list()?;
        Ok(all.into_iter().filter(predicate).collect())
    }

}

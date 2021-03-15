use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::{Db, Depth, FilePath, FolderPath};

#[derive(Debug, Clone)]
pub struct MemoryDb<T>
where
    T: Clone,
{
    items: Rc<RwLock<HashMap<FilePath, Rc<T>>>>,
}

impl<T> MemoryDb<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self {
            items: Rc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with(items: Rc<RwLock<HashMap<FilePath, Rc<T>>>>) -> Self {
        Self { items }
    }

    pub async fn insert_item(&mut self, path: FilePath, item: T) -> Option<Rc<T>> {
        self.insert(path, Rc::new(item)).await
    }
}

#[async_trait(?Send)]
impl<T> Db<Rc<T>> for MemoryDb<T>
where
    T: Clone,
{
    async fn set(&mut self, path: FilePath, item: Option<Rc<T>>) -> Option<Rc<T>> {
        let mut db = self.items.write().unwrap();
        match item {
            Some(i) => db.insert(path, i),
            None => db.remove(&path),
        }
    }

    async fn get(&self, path: &FilePath) -> Option<Rc<T>> {
        self.items.read().unwrap().get(path).map(|rc| rc.clone())
    }

    type List = Vec<(FilePath, Rc<T>)>;
    async fn list(&self, depth: &Depth, sync_path: &FolderPath) -> Self::List {
        self.items
            .read()
            .unwrap()
            .iter()
            .filter(|(path, _)| match depth {
                Depth::Simple => sync_path == path.folder(),
                Depth::Recursive => sync_path.contains(path.folder()),
            })
            .map(|(path, item)| (path.clone(), item.clone()))
            .collect()
    }
}

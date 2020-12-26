use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use sync::Db;
use sync::{Depth, FilePath, FolderPath, SyncDb};

fn main() {
    // Create our databases.
    let local_items = RwLock::new(HashMap::new());
    let mut local = MemoryDb::with(&local_items);
    let remote_items = RwLock::new(HashMap::new());
    let mut remote = MemoryDb::with(&remote_items);
    let mut db = SyncDb::new(MemoryDb::with(&local_items), MemoryDb::with(&remote_items));

    // Insert some items.
    let folder_path: FolderPath = vec!["folder"].into();

    let path1 = FilePath::new(folder_path.clone(), "item1".to_string());
    let item1 = "store me";
    local.insert_item(path1.clone(), item1);

    let path2 = FilePath::new(folder_path, "item2".to_string());
    let item2 = "store me, too";
    remote.insert_item(path2.clone(), item2);

    // Synchronize the databases.
    let root_path: FolderPath = vec![].into();
    db.sync_folder(&Depth::Recursive, &root_path);

    // Look inside the databases.
    let folder_path = vec!["folder"].into();
    let mut stored_locally = local.list(&Depth::Simple, &folder_path);
    let mut stored_remotely = remote.list(&Depth::Simple, &folder_path);

    // Sort to ensure they are in the same order for comparison.
    let expected = vec![(path1, Rc::new(item1)), (path2, Rc::new(item2))];
    stored_locally.sort_by_key(|(path, _)| path.clone());
    stored_remotely.sort_by_key(|(path, _)| path.clone());
    assert_eq!(expected, stored_locally);
    assert_eq!(expected, stored_remotely);
}

struct MemoryDb<'a, T>
where
    T: Clone,
{
    items: &'a RwLock<HashMap<FilePath, Rc<T>>>,
}

impl<'a, T> MemoryDb<'a, T>
where
    T: Clone,
{
    pub fn with(items: &'a RwLock<HashMap<FilePath, Rc<T>>>) -> Self {
        Self { items }
    }

    pub fn insert_item(&mut self, path: FilePath, item: T) -> Option<Rc<T>> {
        self.insert(path, Rc::new(item))
    }
}

impl<'a, T> Db<Rc<T>> for MemoryDb<'a, T>
where
    T: Clone,
{
    fn set(&mut self, path: FilePath, item: Option<Rc<T>>) -> Option<Rc<T>> {
        let mut db = self.items.write().unwrap();
        match item {
            Some(i) => db.insert(path, i),
            None => db.remove(&path),
        }
    }

    fn get(&self, path: &FilePath) -> Option<Rc<T>> {
        self.items.read().unwrap().get(path).map(|rc| rc.clone())
    }

    type List = Vec<(FilePath, Rc<T>)>;
    fn list(&self, depth: &Depth, sync_path: &FolderPath) -> Self::List {
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

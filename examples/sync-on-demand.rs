use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use sync::Db;
use sync::{Path, SyncDb};

fn main() {
    // Create our databases.
    let local_items = RwLock::new(HashMap::new());
    let mut local = MemoryDb::with(&local_items);
    let remote_items = RwLock::new(HashMap::new());
    let mut remote = MemoryDb::with(&remote_items);
    let mut db = SyncDb::new(MemoryDb::with(&local_items), MemoryDb::with(&remote_items));

    // Insert some items.
    let path1: Path = vec!["folder", "item1"].into();
    let item1 = "store me";
    local.insert_item(path1.clone(), item1);

    let path2: Path = vec!["folder", "item2"].into();
    let item2 = "store me, too";
    remote.insert_item(path2.clone(), item2);

    // Synchronize the databases.
    db.sync(&vec![].into());

    // Look inside the databases.
    let folder_path = vec!["folder"].into();
    let mut stored_locally = local.list(&folder_path);
    let mut stored_remotely = remote.list(&folder_path);

    // Sort to ensure they are in the same order for comparison.
    stored_locally.sort_by_key(|(path, _)| path.clone());
    stored_remotely.sort_by_key(|(path, _)| path.clone());
    assert_eq!(stored_locally, stored_remotely);
}

struct MemoryDb<'a, T>
where
    T: Clone,
{
    items: &'a RwLock<HashMap<Path, Rc<T>>>,
}

impl<'a, T> MemoryDb<'a, T>
where
    T: Clone,
{
    pub fn with(items: &'a RwLock<HashMap<Path, Rc<T>>>) -> Self {
        Self { items }
    }

    pub fn insert_item(&mut self, path: Path, item: T) -> Option<Rc<T>> {
        self.insert(path, Rc::new(item))
    }
}

impl<'a, T> Db<Rc<T>> for MemoryDb<'a, T>
where
    T: Clone,
{
    fn set(&mut self, path: Path, item: Option<Rc<T>>) -> Option<Rc<T>> {
        let mut db = self.items.write().unwrap();
        match item {
            Some(i) => db.insert(path, i),
            None => db.remove(&path),
        }
    }

    fn get(&self, path: &Path) -> Option<Rc<T>> {
        self.items.read().unwrap().get(path).map(|rc| rc.clone())
    }

    type List = Vec<(Path, Rc<T>)>;
    fn list(&self, sync_path: &Path) -> Self::List {
        self.items
            .read()
            .unwrap()
            .iter()
            .filter(|(path, _)| sync_path.is_parent_of(path))
            .map(|(path, item)| (path.clone(), item.clone()))
            .collect()
    }
}

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
    local.set(path1.clone(), item1);

    let path2: Path = vec!["folder", "item2"].into();
    let item2 = "store me, too";
    remote.set(path2.clone(), item2);

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

struct MemoryDb<'a, T> {
    items: &'a RwLock<HashMap<Path, Rc<T>>>,
}

impl<'a, T> MemoryDb<'a, T> {
    pub fn with(items: &'a RwLock<HashMap<Path, Rc<T>>>) -> Self {
        Self { items }
    }
}

impl<'a, T> Db<T> for MemoryDb<'a, T>
where
    T: Clone,
{
    fn set(&mut self, path: Path, item: T) {
        self.items.write().unwrap().insert(path, Rc::new(item));
    }

    fn get(&self, path: &Path) -> Option<T> {
        self.items.read().unwrap().get(path).map(|rc| T::clone(rc))
    }

    type List = Vec<(Path, T)>;
    fn list(&self, sync_path: &Path) -> Self::List {
        self.items
            .read()
            .unwrap()
            .iter()
            .filter(|(path, _)| sync_path.is_parent_of(path))
            .map(|(path, item)| (path.clone(), T::clone(item)))
            .collect()
    }
}

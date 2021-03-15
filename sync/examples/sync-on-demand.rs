use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use sync::memory_db::MemoryDb;
use sync::{Db, Depth, FilePath, FolderPath, SyncDb};

fn main() {
    // Create our databases.
    let local_items = Rc::new(RwLock::new(HashMap::new()));
    let mut local = MemoryDb::with(local_items.clone());
    let remote_items = Rc::new(RwLock::new(HashMap::new()));
    let mut remote = MemoryDb::with(remote_items.clone());
    let mut db = SyncDb::new(MemoryDb::with(local_items), MemoryDb::with(remote_items));

    // Insert some items.
    smol::block_on(async {
        let folder_path: FolderPath = vec!["folder"].into();

        let path1 = FilePath::new(folder_path.clone(), "item1".to_string());
        let item1 = "store me";
        local.insert_item(path1.clone(), item1).await;

        let path2 = FilePath::new(folder_path, "item2".to_string());
        let item2 = "store me, too";
        remote.insert_item(path2.clone(), item2).await;

        // Synchronize the databases.
        db.sync_folder(&Depth::Recursive, &FolderPath::root()).await;

        // Look inside the databases.
        let folder_path = vec!["folder"].into();
        let mut stored_locally = local.list(&Depth::Simple, &folder_path).await;
        let mut stored_remotely = remote.list(&Depth::Simple, &folder_path).await;

        // Sort to ensure they are in the same order for comparison.
        let expected = vec![(path1, Rc::new(item1)), (path2, Rc::new(item2))];
        stored_locally.sort_by_key(|(path, _)| path.clone());
        stored_remotely.sort_by_key(|(path, _)| path.clone());
        assert_eq!(expected, stored_locally);
        assert_eq!(expected, stored_remotely);
    });
}

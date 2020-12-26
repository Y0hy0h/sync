use proptest::prelude::*;
use std::rc::Rc;

use sync::memory_db::MemoryDb;
use sync::{Db, Depth, FilePath, FolderPath, SyncDb};

proptest! {
    #[test]
    fn makes_repositories_consistent(local_items in items(), remote_items in items()) {
        let mut local = MemoryDb::new();
        for (path, item) in local_items {
            local.insert(path, Rc::new(item));
        }

        let mut remote = MemoryDb::new();
        for (path, item) in remote_items{
            remote.insert(path, Rc::new(item));
        }

        let mut sync = SyncDb::new(local.clone(), remote.clone());

        let root_folder = FolderPath::root();
        sync.sync_folder(&Depth::Recursive, &root_folder);

        let mut stored_locally = local.list(&Depth::Recursive, &root_folder);
        let mut stored_remotely = remote.list(&Depth::Recursive, &root_folder);

        stored_locally.sort_by_key(|(path, _)| path.clone());
        stored_remotely.sort_by_key(|(path, _)| path.clone());
        assert_eq!(stored_locally, stored_remotely);
    }
}

prop_compose! {
    fn items()
            (entries in proptest::collection::vec(item(), 0..100))
            -> Vec<(FilePath, String)> {
        entries
    }
}

prop_compose! {
    fn item()
            (file_path in file_path(),
            content in ".+")
            -> (FilePath, String) {
        (file_path, content.to_string())
    }
}

const PATH_COMPONENT: &'static str = "[a-z]+";

prop_compose! {
    fn file_path()
            (folder in proptest::collection::vec(PATH_COMPONENT, 0..3),
            file_name in PATH_COMPONENT)
            -> FilePath {
        FilePath::new(folder.into(), file_name.to_string())
    }
}

use core::convert::TryFrom;
use core::marker::PhantomData;
use std::collections::HashSet;

pub struct SyncDb<T, L, R>
where
    L: Db<T>,
    R: Db<T>,
{
    local: L,
    remote: R,
    phantom: PhantomData<T>,
}

impl<T, L, R> SyncDb<T, L, R>
where
    L: Db<T>,
    R: Db<T>,
    T: Clone + Eq,
{
    pub fn new(local: L, remote: R) -> Self {
        Self {
            local: local,
            remote: remote,
            phantom: PhantomData,
        }
    }

    pub fn sync_folder(&mut self, depth: &Depth, sync_path: &FolderPath) {
        let paths_to_sync: HashSet<FilePath> = self
            .local
            .list(depth, sync_path)
            .into_iter()
            .chain(self.remote.list(depth, sync_path).into_iter())
            .map(|(path, _)| path)
            .collect();

        for path in paths_to_sync {
            self.sync_file(path)
        }
    }

    pub fn sync_file(&mut self, sync_path: FilePath) {
        let local = self.local.get(&sync_path);
        let remote = self.remote.get(&sync_path);

        match (local, remote) {
            (Some(l), Some(r)) => {
                if l != r {
                    self.local.insert(sync_path, r);
                }
            }
            (Some(l), None) => {
                self.remote.insert(sync_path, l);
            }
            (None, Some(r)) => {
                self.local.insert(sync_path, r);
            }
            _ => (),
        }
    }
}

pub trait Db<T> {
    fn set(&mut self, path: FilePath, item: Option<T>) -> Option<T>;

    fn insert(&mut self, path: FilePath, item: T) -> Option<T> {
        self.set(path, Some(item))
    }

    fn remove(&mut self, path: FilePath) -> Option<T> {
        self.set(path, None)
    }

    fn get(&self, path: &FilePath) -> Option<T>;

    type List: IntoIterator<Item = (FilePath, T)>;
    fn list(&self, depth: &Depth, path: &FolderPath) -> Self::List;
}

pub enum Depth {
    Recursive,
    Simple,
}

#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone)]
pub struct FolderPath {
    parts: Vec<String>,
}

impl From<Vec<&'static str>> for FolderPath {
    fn from(parts: Vec<&'static str>) -> Self {
        FolderPath {
            parts: parts.into_iter().map(|part| part.to_string()).collect(),
        }
    }
}

impl FolderPath {
    pub fn contains(&self, other: &FolderPath) -> bool {
        self.parts
            .iter()
            .zip(other.parts.iter())
            .all(|(first, second)| first == second)
    }
}

impl std::fmt::Debug for FolderPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", self.parts.join("/"))
    }
}

#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone)]
pub struct FilePath {
    folder: FolderPath,
    file_name: String,
}

impl FilePath {
    pub fn new(folder: FolderPath, file_name: String) -> Self {
        FilePath { folder, file_name }
    }

    pub fn folder(&self) -> &FolderPath {
        &self.folder
    }

    pub fn file_name(&self) -> &String {
        &self.file_name
    }
}

impl TryFrom<Vec<&'static str>> for FilePath {
    type Error = FilePathCreationError;

    fn try_from(mut parts: Vec<&'static str>) -> Result<Self, Self::Error> {
        if parts.is_empty() {
            return Err(FilePathCreationError::MissingFileName);
        }

        let file_name = parts.pop().unwrap().to_string();
        Ok(FilePath {
            folder: parts.into(),
            file_name,
        })
    }
}

impl std::fmt::Debug for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{folder:?}{file_name}",
            folder = self.folder,
            file_name = self.file_name
        )
    }
}

pub enum FilePathCreationError {
    MissingFileName,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

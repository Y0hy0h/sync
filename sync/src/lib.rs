use core::convert::TryFrom;
use core::marker::PhantomData;
use std::collections::HashSet;

use async_trait::async_trait;

pub mod memory_db;

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

    pub async fn sync_folder(&mut self, depth: &Depth, sync_path: &FolderPath) {
        let paths_to_sync: HashSet<FilePath> = self
            .local
            .list(depth, sync_path)
            .await
            .into_iter()
            .chain(self.remote.list(depth, sync_path).await.into_iter())
            .map(|(path, _)| path)
            .collect();

        for path in paths_to_sync {
            self.sync_file(path).await
        }
    }

    pub async fn sync_file(&mut self, sync_path: FilePath) {
        let local = self.local.get(&sync_path).await;
        let remote = self.remote.get(&sync_path).await;

        match (local, remote) {
            (Some(l), Some(r)) => {
                if l != r {
                    self.local.insert(sync_path, r).await;
                }
            }
            (Some(l), None) => {
                self.remote.insert(sync_path, l).await;
            }
            (None, Some(r)) => {
                self.local.insert(sync_path, r).await;
            }
            (None, None) => (),
        }
    }
}

#[async_trait(?Send)]
pub trait Db<T> {
    async fn set(&mut self, path: FilePath, item: Option<T>) -> Option<T>;

    async fn insert(&mut self, path: FilePath, item: T) -> Option<T>
    where
        T: 'async_trait,
    {
        self.set(path, Some(item)).await
    }

    async fn remove(&mut self, path: FilePath) -> Option<T>
    where
        T: 'async_trait,
    {
        self.set(path, None).await
    }

    async fn get(&self, path: &FilePath) -> Option<T>;

    type List: IntoIterator<Item = (FilePath, T)>;
    async fn list(&self, depth: &Depth, path: &FolderPath) -> Self::List;
}

pub enum Depth {
    Recursive,
    Simple,
}

#[derive(Eq, PartialEq, Hash, PartialOrd, Ord, Clone)]
pub struct FolderPath {
    parts: Vec<String>,
}

impl<S> From<Vec<S>> for FolderPath
where
    S: ToString,
{
    fn from(parts: Vec<S>) -> Self {
        FolderPath {
            parts: parts.into_iter().map(|part| part.to_string()).collect(),
        }
    }
}

impl FolderPath {
    pub fn root() -> Self {
        FolderPath { parts: vec![] }
    }

    pub fn contains(&self, other: &Self) -> bool {
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

impl<S> TryFrom<Vec<S>> for FilePath
where
    S: ToString,
{
    type Error = FilePathCreationError;

    fn try_from(mut parts: Vec<S>) -> Result<Self, Self::Error> {
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

#[derive(Debug)]
pub enum FilePathCreationError {
    MissingFileName,
}

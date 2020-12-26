use core::marker::PhantomData;

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

    pub fn sync(&mut self, sync_path: &Path) {
        let remotes = self.remote.list(sync_path);

        for (path, item) in self.local.list(sync_path) {
            if self.remote.get(&path).as_ref() != Some(&item) {
                self.remote.insert(path, item.clone());
            }
        }

        for (path, item) in remotes {
            if self.local.get(&path).as_ref() != Some(&item) {
                self.local.insert(path, item.clone());
            }
        }
    }
}

pub trait Db<T> {
    fn set(&mut self, path: Path, item: Option<T>) -> Option<T>;

    fn insert(&mut self, path: Path, item: T) -> Option<T> {
        self.set(path, Some(item))
    }

    fn remove(&mut self, path: Path) -> Option<T> {
        self.set(path, None)
    }

    fn get(&self, path: &Path) -> Option<T>;

    type List: IntoIterator<Item = (Path, T)>;
    fn list(&self, path: &Path) -> Self::List;
}

#[derive(Eq, PartialEq, Hash, Debug, PartialOrd, Ord, Clone)]
pub struct Path {
    parts: Vec<&'static str>,
}

impl From<Vec<&'static str>> for Path {
    fn from(parts: Vec<&'static str>) -> Self {
        Path { parts }
    }
}

impl Path {
    pub fn is_parent_of(&self, other: &Path) -> bool {
        self.parts
            .iter()
            .zip(other.parts.iter())
            .all(|(first, second)| first == second)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

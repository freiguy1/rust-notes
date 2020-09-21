use std::fs::{DirEntry, ReadDir};
use std::path::Path;
use std::{fs, io};

// Source for walk_dir func, struct, and iterator implementation
// copied from rust nightly source. As of now, it is unstable.
pub fn walk_dir<P: AsRef<Path>>(path: P) -> io::Result<WalkDir> {
    let start = fs::read_dir(path)?;
    Ok(WalkDir {
        cur: Some(start),
        stack: Vec::new(),
    })
}

pub struct WalkDir {
    cur: Option<ReadDir>,
    stack: Vec<io::Result<ReadDir>>,
}

impl Iterator for WalkDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        loop {
            if let Some(ref mut cur) = self.cur {
                match cur.next() {
                    Some(Err(e)) => return Some(Err(e)),
                    Some(Ok(next)) => {
                        let path = next.path();
                        let path_metadata = fs::metadata(&path)
                            .ok()
                            .expect("Error fetching metadata for file");
                        if path_metadata.is_dir() {
                            self.stack.push(fs::read_dir(&*path));
                        }
                        return Some(Ok(next));
                    }
                    None => {}
                }
            }
            self.cur = None;
            match self.stack.pop() {
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(next)) => self.cur = Some(next),
                None => return None,
            }
        }
    }
}

// Source for relative_from copied from rust nightly source.
pub trait RelativeFrom {
    fn my_relative_from<'a, P: ?Sized + AsRef<Path>>(&'a self, base: &'a P) -> Option<&Path>;
}

impl RelativeFrom for Path {
    fn my_relative_from<'a, P: ?Sized + AsRef<Path>>(&'a self, base: &'a P) -> Option<&Path> {
        iter_after(self.components(), base.as_ref().components()).map(|c| c.as_path())
    }
}

fn iter_after<A, I, J>(mut iter: I, mut prefix: J) -> Option<I>
where
    I: Iterator<Item = A> + Clone,
    J: Iterator<Item = A>,
    A: PartialEq,
{
    loop {
        let mut iter_next = iter.clone();
        match (iter_next.next(), prefix.next()) {
            (Some(x), Some(y)) => {
                if x != y {
                    return None;
                }
            }
            (Some(_), None) => return Some(iter),
            (None, None) => return Some(iter),
            (None, Some(_)) => return None,
        }
        iter = iter_next;
    }
}


use std::{ fs, io };
use std::fs::{ PathExt, ReadDir, DirEntry };
use std::path::Path;


// Source for walk_dir func, struct, and iterator implementation
// copied from rust nightly source. As of now, it is unstable.
pub fn walk_dir<P: AsRef<Path>>(path: P) -> io::Result<WalkDir> {
    let start = try!(fs::read_dir(path));
    Ok(WalkDir { cur: Some(start), stack: Vec::new() })
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
                        if path.is_dir() {
                            self.stack.push(fs::read_dir(&*path));
                        }
                        return Some(Ok(next))
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


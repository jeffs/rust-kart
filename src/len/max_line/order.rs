fn len(line: &String) -> usize {
    line.chars().count()
}

pub trait Less {
    fn less(old: usize, new: usize) -> bool;
}

pub enum Longest {}

impl Less for Longest {
    fn less(old: usize, new: usize) -> bool {
        old < new
    }
}

pub enum Shortest {}

impl Less for Shortest {
    fn less(old: usize, new: usize) -> bool {
        new < old
    }
}

pub trait Policy {
    fn max(old: Option<String>, new: Option<String>) -> Option<String>;
}

impl<L: Less> Policy for L {
    fn max(old: Option<String>, new: Option<String>) -> Option<String> {
        match old {
            Some(old) => new.map(|new| {
                if L::less(len(&old), len(&new)) {
                    new
                } else {
                    old
                }
            }),
            None => new,
        }
    }
}

use std::{
    fmt::{Debug, Display},
    ops::Range,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub ln: Range<usize>,
    pub col: Range<usize>,
}
pub struct Indexed<T> {
    pub value: T,
    pub index: Range<usize>,
}
pub struct Located<T> {
    pub value: T,
    pub pos: Position,
}
pub struct PathLocated<T> {
    pub value: T,
    pub path: String,
    pub pos: Position,
}

impl Position {
    #[inline(always)]
    pub fn new(ln: Range<usize>, col: Range<usize>) -> Self {
        Self { ln, col }
    }
    #[inline(always)]
    pub fn single(ln: usize, col: usize) -> Self {
        Self {
            ln: ln..ln + 1,
            col: col..col + 1,
        }
    }
    #[inline(always)]
    pub fn extend(&mut self, other: &Self) {
        self.ln.end = other.ln.end;
        self.col.end = other.col.end;
    }
}
impl<T> Indexed<T> {
    #[inline(always)]
    pub fn new(value: T, index: Range<usize>) -> Self {
        Self { value, index }
    }
    #[inline(always)]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Indexed<U> {
        Indexed {
            value: f(self.value),
            index: self.index,
        }
    }
    #[inline(always)]
    pub fn with_ln(self, ln: usize) -> Located<T> {
        Located {
            value: self.value,
            pos: Position::new(ln..ln, self.index),
        }
    }
}
impl<T: Clone> Clone for Indexed<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            index: self.index.clone(),
        }
    }
}
impl<T: PartialEq> PartialEq for Indexed<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: Default> Default for Indexed<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            value: T::default(),
            index: Range::default(),
        }
    }
}
impl<T: Debug> Debug for Indexed<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display> Display for Indexed<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T> Located<T> {
    #[inline(always)]
    pub fn new(value: T, pos: Position) -> Self {
        Self { value, pos }
    }
    #[inline(always)]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Located<U> {
        Located {
            value: f(self.value),
            pos: self.pos,
        }
    }
    #[inline(always)]
    pub fn with_path<S: ToString>(self, path: S) -> PathLocated<T> {
        PathLocated {
            value: self.value,
            path: path.to_string(),
            pos: self.pos,
        }
    }
}
impl<T: Clone> Clone for Located<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            pos: self.pos.clone(),
        }
    }
}
impl<T: PartialEq> PartialEq for Located<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: Default> Default for Located<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            value: T::default(),
            pos: Position::default(),
        }
    }
}
impl<T: Debug> Debug for Located<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display> Display for Located<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> PathLocated<T> {
    #[inline(always)]
    pub fn new(value: T, path: String, pos: Position) -> Self {
        Self { value, path, pos }
    }
    #[inline(always)]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Located<U> {
        Located {
            value: f(self.value),
            pos: self.pos,
        }
    }
}
impl<T: Clone> Clone for PathLocated<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            path: self.path.clone(),
            pos: self.pos.clone(),
        }
    }
}
impl<T: PartialEq> PartialEq for PathLocated<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: Default> Default for PathLocated<T> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            value: T::default(),
            path: "<input.luna>".to_string(),
            pos: Position::default(),
        }
    }
}
impl<T: Debug> Debug for PathLocated<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display> Display for PathLocated<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

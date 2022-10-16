use std::collections::{vec_deque, VecDeque};
use std::iter::IntoIterator;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct Snake {
    pos: VecDeque<(usize, usize)>,
    dir: Directions,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Food {
    Exists(usize, usize),
    Eaten,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Directions {
    Up,
    Down,
    Right,
    Left,
    Stop,
}

impl IntoIterator for Snake {
    type Item = (usize, usize);
    type IntoIter = vec_deque::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.pos.into_iter()
    }
}

impl Snake {
    pub fn new() -> Self {
        Self {
            pos: VecDeque::from(vec![(2, 2), (2, 3)]),
            dir: Directions::Stop,
        }
    }

    pub fn len(&self) -> usize {
        self.pos.len()
    }

    pub fn move_heads(&mut self, food: &mut Food, range: (usize, usize), exit: Arc<AtomicBool>) {
        use Directions::*;
        if self.dir != Stop {
            let (next_x, next_y) = self.pos.front().unwrap();
            let next = match self.dir {
                Up => {
                    let next = (*next_y).wrapping_sub(1);
                    if next < range.1
                        && next != 0
                        && next != 1
                        && !self.pos.contains(&(*next_x, next))
                    {
                        (*next_x, next)
                    } else {
                        exit.swap(true, Ordering::Relaxed);
                        (*next_x, *next_y)
                    }
                }
                Down => {
                    let next = (*next_y).wrapping_add(1);
                    if next < range.1
                        && next != 0
                        && next != 1
                        && !self.pos.contains(&(*next_x, next))
                    {
                        (*next_x, next)
                    } else {
                        exit.swap(true, Ordering::Relaxed);
                        (*next_x, *next_y)
                    }
                }
                Right => {
                    let next = (*next_x).wrapping_add(1);
                    if next < range.0
                        && next != 0
                        && next != 1
                        && !self.pos.contains(&(next, *next_y))
                    {
                        (next, *next_y)
                    } else {
                        exit.swap(true, Ordering::Relaxed);
                        (*next_x, *next_y)
                    }
                }
                Left => {
                    let next = (*next_x).wrapping_sub(1);
                    if next < range.0
                        && next != 0
                        && next != 1
                        && !self.pos.contains(&(next, *next_y))
                    {
                        (next, *next_y)
                    } else {
                        exit.swap(true, Ordering::Relaxed);
                        (*next_x, *next_y)
                    }
                }
                Stop => (1, 1),
            };
            if let Food::Exists(x, y) = food {
                if (*x, *y) != next {
                    self.pos.pop_back();
                } else {
                    *food = Food::Eaten;
                }
            }
            if !exit.load(Ordering::Relaxed) {
                self.pos.push_front(next);
            }
        }
    }

    pub fn change_dir(&mut self, dir: Directions) {
        use Directions::*;
        match dir {
            Up if self.dir != Down => self.dir = dir,
            Down if self.dir != Up => self.dir = dir,
            Right if self.dir != Left => self.dir = dir,
            Left if self.dir != Right => self.dir = dir,
            _ => (),
        }
    }
}

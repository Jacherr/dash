use std::{fmt::Debug, mem::MaybeUninit};

#[derive(Debug)]
pub struct Stack<T, const N: usize>([MaybeUninit<T>; N], usize);

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        Self(MaybeUninit::uninit_array(), 0)
    }

    pub fn push(&mut self, v: T) {
        assert!(N > self.1);

        unsafe { self.0[self.1].as_mut_ptr().write(v) };
        self.1 += 1;
    }

    pub fn pop(&mut self) -> T {
        assert!(self.1 > 0);

        self.1 -= 1;
        let old = &mut self.0[self.1];
        let val = std::mem::replace(old, MaybeUninit::uninit());
        unsafe { val.assume_init() }
    }

    pub fn get(&self) -> &T {
        unsafe { self.0[self.1 - 1].assume_init_ref() }
    }

    pub fn reset(&mut self) {
        self.1 = 0;
    }

    /// Discards all values on the stack except for the one at the top
    pub fn into_last(&mut self) {
        let last = self.pop();

        for _ in 0..self.1 {
            self.pop();
        }

        self.push(last);
    }

    pub fn dump(&self)
    where
        T: Debug,
    {
        println!("=== STACK DUMP [sp={}] ===", self.1);
        for (idx, val) in self.0.iter().take(self.1).enumerate() {
            let val = unsafe { val.assume_init_ref() };
            println!("{:04}    {:?}", idx, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn stack() {
        let mut s: Stack<_, 127> = Stack::new();

        for i in 0..127 {
            s.push(i);
        }

        for i in (0..127).rev() {
            assert_eq!(s.pop(), i);
        }
    }
}

use anyhow::Result;
use std::ops::{Add, AddAssign, Deref, Mul};

pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Vector<T>
where
    T: Copy + Default + Mul<Output = T> + Add<Output = T> + AddAssign,
{
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }

    // pretend this is a heavy operation, CPU intensive
    pub fn dot_product(&self, other: &Vector<T>) -> Result<T> {
        if self.len() != other.len() {
            return Err(anyhow::anyhow!("length of a must equal to length of b"));
        }

        let mut sum = T::default();
        for i in 0..self.len() {
            sum += self[i] * other[i];
        }

        Ok(sum)
    }
}

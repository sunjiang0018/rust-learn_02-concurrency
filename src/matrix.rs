use core::fmt;
use std::{
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use anyhow::Result;

use crate::vector::Vector;

const NUM_THREADS: usize = 4;

pub struct Matrix<T> {
    data: Vec<T>,
    col: usize,
    row: usize,
}

struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

struct MsgOutput<T> {
    idx: usize,
    value: T,
}

struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> MsgInput<T> {
    fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}
impl<T> MsgOutput<T> {
    fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

impl<T> Msg<T> {
    fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T> Mul for Matrix<T>
where
    T: Copy + Default + Mul<Output = T> + Add<Output = T> + AddAssign + Send + 'static,
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).unwrap()
    }
}

impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //display a 2X3 as {1 2 3, 4 5 6}, 3X2 as {1 2, 3 4, 5 6}
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: Copy + Default + Mul<Output = T> + Add<Output = T> + AddAssign + Send + 'static,
{
    if a.col != b.row {
        return Err(anyhow::anyhow!("cols of a must equal to rows of b"));
    }

    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    let value = msg.input.row.dot_product(&msg.input.col).unwrap();
                    msg.sender
                        .send(MsgOutput::new(msg.input.idx, value))
                        .unwrap();
                }
            });
            tx
        })
        .collect::<Vec<_>>();

    let matrix_len = a.col * b.row;
    let mut receivers = Vec::with_capacity(matrix_len);
    let mut data: Vec<T> = vec![T::default(); matrix_len];
    for i in 0..a.row {
        for j in 0..b.col {
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);
            let col_data = b
                .data
                .iter()
                .skip(j)
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);
            let input = MsgInput::new(i * b.col + j, row, col);
            let (tx, rx) = oneshot::channel::<MsgOutput<T>>();
            let msg = Msg::new(input, tx);
            senders[i % NUM_THREADS].send(msg).unwrap();
            receivers.push(rx);
        }
    }

    for recv in receivers {
        let output = recv.recv()?;
        data[output.idx] = output.value;
    }

    Ok(Matrix::new(data, a.row, b.col))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        assert_eq!(c.col, 2);
        assert_eq!(c.row, 2);
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }
    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a * b;
        assert_eq!(c.data, [7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");

        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _c = a * b;
    }

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }
}

#![no_std]

extern crate generic_array;

use core::marker::PhantomData;
use generic_array::{ArrayLength, GenericArray};
use generic_array::sequence::GenericSequence;
use generic_array::functional::FunctionalSequence;
use generic_array::typenum::Unsigned;

pub trait KeyColumns<N: Unsigned> {
    fn size(&self) -> N;
    fn enable_column(&mut self, col: usize) -> Result<(), ()>;
    fn disable_column(&mut self, col: usize) -> Result<(), ()>;
}

pub trait KeyRows<N: Unsigned> {
    fn size(&self) -> N;
    fn read_row(&mut self, col: usize) -> Result<bool, ()>;
}

pub struct KeyMatrix<CN, RN, C, R> where RN: Unsigned + ArrayLength<bool> + ArrayLength<u8>,
                                         CN: Unsigned + ArrayLength<GenericArray<bool, RN>> + ArrayLength<GenericArray<u8, RN>>,
                                         C: KeyColumns<CN>,
                                         R: KeyRows<RN> {
    cols: C,
    rows: R,
    debounce_count: u8,
    state: GenericArray<GenericArray<u8, RN>, CN>,

    _cn: PhantomData<CN>,
    _cr: PhantomData<RN>,
}

impl<CN, RN, C, R> KeyMatrix<CN, RN, C, R> where RN: Unsigned + ArrayLength<bool> + ArrayLength<u8>,
                                                 CN: Unsigned + ArrayLength<GenericArray<bool, RN>> + ArrayLength<GenericArray<u8, RN>>,
                                                 C: KeyColumns<CN>,
                                                 R: KeyRows<RN>,
{
    /// Create a new key matrix with the given column and row structs.
    ///
    /// The debounce parameter specifies in how many subsequent calls of
    /// `poll()` a key has to be registered as pressed, in order to be
    /// considered pressed by `current_state()`.
    pub fn new(debounce_count: u8, cols: C, rows: R) -> Self {
        KeyMatrix {
            cols,
            rows,
            debounce_count,
            state: Self::init_state(),
            _cn: PhantomData,
            _cr: PhantomData,
        }
    }

    fn init_state() -> GenericArray<GenericArray<u8, RN>, CN> {
        return GenericArray::generate(|_i| GenericArray::generate(|_j| 0u8));
    }

    /// Scan the key matrix once.
    ///
    /// If the matrix was created with a `debounce_count > 0`, this must be
    /// called at least that number of times + 1 to actually show a key as
    /// pressed.
    pub fn poll(&mut self) -> Result<(), ()> {
        for i in 0..<CN as Unsigned>::to_usize() {
            self.cols.enable_column(i)?;

            for j in 0..<RN as Unsigned>::to_usize() {
                match self.rows.read_row(j)? {
                    true => {
                        let cur: u8 = self.state[i][j];
                        // Saturating add to prevent overflow
                        self.state[i][j] = cur.saturating_add(1);
                    }
                    false => {
                        self.state[i][j] = 0;
                    }
                }
            }

            self.cols.disable_column(i)?;
        }
        Ok(())
    }

    /// Return a 2-dimensional array of the last polled state of the matrix.
    pub fn current_state(&self) -> GenericArray<GenericArray<bool, RN>, CN> {
        self.state.clone()
            .map(|col| {
                col.map(|elem| {
                    elem > self.debounce_count
                })
            })
    }

    /// Return the number of rows that the matrix was created with.
    pub fn row_size(&self) -> usize {
        <RN as Unsigned>::to_usize()
    }

    /// Return the number of columns that the matrix was created with.
    pub fn col_size(&self) -> usize {
        <CN as Unsigned>::to_usize()
    }
}

#[macro_export]
macro_rules! key_columns {
    (
        $Type:ident,
        $size_type:ty,
        [$(
            $col_name:ident : ($index:expr , $pintype:ty)
        ),+]) => {
use $crate::KeyColumns;

pub struct $Type {
    $(
        $col_name: $pintype,
    )+
}

impl $Type {
    pub fn new(
        $(
            $col_name: $pintype,
        )+
    ) -> $Type {
        $Type {
            $(
                $col_name,
            )+
        }
    }
}

impl KeyColumns<$size_type> for $Type {
    fn size(&self) -> $size_type {
        <$size_type>::new()
    }

    fn enable_column(&mut self, col: usize) -> Result<(), ()> {
        match col {
            $(
            $index => self.$col_name.set_high().map_err(drop),
            )+
            _ => unreachable!()
        }
    }

    fn disable_column(&mut self, col: usize) -> Result<(), ()> {
        match col {
            $(
            $index => self.$col_name.set_low().map_err(drop),
            )+
            _ => unreachable!()
        }
    }
}

// End macro
};
}

#[macro_export]
macro_rules! key_rows {
    (
        $Type:ident,
        $size_type:ty,
        [$(
            $row_name:ident : ($index:expr , $pintype:ty)
        ),+]) => {
use $crate::KeyRows;

pub struct $Type {
    $(
        $row_name: $pintype,
    )+
}

impl $Type {
    pub fn new(
        $(
            $row_name: $pintype,
        )+
    ) -> $Type {
        $Type {
            $(
                $row_name,
            )+
        }
    }
}

impl KeyRows<$size_type> for $Type {
    fn size(&self) -> $size_type {
        <$size_type>::new()
    }

    fn read_row(&mut self, row: usize) -> Result<bool, ()> {
        match row {
            $(
            $index => self.$row_name.is_high().map_err(drop),
            )+
            _ => unreachable!()
        }
    }
}

// End Macro
};
}


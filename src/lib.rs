#![no_std]

extern crate cortex_m;
extern crate embedded_hal as hal;
extern crate generic_array;

use core::marker::PhantomData;
use generic_array::{ArrayLength, GenericArray};
use generic_array::sequence::GenericSequence;
use generic_array::functional::FunctionalSequence;
use generic_array::typenum::Unsigned;
use hal::timer::{CountDown, Periodic};

use core::default::Default;

pub trait KeyColumns<N: Unsigned> {
    fn size(&self) -> N;
    fn enable_column(&mut self, col: usize);
    fn disable_column(&mut self, col: usize);
}

pub trait KeyRows<N: Unsigned> {
    fn size(&self) -> N;
    fn read_row(&mut self, col: usize) -> bool;
}

pub struct KeyMatrix<CN, RN, C, R> where RN: Unsigned + ArrayLength<bool> + ArrayLength<u8>,
                                         CN: Unsigned + ArrayLength<GenericArray<bool, RN>> + ArrayLength<GenericArray<u8, RN>>,
                                         C: KeyColumns<CN>,
                                         R: KeyRows<RN> {
    cols: C,
    rows: R,
    debounce: GenericArray<GenericArray<u8, RN>, CN>,

    _cn: PhantomData<CN>,
    _cr: PhantomData<RN>,
}

impl<CN, RN, C, R> KeyMatrix<CN, RN, C, R> where RN: Unsigned + ArrayLength<bool> + ArrayLength<u8>,
                                                 CN: Unsigned + ArrayLength<GenericArray<bool, RN>> + ArrayLength<GenericArray<u8, RN>>,
                                                 C: KeyColumns<CN>,
                                                 R: KeyRows<RN>,
{
    pub fn new<TU, CT, T>(counter: &mut CT,
                          freq: T,
                          cols: C,
                          rows: R) -> KeyMatrix<CN, RN, C, R>
        where T: Into<TU>,
              CT: CountDown<Time=TU> + Periodic,
              C: KeyColumns<CN>,
              R: KeyRows<RN>,
    {
        counter.start(freq.into());
        KeyMatrix {
            cols,
            rows,
            debounce: KeyMatrix::<CN, RN, C, R>::init_state(),
            _cn: PhantomData,
            _cr: PhantomData,
            //counter
        }
    }

    fn init_state() -> GenericArray<GenericArray<u8, RN>, CN> {
        return GenericArray::generate(|_i| GenericArray::generate(|_j| Default::default()));
    }

    fn init_output_state() -> GenericArray<GenericArray<bool, RN>, CN> {
        return GenericArray::generate(|_i| GenericArray::generate(|_j| Default::default()));
    }

    pub fn poll(&mut self) {
        for i in 0..<CN as Unsigned>::to_usize() {
            self.cols.enable_column(i);

            for j in 0..<RN as Unsigned>::to_usize() {
                match self.rows.read_row(j) {
                    true => {
                        let cur: u8 = self.debounce[i][j];
                        // Saturating add to prevent overflow
                        self.debounce[i][j] = cur.saturating_add(1);
                    }
                    false => {
                        self.debounce[i][j] = 0;
                    }
                }
            }

            self.cols.disable_column(i);
        }
    }

    pub fn current_state(&self) -> GenericArray<GenericArray<bool, RN>, CN> {
        self.debounce.clone().map(|col| col.map(|elem| elem > 5))
    }

    pub fn row_size(&self) -> usize {
        <RN as Unsigned>::to_usize()
    }

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

    fn enable_column(&mut self, col: usize) {
        match col {
            $(
            $index => self.$col_name.set_high(),
            )+
            _ => unreachable!()
        };
    }

    fn disable_column(&mut self, col: usize) {
        match col {
            $(
            $index => self.$col_name.set_low(),
            )+
            _ => unreachable!()
        };
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

    fn read_row(&mut self, row: usize) -> bool {
        match row {
            $(
            $index => self.$row_name.is_high(),
            )+
            _ => unreachable!()
        }
    }
}

// End Macro
};
}


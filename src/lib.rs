#![no_std]

extern crate cortex_m;
extern crate embedded_hal as hal;
extern crate generic_array;

use core::marker::PhantomData;
use generic_array::{ArrayLength, GenericArray};
use generic_array::sequence::GenericSequence;
use generic_array::typenum::Unsigned;
use hal::timer::{CountDown, Periodic};

pub trait KeyColumns<N: Unsigned> {
    fn size(&self) -> N;
    fn enable_column(&mut self, col: usize);
    fn disable_column(&mut self, col: usize);
}

pub trait KeyRows<N: Unsigned> {
    fn size(&self) -> N;
    fn read_row(&mut self, col: usize) -> bool;
}

pub struct KeyMatrix<CN: Unsigned, RN: Unsigned, C: KeyColumns<CN>, R: KeyRows<RN>> {
    cols: C,
    rows: R,

    _cn: PhantomData<CN>,
    _cr: PhantomData<RN>,
    //counter: C,
}

impl<CN, RN, C, R> KeyMatrix<CN, RN, C, R> where CN: Unsigned + ArrayLength<bool>,
                                                 RN: Unsigned + ArrayLength<bool>,
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
            _cn: PhantomData,
            _cr: PhantomData,
            //counter
        }
    }

    pub fn poll(&mut self) {
        for i in <CN as Unsigned>::to_usize().. {
            self.cols.enable_column(i);

            let mut row_state: GenericArray<bool, RN> = GenericArray::generate(|_i| false);
            //let row_builder = ArrayBuilder::new();
            for j in <RN as Unsigned>::to_usize().. {
                row_state[j] = self.rows.read_row(j);
            }

            self.cols.disable_column(i);
        }
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


#![no_std]

pub trait KeyColumns<const N: usize> {
    fn size(&self) -> usize;
    fn enable_column(&mut self, col: usize) -> Result<(), ()>;
    fn disable_column(&mut self, col: usize) -> Result<(), ()>;
}

pub trait KeyRows<const N: usize> {
    fn size(&self) -> usize;
    fn read_row(&mut self, col: usize) -> Result<bool, ()>;
}

pub struct KeyMatrix<C, R, const CN: usize, const RN: usize>
where C: KeyColumns<CN>, R: KeyRows<RN> {
    cols: C,
    rows: R,
    debounce_count: u8,
    state: [[u8; RN]; CN],
}

impl<C, R, const CN: usize, const RN: usize> KeyMatrix<C, R, CN, RN>
where C: KeyColumns<CN>, R: KeyRows<RN> {
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
        }
    }

    fn init_state() -> [[u8; RN]; CN] {
        [[0; RN]; CN]
    }

    /// Scan the key matrix once.
    ///
    /// If the matrix was created with a `debounce_count > 0`, this must be
    /// called at least that number of times + 1 to actually show a key as
    /// pressed.
    pub fn poll(&mut self) -> Result<(), ()> {
        for i in 0..CN {
            self.cols.enable_column(i)?;

            for j in 0..RN {
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
    pub fn current_state(&self) -> [[bool; RN]; CN] {
        let mut state = [[false; RN]; CN];
        for (i, row) in self.state.iter().enumerate() {
            for (j, &elem) in row.iter().enumerate() {
                if elem > self.debounce_count {
                    state[i][j] = true;
                }
            }
        }
        state
    }

    /// Return the number of rows that the matrix was created with.
    pub fn row_size(&self) -> usize {
        RN
    }

    /// Return the number of columns that the matrix was created with.
    pub fn col_size(&self) -> usize {
        CN
    }
}

#[macro_export]
macro_rules! key_columns {
    (
        $Type:ident,
        $size:literal,
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

impl KeyColumns<$size> for $Type {
    fn size(&self) -> usize {
        $size
    }

    fn enable_column(&mut self, col: usize) -> Result<(), ()> {
        use embedded_hal::digital::OutputPin;
        match col {
            $(
            $index => OutputPin::set_low(&mut self.$col_name).map_err(drop),
            )+
            _ => unreachable!()
        }
    }

    fn disable_column(&mut self, col: usize) -> Result<(), ()> {
        use embedded_hal::digital::OutputPin;
        match col {
            $(
            $index => OutputPin::set_high(&mut self.$col_name).map_err(drop),
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
        $size:literal,
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

impl KeyRows<$size> for $Type {
    fn size(&self) -> usize {
        $size
    }

    fn read_row(&mut self, row: usize) -> Result<bool, ()> {
        use embedded_hal::digital::InputPin;
        match row {
            $(
            $index => InputPin::is_low(&self.$row_name).map_err(drop),
            )+
            _ => unreachable!()
        }
    }
}

// End Macro
};
}


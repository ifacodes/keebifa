use embedded_hal::digital::v2::{InputPin, OutputPin};

/// Matrix Structure
pub struct Matrix<C, R, const CS: usize, const RS: usize>
where
    C: InputPin,
    R: OutputPin,
{
    #[allow(dead_code)]
    cols: [C; CS],
    rows: [R; RS],
}

impl<C, R, const CS: usize, const RS: usize> Matrix<C, R, CS, RS>
where
    C: InputPin,
    R: OutputPin,
{
    pub fn new<Err>(cols: [C; CS], rows: [R; RS]) -> Result<Self, Err>
    where
        C: InputPin<Error = Err>,
        R: OutputPin<Error = Err>,
    {
        let mut result = Self { cols, rows };
        result.clear()?;
        Ok(result)
    }

    pub fn clear<Err>(&mut self) -> Result<(), Err>
    where
        C: InputPin<Error = Err>,
        R: OutputPin<Error = Err>,
    {
        for row in self.rows.iter_mut() {
            row.set_high()?
        }
        Ok(())
    }

    pub fn get<Err>(&mut self) -> Result<[[bool; CS]; RS], Err>
    where
        C: InputPin<Error = Err>,
        R: OutputPin<Error = Err>,
    {
        let mut keys = [[false; CS]; RS];

        // now we set each column to logical 1, and check the rows

        for (y, row) in self.rows.iter_mut().enumerate() {
            row.set_low()?;
            for (x, col) in self.cols.iter().enumerate() {
                keys[x][y] = col.is_low()?;
            }
            row.set_high()?;
        }

        Ok(keys)
    }
}

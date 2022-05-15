use core::iter::once;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use smart_leds::{brightness, colors::*, SmartLedsWrite};

/// Matrix Structure
pub struct Matrix<C, R, const CS: usize, const RS: usize>
where
    C: OutputPin,
    R: InputPin,
{
    cols: [C; CS],
    rows: [R; RS],
}

impl<C, R, const CS: usize, const RS: usize> Matrix<C, R, CS, RS>
where
    C: OutputPin,
    R: InputPin,
{
    pub fn new<Err>(cols: [C; CS], rows: [R; RS]) -> Result<Self, Err>
    where
        C: OutputPin<Error = Err>,
        R: InputPin<Error = Err>,
    {
        let mut result = Self { cols, rows };
        for column in result.cols.iter_mut() {
            column.set_low()?
        }
        Ok(result)
    }

    pub fn poll<S, CL, Err>(&mut self, ws: &mut S) -> Result<[[bool; CS]; RS], Err>
    where
        C: OutputPin<Error = Err>,
        R: InputPin<Error = Err>,
        S: SmartLedsWrite<Color = CL>,
        CL: From<smart_leds::RGB<u8>>,
    {
        // no switches are pressed initially
        let mut result = [[false; CS]; RS];

        // now we set each column to logical 1, and check the rows

        for (x, col) in self.cols.iter_mut().enumerate() {
            col.set_high()?;
            for (y, row) in self.rows.iter().enumerate() {
                if row.is_high()? {
                    result[y][x] = true;
                    ws.write(brightness(once(INDIGO), 32)).ok().unwrap();
                } else {
                    ws.write(brightness(once(BLACK), 32)).ok().unwrap();
                }
            }
        }

        Ok(result)
    }
    // I'm not sure whether to act on each key as it is pressed, or all keys that were pressed. hmm.
}

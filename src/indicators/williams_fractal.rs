use std::fmt;

use crate::{Close, High, Low, Next, Open};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Bill Williams Fractal Indicator
///
/// At time t, reports whether the candlestick at time t-2 is a
/// `Bullish` or `Bearish` fractal with the associated low and high values at time t-2 respectively,
/// or `Neither` if neither `Bullish` nor `Bearish`.
///
/// # Definition
///
/// There are varying definitions, but this one follows the strict definition of
/// consistently increasing/decreasing highs/lows and
/// 3 bullish followed by 2 bearish for `Bearish` and
/// 3 bearish followed by 2 bullish for `Bullish`.
///
/// A `Bullish` fractal at time t-2 is calculated at time t and is defined by:
/// * Low<sub>t-3</sub> < Low<sub>t-4</sub>
/// * Low<sub>t-2</sub> < Low<sub>t-3</sub>
/// * Low<sub>t-1</sub> > Low<sub>t-2</sub>
/// * Low<sub>t</sub> > Low<sub>t-1</sub>
/// * Close<sub>t-4</sub> < Open<sub>t-4</sub>
/// * Close<sub>t-3</sub> < Open<sub>t-3</sub>
/// * Close<sub>t-2</sub> < Open<sub>t-2</sub>
/// * Close<sub>t-1</sub> > Open<sub>t-1</sub>
/// * Close<sub>t</sub> > Open<sub>t</sub>
///
/// A `Bearish` fractal at time t-2 is calculated at time t and is defined by:
/// * High<sub>t-3</sub> > High<sub>t-4</sub>
/// * High<sub>t-2</sub> > High<sub>t-3</sub>
/// * High<sub>t-1</sub> < High<sub>t-2</sub>
/// * High<sub>t</sub> < High<sub>t-1</sub>
/// * Close<sub>t-4</sub> > Open<sub>t-4</sub>
/// * Close<sub>t-3</sub> > Open<sub>t-3</sub>
/// * Close<sub>t-2</sub> > Open<sub>t-2</sub>
/// * Close<sub>t-1</sub> < Open<sub>t-1</sub>
/// * Close<sub>t</sub> < Open<sub>t</sub>
///
/// # Example
///
/// ```
///
/// ```
///
/// # Links
/// * [Currency.com](https://currency.com/how-to-read-and-use-williams-fractal-trading-indicator)

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct WilliamsFractal {
    // store highs, lows, is_bullish in a circular 5-element buffer
    highs: [f64; 5],
    lows: [f64; 5],
    is_bullish: [bool; 5],
    // index to write next latest entry (time t) to
    t_i: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WilliamsFractalType {
    Bearish(f64),
    Bullish(f64),
    Neither,
}

impl WilliamsFractal {
    /// Creates a new `WilliamsFractal` with the last 4 high and low values,
    /// in consecutive order: earliest entries at index 0 and latest at index 3
    pub fn new(
        past_highs: [f64; 4],
        past_lows: [f64; 4],
        past_opens: [f64; 4],
        past_closes: [f64; 4],
    ) -> Self {
        let mut highs = [0.0; 5];
        let mut lows = [0.0; 5];
        let mut is_bullish = [false; 5];
        highs[..4].copy_from_slice(&past_highs);
        lows[..4].copy_from_slice(&past_lows);
        for i in 0..4 {
            is_bullish[i] = past_closes[i] > past_opens[i];
        }
        Self {
            highs,
            lows,
            is_bullish,
            t_i: 4,
        }
    }

    /// Creates a new `WilliamsFractal` with the last known high and low value.
    /// The next 4 entries will always return `Neither`
    pub fn initial(high: f64, low: f64, open: f64, close: f64) -> Self {
        Self {
            highs: [high; 5],
            lows: [low; 5],
            is_bullish: [close > open; 5],
            t_i: 0,
        }
    }

    /// Constructor from array of generics
    /// in consecutive order: earliest entries at index 0 and latest at index 3
    pub fn from_data<T: High + Low + Open + Close>(past: [&T; 4]) -> Self {
        let mut highs = [0.0; 5];
        let mut lows = [0.0; 5];
        let mut is_bullish = [false; 5];
        for i in 0..4 {
            let p = past[i];
            highs[i] = p.high();
            lows[i] = p.low();
            is_bullish[i] = p.close() > p.open();
        }
        Self {
            highs,
            lows,
            is_bullish,
            t_i: 4,
        }
    }

    /// Constructor from initial generic
    /// The next 4 entries will always return `Neither`
    pub fn from_initial<T: High + Low + Open + Close>(initial: &T) -> Self {
        Self {
            highs: [initial.high(); 5],
            lows: [initial.low(); 5],
            is_bullish: [initial.close() > initial.open(); 5],
            t_i: 0,
        }
    }
}

impl<T: High + Low + Open + Close> Next<&T> for WilliamsFractal {
    type Output = WilliamsFractalType;

    fn next(&mut self, input: &T) -> Self::Output {
        let t_i = self.t_i;
        self.highs[t_i] = input.high();
        self.lows[t_i] = input.low();
        self.is_bullish[t_i] = input.close() > input.open();

        let mut indices = [0; 4];
        for i in 1..=4 {
            indices[i - 1] = match t_i >= i {
                true => t_i - i,
                false => 5 - (i - t_i),
            };
        }
        let (t1_i, t2_i, t3_i, t4_i) = (indices[0], indices[1], indices[2], indices[3]);

        let bullish = self.lows[t3_i] < self.lows[t4_i]
            && self.lows[t2_i] < self.lows[t3_i]
            && self.lows[t1_i] > self.lows[t2_i]
            && self.lows[t_i] > self.lows[t1_i]
            && !self.is_bullish[t4_i]
            && !self.is_bullish[t3_i]
            && !self.is_bullish[t2_i]
            && self.is_bullish[t1_i]
            && self.is_bullish[t_i];

        let bearish = self.highs[t3_i] > self.highs[t4_i]
            && self.highs[t2_i] > self.highs[t3_i]
            && self.highs[t1_i] < self.highs[t2_i]
            && self.highs[t_i] < self.highs[t1_i]
            && self.is_bullish[t4_i]
            && self.is_bullish[t3_i]
            && self.is_bullish[t2_i]
            && !self.is_bullish[t1_i]
            && !self.is_bullish[t_i];

        self.t_i = (t_i + 1) % 5;
        if bullish {
            WilliamsFractalType::Bullish(self.lows[t2_i])
        } else if bearish {
            WilliamsFractalType::Bearish(self.highs[t2_i])
        } else {
            WilliamsFractalType::Neither
        }
    }
}

impl fmt::Display for WilliamsFractal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WFRACTAL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    #[test]
    fn test_bullish_basic() {
        let mut wf = WilliamsFractal::new(
            [4.0, 3.0, 2.0, 3.0],
            [3.0, 2.0, 1.0, 2.0],
            [4.0, 3.0, 2.0, 2.0],
            [3.0, 2.0, 1.0, 3.0],
        );
        let bar = Bar::new()
            .open(3.0)
            .close(4.0)
            .high(4.0)
            .low(3.0)
            .volume(0.0);
        assert_eq!(wf.next(&bar), WilliamsFractalType::Bullish(1.0))
    }

    #[test]
    fn test_bearish_basic() {
        let mut wf = WilliamsFractal::new(
            [2.0, 3.0, 4.0, 3.0],
            [1.0, 2.0, 3.0, 2.0],
            [1.0, 2.0, 1.0, 3.0],
            [2.0, 3.0, 2.0, 2.0],
        );
        let bar = Bar::new()
            .open(2.0)
            .close(1.0)
            .high(2.0)
            .low(1.0)
            .volume(0.0);
        assert_eq!(wf.next(&bar), WilliamsFractalType::Bearish(4.0))
    }

    #[test]
    fn test_neither_basic() {
        let mut wf = WilliamsFractal::new(
            [2.0, 3.0, 4.0, 5.0],
            [1.0, 2.0, 3.0, 4.0],
            [1.0, 2.0, 1.0, 4.0],
            [2.0, 3.0, 2.0, 5.0],
        );
        let bar = Bar::new()
            .open(2.0)
            .close(1.0)
            .high(2.0)
            .low(1.0)
            .volume(0.0);
        assert_eq!(wf.next(&bar), WilliamsFractalType::Neither);
    }
}

use std::fmt;

use crate::{High, Low, Next};
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
/// A `Bullish` fractal at time t-2 is calculated at time t and is defined by:
/// * Low<sub>t-2</sub> < Low<sub>t-4</sub>
/// * Low<sub>t-2</sub> < Low<sub>t-3</sub>
/// * Low<sub>t-2</sub> < Low<sub>t-1</sub>
/// * Low<sub>t-2</sub> < Low<sub>t</sub>
///
/// A `Bearish` fractal at time t-2 is calculated at time t and is defined by:
/// * High<sub>t-2</sub> > High<sub>t-4</sub>
/// * High<sub>t-2</sub> > High<sub>t-3</sub>
/// * High<sub>t-2</sub> > High<sub>t-1</sub>
/// * High<sub>t-2</sub> > High<sub>t</sub>
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
    pub fn new(past_highs: [f64; 4], past_lows: [f64; 4]) -> Self {
        let mut highs = [0.0; 5];
        let mut lows = [0.0; 5];
        highs[..4].copy_from_slice(&past_highs);
        lows[..4].copy_from_slice(&past_lows);
        Self {
            highs,
            lows,
            t_i: 4,
        }
    }

    /// Creates a new `WilliamsFractal` with the last known high and low value.
    /// The next 4 entries will always return `Neither`
    pub fn initial(high: f64, low: f64) -> Self {
        Self {
            highs: [high; 5],
            lows: [low; 5],
            t_i: 0,
        }
    }

    /// Constructor from array of generics
    /// in consecutive order: earliest entries at index 0 and latest at index 3
    pub fn from_data<T: High + Low>(past: [&T; 4]) -> Self {
        let mut highs = [0.0; 5];
        let mut lows = [0.0; 5];
        for i in 0..4 {
            let p = past[i];
            highs[i] = p.high();
            lows[i] = p.low();
        }
        Self {
            highs,
            lows,
            t_i: 4,
        }
    }

    /// Constructor from initial generic
    /// The next 4 entries will always return `Neither`
    pub fn from_initial<T: High + Low>(initial: &T) -> Self {
        Self {
            highs: [initial.high(); 5],
            lows: [initial.low(); 5],
            t_i: 0,
        }
    }
}

impl<T: High + Low> Next<&T> for WilliamsFractal {
    type Output = WilliamsFractalType;

    fn next(&mut self, input: &T) -> Self::Output {
        let t_i = self.t_i;
        self.highs[t_i] = input.high();
        self.lows[t_i] = input.low();

        let mut indices = [0; 4];
        for i in 1..=4 {
            indices[i - 1] = match t_i >= i {
                true => t_i - i,
                false => 5 - (i - t_i),
            };
        }
        let (t1_i, t2_i, t3_i, t4_i) = (indices[0], indices[1], indices[2], indices[3]);

        let bullish = self.lows[t2_i] < self.lows[t4_i]
            && self.lows[t2_i] < self.lows[t3_i]
            && self.lows[t2_i] < self.lows[t1_i]
            && self.lows[t2_i] < self.lows[t_i];

        let bearish = self.highs[t2_i] > self.highs[t4_i]
            && self.highs[t2_i] > self.highs[t3_i]
            && self.highs[t2_i] > self.highs[t1_i]
            && self.highs[t2_i] > self.highs[t_i];

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
        let mut wf = WilliamsFractal::new([4.0, 3.0, 2.0, 3.0], [3.0, 2.0, 1.0, 2.0]);
        let bar = Bar::new().high(4.0).low(3.0).volume(0.0);
        assert_eq!(wf.next(&bar), WilliamsFractalType::Bullish(1.0))
    }

    #[test]
    fn test_bearish_basic() {
        let mut wf = WilliamsFractal::new([2.0, 3.0, 4.0, 3.0], [1.0, 2.0, 3.0, 2.0]);
        let bar = Bar::new().high(2.0).low(1.0).volume(0.0);
        assert_eq!(wf.next(&bar), WilliamsFractalType::Bearish(4.0))
    }

    #[test]
    fn test_neither_basic() {
        let mut wf = WilliamsFractal::new([2.0, 3.0, 4.0, 5.0], [1.0, 2.0, 3.0, 4.0]);
        let bar = Bar::new().high(2.0).low(1.0).volume(0.0);
        assert_eq!(wf.next(&bar), WilliamsFractalType::Neither);
    }
}

//! One-unit sources and strict mana costs, using landlord's bipartite matcher.
//! Generic costs accept any source; {C} requires an actual colorless source.
use crate::{Error, bipartite::maximum_bipartite_matching};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}
impl Color {
    fn mask(self) -> u8 {
        1 << self as u8
    }
    fn symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "W" => Some(Self::White),
            "U" => Some(Self::Blue),
            "B" => Some(Self::Black),
            "R" => Some(Self::Red),
            "G" => Some(Self::Green),
            "C" => Some(Self::Colorless),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source(u8);
impl Source {
    pub fn new(colors: &[Color]) -> Result<Self, Error> {
        if colors.is_empty() {
            return Err(Error::Mana("empty source".into()));
        }
        Ok(Self(
            colors.iter().fold(0, |bits, color| bits | color.mask()),
        ))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManaCost {
    generic: usize,
    pips: Vec<u8>,
}
impl ManaCost {
    /// Supports generic, WUBRG, explicit C and two-color hybrid symbols.
    /// X, phyrexian, snow, and two-brid require explicit scenario modeling.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let invalid = || Error::Mana(input.chars().take(128).collect());
        if input.len() > 1024 {
            return Err(invalid());
        }
        let mut cost = Self::default();
        let mut remaining = input;
        while !remaining.is_empty() {
            let tail = remaining.strip_prefix('{').ok_or_else(invalid)?;
            let (symbol, rest) = tail.split_once('}').ok_or_else(invalid)?;
            if !symbol.is_empty() && symbol.bytes().all(|b| b.is_ascii_digit()) {
                let count = symbol.parse::<usize>().map_err(|_| invalid())?;
                cost.generic = cost.generic.checked_add(count).ok_or_else(invalid)?;
            } else if let Some(color) = Color::symbol(symbol) {
                cost.pips.push(color.mask());
            } else if let Some((a, b)) = symbol.split_once('/') {
                let a = Color::symbol(a)
                    .filter(|c| *c != Color::Colorless)
                    .ok_or_else(invalid)?;
                let b = Color::symbol(b)
                    .filter(|c| *c != Color::Colorless)
                    .ok_or_else(invalid)?;
                cost.pips.push(a.mask() | b.mask());
            } else {
                return Err(invalid());
            }
            if cost.mana_value() > 64 {
                return Err(invalid());
            }
            remaining = rest;
        }
        Ok(cost)
    }

    pub fn mana_value(&self) -> usize {
        self.generic + self.pips.len()
    }

    /// Source indices spent to pay this cost, or None if impossible. One source
    /// produces one unit. The caller controls availability, restrictions and taps.
    pub fn payment(&self, sources: &[Source]) -> Option<Vec<usize>> {
        if sources.len() < self.mana_value() {
            return None;
        }
        let n = sources.len();
        let mut edges = Vec::with_capacity(self.pips.len() * n);
        for pip in &self.pips {
            edges.extend(sources.iter().map(|source| u8::from(source.0 & pip != 0)));
        }
        let mut seen = vec![false; n];
        let mut matches = vec![-1; n];
        let paid = maximum_bipartite_matching(&edges, self.pips.len(), n, &mut seen, &mut matches);
        if paid != self.pips.len() {
            return None;
        }
        let mut payment: Vec<_> = matches
            .iter()
            .enumerate()
            .filter_map(|(i, m)| (*m >= 0).then_some(i))
            .collect();
        payment.extend(
            matches
                .iter()
                .enumerate()
                .filter_map(|(i, m)| (*m < 0).then_some(i))
                .take(self.generic),
        );
        Some(payment)
    }
}

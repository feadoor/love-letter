//! Definitions of the cards contained in the game of Love Letter.

/// A single card belonging to a Love Letter deck.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Card {

    /// The Guard, with a value of 1
    Guard = 1,

    /// The Priest, with a value of 2
    Priest = 2,

    /// The Baron, with a value of 3
    Baron = 3,

    /// The Handmaid, with a value of 4
    Handmaid = 4,

    /// The Prince, with a value of 5
    Prince = 5,

    /// The King, with a value of 6
    King = 6,

    /// The Countess, with a value of 7
    Countess = 7,

    /// The Princess, with a value of 8
    Princess = 8,
}

impl Card {

    /// Returns whether or not this `Card` is one whose action has a target.
    ///
    /// # Examples
    ///
    /// ```
    /// # use love_letter::card::Card;
    /// assert!(Card::Guard.has_target());
    /// assert!(!Card::Handmaid.has_target());
    /// ```
    pub fn has_target(self) -> bool {
        match self {
            Self::Guard | Self::Priest | Self::Baron | Self::Prince | Self::King => true,
            _ => false,
        }
    }
}

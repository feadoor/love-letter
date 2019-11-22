//! Definitions of the actions that can be taken in a game of Love Letter.
//!
//! In our terminology, an `Action` is anything carried out on behalf of the players, for example
//! playing a card. Drawing a card is not considered an `Action` - instead, that is viewed as if the
//! player is being dealt a card by the game. Beginning a new game is also an `Action`.

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use crate::card::Card;

/// An external action that can be taken to progress a game of Love Letter.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Action {

    /// A new game is beginning.
    StartGame { players: usize },

    /// One of the players plays a card.
    PlayCard { player_idx: usize, details: PlayCardDetails },
}

/// Details about a play taken by one of the players.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PlayCardDetails {

    /// A Guard has been played, and a guess has been made about another player's card.
    PlayGuard { target_idx: Option<usize>, guess: Card },

    /// A Priest has been played on another player.
    PlayPriest { target_idx: Option<usize> },

    /// A Baron has been played on another player.
    PlayBaron { target_idx: Option<usize> },

    /// A Handmaid has been played.
    PlayHandmaid {},

    // A Prince has been played on one of the players in the game.
    PlayPrince { target_idx: usize },

    /// A King has been played on another player.
    PlayKing { target_idx: Option<usize> },

    /// A Countess has been played.
    PlayCountess {},

    /// A Princess has been played.
    PlayPrincess {},
}

impl PlayCardDetails {

    /// Get the card associated with this play.
    ///
    /// # Examples
    ///
    /// ```
    /// # use love_letter::action::PlayCardDetails;
    /// # use love_letter::card::Card;
    /// let details = PlayCardDetails::PlayHandmaid {};
    /// assert_eq!(details.card(), Card::Handmaid);
    /// ```
    pub fn card(&self) -> Card {
        match self {
            Self::PlayGuard { .. } => Card::Guard,
            Self::PlayPriest { .. } => Card::Priest,
            Self::PlayBaron { .. } => Card::Baron,
            Self::PlayHandmaid { .. } => Card::Handmaid,
            Self::PlayPrince { .. } => Card::Prince,
            Self::PlayKing { .. } => Card::King,
            Self::PlayCountess { .. } => Card::Countess,
            Self::PlayPrincess { .. } => Card::Princess,
        }
    }

    /// Get the target associated with this play.
    ///
    /// # Examples
    ///
    /// ```
    /// # use love_letter::action::PlayCardDetails;
    /// # use love_letter::card::Card;
    /// let baron_details = PlayCardDetails::PlayBaron { target_idx: Some(1) };
    /// let handmaid_details = PlayCardDetails::PlayHandmaid {};
    /// assert_eq!(baron_details.target(), Some(1));
    /// assert_eq!(handmaid_details.target(), None);
    /// ```
    pub fn target(&self) -> Option<usize> {
        match self {
            Self::PlayGuard { target_idx, .. } => *target_idx,
            Self::PlayPriest { target_idx } => *target_idx,
            Self::PlayBaron { target_idx } => *target_idx,
            Self::PlayPrince { target_idx } => Some(*target_idx),
            Self::PlayKing { target_idx } => *target_idx,
            _ => None,
        }
    }
}

//! A representation of a single player in a game of Love Letter.

use crate::card::Card;

/// A player in a game of Love Letter.
#[derive(Clone, Debug)]
pub struct Player {

    /// The cards currently held in this player's hand.
    hand: Vec<Card>,

    /// The cards that have previously been discarded by this player.
    discards: Vec<Card>,

    /// Whether this player is currently protected by the effect of the Handmaid.
    protected: bool,

    /// Whether this player is still active (i.e. has not been eliminated from the game).
    active: bool,
}

impl Player {

    /// A new player at the start of a game.
    pub fn new() -> Self {
        Player {
            hand: Vec::new(),
            discards: Vec::new(),
            protected: false,
            active: true,
        }
    }

    /// Deal a card to this player.
    pub fn give_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    /// Take the specified card from this player.
    pub fn play_card(&mut self, card: Card) -> Result<(), ()> {
        match self.hand.iter().position(|&c| c == card) {
            Some(index) => {
                self.hand.remove(index);
                self.discards.push(card);
                Ok(())
            }
            None => Err(()),
        }
    }

    /// Check if this player is holding a particular card.
    pub fn is_holding_card(&self, card: Card) -> bool {
        self.hand.contains(&card)
    }

    /// Get the single card that this player is holding.
    pub fn card(&self) -> Option<Card> {
        if self.hand.len() == 1 { self.hand.first().map(|c| *c) } else { None }
    }

    /// Get the single card that this player is holding and remove it from their hand.
    pub fn take_card(&mut self) -> Option<Card> {
        let card = self.card();
        if let Some(card) = card { 
            self.discards.push(card);
            self.hand.clear();
        }
        card
    }

    /// Get the total value of the cards this player has discarded
    pub fn value_of_discards(&self) -> u32 {
        self.discards.iter().map(|&c| c as u32).sum()
    }

    /// Check if this player is currently protected by a Handmaid.
    pub fn protected(&self) -> bool {
        self.protected
    }

    /// Protect this player.
    pub fn make_protected(&mut self) {
        self.protected = true;
    }

    /// Remove this player's protection.
    pub fn make_unprotected(&mut self) {
        self.protected = false;
    }

    /// Check if this player is currently active in the game.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Eliminate this player.
    pub fn eliminate(&mut self) {
        self.active = false;
    }
}

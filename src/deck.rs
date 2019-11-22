//! A deck structure used as part of the game engine.

use rand::prelude::*;

use crate::card::Card;

/// A Love Letter deck.
#[derive(Clone, Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {

    /// Returns a new `Deck` with the cards in a fixed default order.
    pub fn new() -> Self {
        Self {
            cards: vec![
                Card::Guard, Card::Guard, Card::Guard, Card::Guard, Card::Guard,
                Card::Priest, Card::Priest,
                Card::Baron, Card::Baron,
                Card::Handmaid, Card::Handmaid,
                Card::Prince, Card::Prince,
                Card::King,
                Card::Countess,
                Card::Princess,
            ],
        }
    }

    /// Checks if the deck is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Shuffles the cards in this `Deck` into a random order.
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    /// Draws the top card from the `Deck` and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

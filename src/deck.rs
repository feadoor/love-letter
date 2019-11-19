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

    /// Shuffles the cards in this `Deck` into a random order.
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    /// Draws the top card from the `Deck` and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Exposes the cards in the `Deck` as a slice of `Card`s.
    #[cfg(test)]
    pub fn as_slice(&self) -> &[Card] {
        &self.cards
    }
}

#[cfg(test)]
mod test {

    use super::{Card, Deck};

    #[test]
    fn test_deck_has_expected_cards() {

        // Grab all of the cards from the deck
        let deck = Deck::new();
        let all_cards = deck.as_slice().to_vec();

        // Check that each card appears the right number of times
        let expected_counts = vec![
            (Card::Guard, 5), (Card::Priest, 2), (Card::Baron, 2), (Card::Handmaid, 2),
            (Card::Prince, 2), (Card::King, 1), (Card::Countess, 1), (Card::Princess, 1),
        ];

        for (card, expected_count) in expected_counts {
            let actual_count = all_cards.iter().filter(|&&c| c == card).count();
            assert_eq!(
                actual_count, expected_count,
                "Expected {} copies of {:?}, found {}",
                expected_count, card, actual_count,
            );
        }
    }
}

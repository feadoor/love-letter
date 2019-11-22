//! Definitions of the events that can occur in a game of Love Letter.
//!
//! In our terminology, an event is anything that happens as a result of an `Action` and which
//! players of the game might reasonably expect to be informed of. For example, as a result of
//! one player playing the Baron, the following events may occur before another action is required:
//!
//!   - Player 1 plays a Baron
//!   - Players 1 and 2 compare hands
//!   - Player 2 is eliminated
//!   - Player 2 reveals a King
//!   - Player 3 draws a card
//!
//! There are also "no-op" events that don't represent any concrete occurrence in the game, but can
//! be used by players to easily keep track of the flow of the game. Some examples would be:
//!
//!   - Game starts
//!   - Players join the game
//!   - It is player X's turn
//!   - Game ends

use crate::card::Card;

/// An event that happens as a result of an action in a game of Love Letter.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Event {

    /// A new game begins.
    NewGame { players: usize },

    /// A player joins the game.
    RegisterPlayer { player_idx: usize },

    /// A card is burned from the top of the deck.
    BurnCard {},

    /// Additional cards are publicly removed from the top of the deck.
    RemoveCardFromGame { card: Card },

    /// A card is dealt to one of the players.
    DealCard { player_idx: usize, card: Card },

    /// One of the players needs to play a card.
    ReadyToPlay { player_idx: usize },

    /// One of the players plays a card from their hand.
    PlayCard { player_idx: usize, card: Card },

    /// One of the players has a guess made about their card.
    Guess { target_idx: usize, guess: Card },

    /// One of the players shows their card to another player.
    ShowCard { player_idx: usize, target_idx: usize, card: Card },

    /// Two players compare their hands.
    CompareHands { player_idx: usize, player_card: Card, target_idx: usize, target_card: Card },

    /// One of the players is forced to discard a card from their hand.
    DiscardCard { target_idx: usize, card: Card },

    /// Two players swap hands.
    SwapHands { player_idx: usize, player_card: Card, target_idx: usize, target_card: Card },

    /// A player is eliminated from the game.
    EliminatePlayer { player_idx: usize },

    /// One of the players reveals their card after being eliminated.
    RevealCard { player_idx: usize, card: Card },

    /// The game ends and the winners are announced.
    GameOver { winner_indices: Vec<usize> },
}

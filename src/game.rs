//! An engine that is capable of arbitrating over a whole game of Love Letter.
//!
//! This game engine operates on a system of "actions" and "events". An action is initiated
//! externally to the game structure itself - for example, requesting that a new game begins,
//! or a player playing a card. An event is something that happens as a result of an action being
//! taken, and is initiated by the game engine itself - for example, a player being eliminated.
//!
//! This engine uses these two concepts in a very simple way - ask it to take an action, and it
//! will return a list of events that happened as a result of that action. Users of this engine
//! are responsible for correctly interpreting those events.

use std::error::Error;
use std::fmt;

use crate::action::{Action, PlayCardDetails};
use crate::card::Card;
use crate::deck::Deck;
use crate::event::Event;
use crate::player::Player;

/// An engine capable of playing a whole game of Love Letter.
#[derive(Clone, Debug)]
pub struct Game {

    /// The deck for this game.
    deck: Deck,

    /// The burned card for this game.
    burned_card: Option<Card>,

    /// The players in the game.
    players: Vec<Player>,

    /// The player whose turn it is to play.
    turn_counter: usize,

    /// The high-level state of the game.
    state: GameState,
}

impl Game {

    /// Create a new game which has not yet started.
    ///
    /// # Examples
    ///
    /// ```
    /// # use love_letter::game::Game;
    /// let game = Game::new();
    /// ```
    pub fn new() -> Self {
        Game {
            deck: Deck::new(),
            burned_card: None,
            players: Vec::new(),
            turn_counter: 0,
            state: GameState::NotStarted,
        }
    }

    /// Carry out the given action on the game, returning all of the events which occur as a result.
    ///
    /// The first action that _must_ be carried out a new `Game` is a `StartGame` action, in order
    /// to shuffle the deck and set up the players. Any other action is invalid before `StartGame`
    /// has been carried out.
    ///
    /// # Examples
    ///
    /// ```
    /// # use love_letter::action::Action;
    /// # use love_letter::game::Game;
    /// let mut game = Game::new();
    /// let events = game.perform_action(&Action::StartGame { players: 2 });
    /// ```
    pub fn perform_action(&mut self, action: &Action) -> Result<Vec<Event>, GameError> {
        match action {
            Action::StartGame { players } => self.start_game(*players),
            Action::PlayCard { player_idx, details } => self.play_card(*player_idx, details),
        }
    }

    /// Start a new game with the given number of players.
    fn start_game(&mut self, players: usize) -> Result<Vec<Event>, GameError> {

        // Check that the number of players is legal for a game of Love Letter.
        if players < 2 || players > 4 {
            return Err(GameError::InvalidNumberOfPlayers(players));
        }

        // The events that will result from this action being carried out.
        let mut events = vec![Event::NewGame { players }];

        // Reset the deck and shuffle it.
        self.deck = Deck::new();
        self.deck.shuffle();

        // Register players with the game.
        self.players.clear();
        for player_idx in 0..players {
            self.players.push(Player::new());
            events.push(Event::RegisterPlayer { player_idx });
        }

        // Burn a card from the top of the deck.
        self.burned_card = self.deck.pop();
        events.push(Event::BurnCard {});

        // In a two player game, additionally discard three cards from the deck.
        if players == 2 {
            for _ in 0..3 {
                events.push(Event::RemoveCardFromGame {
                    card: self.deck.pop().unwrap(),
                })
            }
        }

        // Deal a card to each player
        for player_idx in 0..players {
            events.push(self.draw_and_give_card_to_player(player_idx));
        }

        // Deal an additional card to the first player and inform them that they are ready to play
        events.push(self.draw_and_give_card_to_player(0));
        events.push(self.start_player_turn(0));

        // Set the game as in-progress and return the events that have occurred back to the caller.
        self.state = GameState::InProgress;
        Ok(events)
    }

    /// Determine the events resulting from a player playing a card.
    fn play_card(&mut self, player_idx: usize, details: &PlayCardDetails) -> Result<Vec<Event>, GameError> {

        use PlayCardDetails::*;

        // Events resulting from this action.
        let mut events = Vec::new();

        // Check that a game is in progress.
        self.is_game_in_progress()?;

        // Perform common checks - is it this player's turn, have they chosen a valid target, and 
        // are they holding the card they want to play? If so, remove it from their hand.
        self.does_player_exist(player_idx)?;
        self.is_it_players_turn(player_idx)?;
        self.is_target_valid(player_idx, details.target(), details.card())?;
        self.is_player_allowed_to_play_card(player_idx, details.card())?;
        events.push(self.play_card_from_player_hand(player_idx, details.card())?);

        // Describe the events specifically resulting from the play of the card.
        events.append(&mut match details {
            PlayGuard { target_idx: Some(target_idx), guess } => self.play_guard(*target_idx, *guess),
            PlayPriest { target_idx: Some(target_idx) } => self.play_priest(player_idx, *target_idx),
            PlayBaron { target_idx: Some(target_idx) } => self.play_baron(player_idx, *target_idx),
            PlayHandmaid {} => self.play_handmaid(player_idx),
            PlayPrince { target_idx } => self.play_prince(*target_idx),
            PlayKing { target_idx: Some(target_idx) } => self.play_king(player_idx, *target_idx),
            PlayCountess {} => self.play_countess(),
            PlayPrincess {} => self.play_princess(player_idx),
            _ => Ok(Vec::new()),
        }?);

        // Check if there is only one player left standing
        let active_players = self.active_players();
        if active_players.len() == 1 {
            events.push(self.end_game_with_winners(active_players));
        }

        // Otherwise, check if the deck is empty
        else if self.deck.is_empty() {
            let winners = self.calculate_winners();
            events.push(self.end_game_with_winners(winners));
        }

        // Otherwise, it's the next player's turn
        else {
            let next_player = self.next_player();
            events.push(self.draw_and_give_card_to_player(next_player));
            events.push(self.start_player_turn(next_player));
        }

        Ok(events)
    }

    /// Determine the events resulting from a player playing a Guard.
    fn play_guard(&mut self, target_idx: usize, guess: Card) -> Result<Vec<Event>, GameError> {

        // The target has a guess made about their hand.
        let mut events = vec![Event::Guess { target_idx, guess }];

        // Record that the target player is eliminated, and reveal the card in their hand.
        if self.players[target_idx].is_holding_card(guess) {
            events.push(self.eliminate_player(target_idx));
            events.push(self.reveal_eliminated_player_card(target_idx));
        }

        Ok(events)
    }

    /// Determine the events resulting from a player playing a Priest.
    fn play_priest(&mut self, player_idx: usize, target_idx: usize) -> Result<Vec<Event>, GameError> {

        // The target must show their hand.
        let card = self.players[target_idx].card().unwrap();
        Ok(vec![Event::ShowCard { player_idx, target_idx, card }])
    }

    /// Determine the events resulting from a player playing a Baron.
    fn play_baron(&mut self, player_idx: usize, target_idx: usize) -> Result<Vec<Event>, GameError> {

        // The player and the target compare their hands.
        let player_card = self.players[player_idx].card().unwrap();
        let target_card = self.players[target_idx].card().unwrap();
        let mut events = vec![Event::CompareHands { player_idx, player_card, target_idx, target_card }];

        // The loser is eliminated from the game
        if let Some(losing_player_idx) = if player_card < target_card { Some(player_idx) } else if target_card < player_card { Some(target_idx) } else { None } {
            events.push(self.eliminate_player(losing_player_idx));
            events.push(self.reveal_eliminated_player_card(losing_player_idx));
        }

        Ok(events)
    }

    /// Determine the events resulting from a player playing a Handmaid.
    fn play_handmaid(&mut self, player_idx: usize) -> Result<Vec<Event>, GameError> {
        self.players[player_idx].make_protected();
        Ok(Vec::new())
    }

    /// Determine the events resulting from a player playing a Prince.
    fn play_prince(&mut self, target_idx: usize) -> Result<Vec<Event>, GameError> {

        // The target is forced to discard their hand.
        let card = self.players[target_idx].card().unwrap();
        let mut events = vec![self.discard_hand(target_idx)];

        // If the discarded card is the Princess, then the player is eliminated.
        if card == Card::Princess {
            events.push(self.eliminate_player(target_idx));
        }

        // Otherwise, draw a new card and give it to the player.
        else {
            events.push(self.draw_and_give_card_to_player(target_idx));
        }

        Ok(events)
    }

    /// Determine the events resulting from a player playing a King.
    fn play_king(&mut self, player_idx: usize, target_idx: usize) -> Result<Vec<Event>, GameError> {
        
        // The target and the player swap their hands
        let player_card = self.players[player_idx].take_card().unwrap();
        let target_card = self.players[target_idx].take_card().unwrap();
        self.players[player_idx].give_card(target_card);
        self.players[target_idx].give_card(player_card);
        Ok(vec![Event::SwapHands { player_idx, player_card, target_idx, target_card }])
    }

    /// Determine the events resulting from a player playing a Countess.
    fn play_countess(&mut self) -> Result<Vec<Event>, GameError> {
        Ok(Vec::new())
    }

    /// Determine the events resulting from a player playing a Princess.
    fn play_princess(&mut self, player_idx: usize) -> Result<Vec<Event>, GameError> {

        // The player is immediately eliminated
        Ok(vec![self.eliminate_player(player_idx)])
    }

    /// Check that the game is currently in progress.
    fn is_game_in_progress(&self) -> Result<(), GameError> {
        match self.state {
            GameState::NotStarted | GameState::Complete => Err(GameError::GameNotInProgress),
            GameState::InProgress => Ok(()),
        }
    }

    /// Check that it is the given player's turn
    fn is_it_players_turn(&self, player_idx: usize) -> Result<(), GameError> {
        if self.turn_counter == player_idx {
            Ok(())
        } else {
            Err(GameError::PlayedOutOfTurn(player_idx))
        }
    }

    /// Check that the player exists.
    fn does_player_exist(&self, player_idx: usize) -> Result<(), GameError> {
        if player_idx >= self.players.len() {
            Err(GameError::PlayerDoesNotExist(player_idx))
        } else {
            Ok(())
        }
    }

    /// Check the the player is allowed to play a particular card.
    fn is_player_allowed_to_play_card(&self, player_idx: usize, card: Card) -> Result<(), GameError> {

        // The Prince and King cannot be played if the player also holds a Countess
        if card == Card::Prince || card == Card::King {
            if self.players[player_idx].is_holding_card(Card::Countess) {
                return Err(GameError::CannotPlayWhileHoldingCountess(card));
            }
        }

        Ok(())
    }

    /// Check that the target player exists and is allowed to be targeted by the given card.
    fn is_target_valid(&self, player_idx: usize, target_idx: Option<usize>, card: Card) -> Result<(), GameError> {

        // If there are unprotected targets and this card is one that requires a target, then a
        // target must be given.
        if card.has_target() {
            let unprotected_targets = self.unprotected_targets(player_idx, card == Card::Prince);
            if !unprotected_targets.is_empty() && target_idx.is_none() {
                return Err(GameError::MustProvideTarget(card));
            }
        }

        // Targeting oneself is only possible when the card being played is the Prince.
        if card != Card::Prince && target_idx == Some(player_idx) {
            return Err(GameError::CannotTargetSelf(card));
        }

        // Targeting a non-existent player is not allowed.
        if let Some(idx) = target_idx {
            if idx >= self.players.len() {
                return Err(GameError::PlayerDoesNotExist(idx));
            }
        }

        // Targeting a protected player is not allowed.
        if let Some(idx) = target_idx {
            if self.players[idx].protected() {
                return Err(GameError::CannotTargetProtectedPlayer);
            }
        }

        // Targeting an eliminated player is not allowed.
        if let Some(idx) = target_idx {
            if !self.players[idx].active() {
                return Err(GameError::CannotTargetEliminatedPlayer);
            }
        }

        Ok(())
    }

    /// Check that the player is holding the card they want to play, and remove it from their hand
    /// if they are.
    fn play_card_from_player_hand(&mut self, player_idx: usize, card: Card) -> Result<Event, GameError> {
        match self.players[player_idx].play_card(card) {
            Ok(()) => Ok(Event::PlayCard { player_idx, card }),
            Err(()) => Err(GameError::PlayerDoesNotHaveCard(player_idx, card)),
        }
    }

    /// Draw a card, deal it to a player, and return an `Event` summarising as much.
    fn draw_and_give_card_to_player(&mut self, player_idx: usize) -> Event {

        // Use the deck if possible, or the burned card if the deck is empty.
        let card = self.deck.pop().or(self.burned_card).take().unwrap();
        self.players[player_idx].give_card(card);
        Event::DealCard { player_idx, card }
    }

    /// Begin a player's turn.
    fn start_player_turn(&mut self, player_idx: usize) -> Event {
        self.turn_counter = player_idx;
        self.players[player_idx].make_unprotected();
        Event::ReadyToPlay { player_idx }
    }

    /// Discard the given player's hand.
    fn discard_hand(&mut self, target_idx: usize) -> Event {
        Event::DiscardCard { target_idx, card: self.players[target_idx].take_card().unwrap() }
    }

    /// Eliminate the given player from the game and return an `Event` to represent this.
    fn eliminate_player(&mut self, player_idx: usize) -> Event {
        self.players[player_idx].eliminate();
        Event::EliminatePlayer { player_idx }
    }

    /// End the game with the given player as the winner.
    fn end_game_with_winners(&mut self, winner_indices: Vec<usize>) -> Event {
        self.state = GameState::Complete;
        Event::GameOver { winner_indices }
    }

    /// Reveal the final card from an eliminated player's hand.
    fn reveal_eliminated_player_card(&mut self, player_idx: usize) -> Event {
        Event::RevealCard { player_idx, card: self.players[player_idx].take_card().unwrap() }
    }

    /// Get the players who are potential targets for an action.
    fn unprotected_targets(&self, player_idx: usize, include_self: bool) -> Vec<usize> {
        let unprotected_players = (0..self.players.len())
            .filter(|&idx| self.players[idx].active() && !self.players[idx].protected());
        if include_self {
            unprotected_players.collect()
        } else {
            unprotected_players.filter(|&idx| idx != player_idx).collect()
        }
    }

    /// Get the players who are still active in the game
    fn active_players(&self) -> Vec<usize> {
        (0..self.players.len()).filter(|&idx| self.players[idx].active()).collect()
    }

    /// Work out whose turn it should be next in the game
    fn next_player(&self) -> usize {
        let mut player_idx = (self.turn_counter + 1) % self.players.len();
        while !self.players[player_idx].active() {
            player_idx = (self.turn_counter + 1) % self.players.len();
        }
        player_idx
    }

    /// Given that the deck is empty but no player has won outright, determine the winners
    fn calculate_winners(&self) -> Vec<usize> {

        // Find the players who are still in the game
        let active_players = self.active_players();

        // Calculate each player's effective score, consisting of the card they hold and the total
        // value of their discarded cards throughout the game
        let mut scores = active_players.iter().map(|&idx| 
            (self.players[idx].card().unwrap(), self.players[idx].value_of_discards(), idx)
        ).collect::<Vec<_>>();

        // Sort the scores and return each player who has the highest score
        scores.sort(); let high_score = scores[0];
        scores.iter().filter(|s| s.0 == high_score.0 && s.1 == high_score.1).map(|s| s.2).collect()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

/// An enum representing the possible high-level states of a game of Love Letter.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum GameState {

    /// The game has not yet started.
    NotStarted,

    /// The game is in progress
    InProgress,

    /// The game has finished
    Complete,
}

/// An error type representing the possible reasons that a game action might fail.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum GameError {

    /// Attempted to start a game with an invalid number of players.
    InvalidNumberOfPlayers(usize),

    /// Tried to play a card when the game was not in progress.
    GameNotInProgress,

    /// Referenced a player who doesn't exist in the current game.
    PlayerDoesNotExist(usize),

    /// Tried to play out of turn.
    PlayedOutOfTurn(usize),

    /// Tried to play a card that the player was not holding.
    PlayerDoesNotHaveCard(usize, Card),

    // Did not provide a target when there were valid targets available.
    MustProvideTarget(Card),

    /// Tried to target oneself using a card that is not the Prince.
    CannotTargetSelf(Card),

    /// Tried to target a protected player.
    CannotTargetProtectedPlayer,

    // Tried to target an eliminated player.
    CannotTargetEliminatedPlayer,

    // Tried to illegally play the Prince or King while holding the Countess
    CannotPlayWhileHoldingCountess(Card),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use GameError::*;
        match self {
            InvalidNumberOfPlayers(players) => write!(f, "Invalid number of players: {}. There must be between 2 and 4 players in a game.", players),
            GameNotInProgress => write!(f, "No game is in progress."),
            PlayerDoesNotExist(player) => write!(f, "Player {} does not exist.", player),
            PlayedOutOfTurn(player) => write!(f, "It is not Player {}'s turn", player),
            PlayerDoesNotHaveCard(player, card) => write!(f, "Player {} is not holding a {:?}.", player, card),
            MustProvideTarget(card) => write!(f, "You must provide a target when playing the {:?}.", card),
            CannotTargetSelf(card) => write!(f, "You cannot target yourself when playing the {:?}.", card),
            CannotTargetProtectedPlayer => write!(f, "You cannot target a protected player."),
            CannotTargetEliminatedPlayer => write!(f, "You cannot target an eliminated player."),
            CannotPlayWhileHoldingCountess(card) => write!(f, "You cannot player the {:?} while holding the Countess", card),
        }
    }
}

impl Error for GameError {}

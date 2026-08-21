const MAX_BET_DIGITS: usize = 19;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CardHandPhase {
    #[default]
    Covered,
    Betting,
    Playing,
}

/// UI-independent state shared by casino tables.
///
/// Wallet persistence remains an application concern. This type only owns the
/// wager input and legal card-table phase transitions.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct CasinoState {
    bet: u64,
    bet_input: String,
    total_wager: u64,
    bet_committed: bool,
    card_phase: CardHandPhase,
}

impl CasinoState {
    pub fn reset_round(&mut self) {
        *self = Self::default();
    }

    pub fn bet(&self) -> u64 {
        self.bet
    }

    pub fn bet_input(&self) -> &str {
        &self.bet_input
    }

    pub fn total_wager(&self) -> u64 {
        self.total_wager
    }

    pub fn bet_committed(&self) -> bool {
        self.bet_committed
    }

    pub fn card_phase(&self) -> CardHandPhase {
        self.card_phase
    }

    pub fn push_bet_digit(&mut self, digit: char) -> bool {
        if !digit.is_ascii_digit() || self.bet_input.len() >= MAX_BET_DIGITS {
            return false;
        }
        if self.bet_input == "0" {
            self.bet_input.clear();
        }
        self.bet_input.push(digit);
        self.sync_bet();
        true
    }

    pub fn backspace_bet(&mut self) {
        self.bet_input.pop();
        self.sync_bet();
    }

    pub fn open_card_betting(&mut self) {
        if self.card_phase == CardHandPhase::Covered {
            self.card_phase = CardHandPhase::Betting;
        }
    }

    pub fn start_card_hand(&mut self) {
        if self.card_phase == CardHandPhase::Betting && self.bet_committed {
            self.card_phase = CardHandPhase::Playing;
        }
    }

    pub fn commit_initial_wager(&mut self) {
        debug_assert!(self.bet > 0);
        if !self.bet_committed {
            self.total_wager = self.bet;
            self.bet_committed = true;
        }
    }

    pub fn add_wager(&mut self, amount: u64) {
        self.total_wager = self.total_wager.saturating_add(amount);
    }

    fn sync_bet(&mut self) {
        self.bet = self.bet_input.parse().unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wager_input_stays_numeric_and_resets_between_rounds() {
        let mut state = CasinoState::default();
        assert!(!state.push_bet_digit('x'));
        assert!(state.push_bet_digit('0'));
        assert!(state.push_bet_digit('5'));
        assert_eq!(state.bet(), 5);
        assert_eq!(state.bet_input(), "5");

        state.commit_initial_wager();
        state.add_wager(25);
        assert_eq!(state.total_wager(), 30);

        state.reset_round();
        assert_eq!(state, CasinoState::default());
    }

    #[test]
    fn card_hand_can_only_start_after_betting_and_commit() {
        let mut state = CasinoState::default();
        state.start_card_hand();
        assert_eq!(state.card_phase(), CardHandPhase::Covered);

        state.open_card_betting();
        state.start_card_hand();
        assert_eq!(state.card_phase(), CardHandPhase::Betting);

        state.push_bet_digit('1');
        state.commit_initial_wager();
        state.start_card_hand();
        assert_eq!(state.card_phase(), CardHandPhase::Playing);
    }
}

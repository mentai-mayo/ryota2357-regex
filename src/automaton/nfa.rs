use std::collections::{HashMap, HashSet};

use crate::parser::Node;

/// NFAの状態
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub(crate) struct NFAState(pub u32);

pub(crate) struct Context {
    states: u32,
}

impl Context {
    fn new() -> Self {
        Context { states: 0 }
    }

    pub(crate) fn new_state(&mut self) -> NFAState {
        let id: u32 = self.states;
        self.states += 1;
        NFAState(id)
    }
}

/// NondeterministicFiniteAutomaton
#[allow(clippy::upper_case_acronyms)]
pub(crate) struct NFA {
    /// 開始状態
    pub start: NFAState,
    /// 受理状態
    pub accepts: HashSet<NFAState>,
    /// 遷移テーブル
    pub(crate) transition: HashMap<NFAState, HashMap<Option<char>, HashSet<NFAState>>>,
}

impl NFA {
    pub(crate) fn new(start: NFAState, accepts: HashSet<NFAState>) -> Self {
        NFA {
            start,
            accepts,
            transition: HashMap::new(),
        }
    }

    pub(crate) fn next_chars(&self, state: NFAState) -> HashSet<Option<char>> {
        self.transition
            .get(&state)
            .map(|table| table.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub(crate) fn next_states(&self, state: NFAState, chara: Option<char>) -> HashSet<NFAState> {
        self.transition
            .get(&state)
            .and_then(|table| table.get(&chara))
            .cloned()
            .unwrap_or(HashSet::new())
    }

    pub(crate) fn add_transition(mut self, from: NFAState, chara: char, to: NFAState) -> Self {
        self._insert_transition(from, to, Some(chara));
        self
    }

    pub(crate) fn add_empty_transition(mut self, from: NFAState, to: NFAState) -> Self {
        self._insert_transition(from, to, None);
        self
    }

    pub(crate) fn merge_transition(mut self, other: &Self) -> Self {
        for (from, trans) in &other.transition {
            for (chara, to) in trans {
                self.transition
                    .entry(*from)
                    .or_default()
                    .entry(*chara)
                    .or_default()
                    .extend(to);
            }
        }
        self
    }

    fn _insert_transition(&mut self, from: NFAState, to: NFAState, chara: Option<char>) {
        let states = self
            .transition
            .entry(from)
            .or_default()
            .entry(chara)
            .or_default();
        states.insert(to);
    }

    pub(crate) fn from_node(node: Node) -> Self {
        node.assemble(&mut Context::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context() {
        let mut context = Context::new();
        assert_eq!(context.new_state(), NFAState(0));
        assert_eq!(context.new_state(), NFAState(1));
        assert_eq!(context.new_state(), NFAState(2));
    }
}

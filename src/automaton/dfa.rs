use std::collections::{HashMap, HashSet};

use crate::automaton::{NFA, NFAState};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct DFAState(u32);

struct Context {
    states: u32,
    statemap: HashMap<Vec<NFAState>, DFAState>,
}

impl Context {
    fn new() -> Self {
        Context {
            states: 0,
            statemap: HashMap::new(),
        }
    }

    fn get_state(&mut self, states: &[NFAState]) -> DFAState {
        let mut sorted_states: Vec<NFAState> = states.to_vec();
        sorted_states.sort();
        match self.statemap.get(&sorted_states) {
            Some(state) => *state,
            None => {
                let id: u32 = self.states;
                self.states += 1;
                self.statemap.insert(sorted_states, DFAState(id));
                DFAState(id)
            }
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub(crate) struct DFA {
    pub(crate) start: DFAState,
    pub(crate) accepts: HashSet<DFAState>,
    transition: HashMap<(DFAState, char), DFAState>,
}

impl DFA {
    pub(crate) fn next_state(&self, state: DFAState, chara: char) -> Option<DFAState> {
        self.transition.get(&(state, chara)).cloned()
    }

    pub(crate) fn from_nfa(nfa: NFA) -> Self {
        let mut context: Context = Context::new();

        // start: DFASの開始状態 (DFAState)
        // start_states: NFAとしての開始状態集合 (Vec<NFAState>)
        let (start, start_states) = {
            let mut ret = vec![nfa.start];
            let mut stack = nfa
                .next_states(nfa.start, None)
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            while let Some(state) = stack.pop() {
                ret.push(state);
                let next: HashSet<NFAState> = nfa.next_states(state, None);
                stack.extend(next.iter().filter(|s| !ret.contains(s)).cloned());
            }
            (context.get_state(&ret), ret)
        };

        // 遷移テーブル
        let transition: HashMap<(DFAState, char), DFAState> = {
            let mut ret: HashMap<(DFAState, char), DFAState> = HashMap::new();
            let mut waiting: Vec<Vec<NFAState>> = vec![start_states];
            let mut visited: HashSet<DFAState> = HashSet::new();
            while let Some(look_states) = waiting.pop() {
                visited.insert(context.get_state(&look_states));

                // Collect states that can be transitioned from the current state (look_states).
                // transition_map[char] = The set of states that can be transitioned by `char`.
                let mut transition_map: HashMap<char, HashSet<NFAState>> = HashMap::new();
                for look_state in &look_states {
                    for chara in nfa.next_chars(*look_state).iter().filter_map(|c| *c) {
                        let mut next_states: Vec<NFAState> = nfa
                            .next_states(*look_state, Some(chara))
                            .into_iter()
                            .chain(nfa.next_states(*look_state, None))
                            .collect::<Vec<_>>();
                        let mut stack: Vec<NFAState> = next_states
                            .iter()
                            .filter(|s| !nfa.next_states(**s, None).is_empty())
                            .cloned()
                            .collect::<Vec<_>>();
                        while let Some(state) = stack.pop() {
                            let next: HashSet<NFAState> = nfa.next_states(state, None);
                            stack.extend(next.iter().filter(|s| !next_states.contains(s)).cloned());
                            next_states.extend(next);
                        }
                        transition_map.entry(chara).or_default().extend(next_states);
                    }
                }
                let from: DFAState = context.get_state(&look_states);
                for (chara, next_states) in transition_map {
                    let next_states_vec: Vec<NFAState> = next_states.iter().cloned().collect();
                    let to: DFAState = context.get_state(&next_states_vec);
                    if !visited.contains(&to) {
                        waiting.push(next_states.into_iter().collect());
                    }
                    ret.insert((from, chara), to);
                }
            }
            ret
        };

        // 受理状態 (HashSet<DFAState>)
        let accepts = {
            let mut ret: HashSet<DFAState> = HashSet::new();
            for (nfa_states, dfa_state) in context.statemap {
                if nfa_states.iter().any(|s| nfa.accepts.contains(s)) {
                    ret.insert(dfa_state);
                }
            }
            ret
        };

        DFA {
            start,
            accepts,
            transition,
        }
    }
}

#[cfg(test)] #[rustfmt::skip]
mod tests {
    use super::*;

    #[test]
    fn dfa_context() {
        let mut context = Context::new();
        assert_eq!(context.get_state(&[ NFAState(0) ]),                           DFAState(0));
        assert_eq!(context.get_state(&[ NFAState(0), NFAState(1) ]),              DFAState(1));
        assert_eq!(context.get_state(&[ NFAState(1), NFAState(2), NFAState(3) ]), DFAState(2));
        assert_eq!(context.get_state(&[ ]),                                       DFAState(3));
        assert_eq!(context.get_state(&[ NFAState(0) ]),                           DFAState(0));
        assert_eq!(context.get_state(&[ NFAState(2), NFAState(1), NFAState(3) ]), DFAState(2));
        assert_eq!(context.get_state(&[ NFAState(4) ]),                           DFAState(4));
    }

    #[test]
    fn dfa_from_nfa_simple() {
        // -> 0 --a--> 1
        // accept: 1
        let dfa = DFA::from_nfa(NFA::new(NFAState(0), [NFAState(1)].into()).add_transition(
            NFAState(0),
            'a',
            NFAState(1),
        ));

        // -> 0 --a--> 1
        // accept: 1
        assert_eq!(dfa.start, DFAState(0));
        assert_eq!(dfa.accepts, [DFAState(1)].into());
        assert_eq!(dfa.transition.len(), 1);
        assert_eq!(dfa.transition[&(DFAState(0), 'a')], DFAState(1));
    }

    #[test]
    fn dfa_from_nfa_simple_concat() {
        // -> 0 --a--> 1 --b--> 2
        // accept: 2
        let dfa = DFA::from_nfa(
            NFA::new(NFAState(0), [NFAState(2)].into())
                .add_transition(NFAState(0), 'a', NFAState(1))
                .add_transition(NFAState(1), 'b', NFAState(2)),
        );

        // -> 0 --a--> 1 --b--> 2
        // accept: 2
        assert_eq!(dfa.start, DFAState(0));
        assert_eq!(dfa.accepts, [DFAState(2)].into());
        assert_eq!(dfa.transition.len(), 2);
        assert_eq!(dfa.transition[&(DFAState(0), 'a')], DFAState(1));
        assert_eq!(dfa.transition[&(DFAState(1), 'b')], DFAState(2));
    }

    #[test]
    fn dfa_from_nfa_simple_union() {
        //     /--ε--> 1 --a--> 2
        // -> 0
        //     \--ε--> 3 --b--> 4
        // accept: 2, 4
        let dfa = DFA::from_nfa(
            NFA::new(NFAState(0), [NFAState(2), NFAState(4)].into())
                .add_empty_transition(NFAState(0), NFAState(1))
                .add_empty_transition(NFAState(0), NFAState(3))
                .add_transition(NFAState(1), 'a', NFAState(2))
                .add_transition(NFAState(3), 'b', NFAState(4)),
        );

        //     /--a--> 1 (or 2)
        // -> 0
        //     \--b--> 2 (or 1)
        // accept: 1, 2
        assert_eq!(dfa.start, DFAState(0));
        assert_eq!(dfa.accepts, [DFAState(1), DFAState(2)].into());
        assert_eq!(dfa.transition.len(), 2);
        if dfa.transition[&(DFAState(0), 'a')] == DFAState(1) {
            assert_eq!(dfa.transition[&(DFAState(0), 'b')], DFAState(2));
        } else {
            assert_eq!(dfa.transition[&(DFAState(0), 'a')], DFAState(2));
            assert_eq!(dfa.transition[&(DFAState(0), 'b')], DFAState(1));
        }
    }

    #[test]
    fn dfa_from_nfa_simple_star() {
        // -> 0 --ε--> 1 --a--> 2
        //              \<--ε--/
        // accept: 0, 2
        let dfa = DFA::from_nfa(
            NFA::new(NFAState(0), [NFAState(0), NFAState(2)].into())
                .add_empty_transition(NFAState(0), NFAState(1))
                .add_transition(NFAState(1), 'a', NFAState(2))
                .add_empty_transition(NFAState(2), NFAState(1)),
        );

        // -> 0 --a--> 1
        //            / \
        //           /   \
        //           <-a-/
        // accept: 0, 1
        assert_eq!(dfa.start, DFAState(0));
        assert_eq!(dfa.accepts, [DFAState(0), DFAState(1)].into());
        assert_eq!(dfa.transition.len(), 2);
        assert_eq!(dfa.transition[&(DFAState(0), 'a')], DFAState(1));
        assert_eq!(dfa.transition[&(DFAState(1), 'a')], DFAState(1));
    }

    #[test]
    fn dfa_from_nfa_complex() {
        // -> 0 --x--> 1
        //            /
        //    /<--ε---
        //    |         /<--ε--\
        //    4 --ε--> 2 --y--> 3
        //    \                /
        //     \        /<--ε--
        //      --ε--> 5 --z--> 6
        // accept: 6
        let dfa = DFA::from_nfa(
            NFA::new(NFAState(0), [NFAState(6)].into())
                .add_transition(NFAState(0), 'x', NFAState(1))
                .add_empty_transition(NFAState(1), NFAState(2))
                .add_empty_transition(NFAState(1), NFAState(5))
                .add_transition(NFAState(2), 'y', NFAState(3))
                .add_transition(NFAState(5), 'z', NFAState(6))
                .add_empty_transition(NFAState(3), NFAState(2))
                .add_empty_transition(NFAState(3), NFAState(5)),
        );

        // -> 0 --x--> 1
        //            / \
        //    /<--y---   ---z-->\
        //    |                 |
        //    2 -------z------> 3
        //   / \
        //  /   \
        //  <-y-/
        // accept: 3
        // NOTE: 2 and 3 can be swapped
        assert_eq!(dfa.start, DFAState(0));
        assert_eq!(dfa.transition.len(), 5);
        let (s2, s3) = if dfa.accepts == [DFAState(3)].into() {
            (2, 3)
        } else {
            (3, 2)
        };
        assert_eq!(dfa.transition[&(DFAState(0), 'x')], DFAState(1));
        assert_eq!(dfa.transition[&(DFAState(1), 'y')], DFAState(s2));
        assert_eq!(dfa.transition[&(DFAState(1), 'z')], DFAState(s3));
        assert_eq!(dfa.transition[&(DFAState(s2), 'z')], DFAState(s3));
        assert_eq!(dfa.transition[&(DFAState(s2), 'y')], DFAState(s2));
    }
}

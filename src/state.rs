use std::collections::HashSet;
use std::hash::Hash;

use crate::input::Input;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

pub type Card = usize;

pub type Pile = Vec<Card>;

#[derive(Clone, Debug, Default)]
pub struct PokemonSlot
{
    pub cards: Pile,
    pub damage: u8,
}

pub type Slots<T> = Vec<T>;

#[derive(Clone, Debug, Default)]
pub struct PrizeCard
{
    pub card: Card,
    pub is_face_up: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CardLayout
{
    pub slots: Slots<PokemonSlot>,
    pub hand: Pile,
    pub discard: Pile,
    // Last = top of deck
    pub deck: Pile,
    pub lost_zone: Pile,
    pub prizes: Vec<PrizeCard>,

    // "Why is this a pile and why are there two of them?"
    // you may ask. Well, it's way easier this way lol
    pub stadium: Pile,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Selection
{
    Slot
    {
        slot_index: usize, pokemon_index: Option<usize>
    },
    Hand
    {
        index: usize
    },
    Discard
    {
        index: usize
    },
    LostZone
    {
        index: usize
    },
    Prize
    {
        index: usize
    },
    Stadium
    {
        index: usize
    },
}

impl Default for Selection
{
    fn default() -> Self { Selection::Slot { slot_index: 0, pokemon_index: None } }
}

impl Selection
{
    fn deepest_index(&self) -> Option<usize>
    {
        match self {
            Selection::Slot { pokemon_index, .. } => pokemon_index.clone(),
            Selection::Hand { index }
            | Selection::Discard { index }
            | Selection::LostZone { index }
            | Selection::Prize { index }
            | Selection::Stadium { index } => Some(*index),
        }
    }

    fn change_deepest_index(&self, change: i32) -> Self
    {
        let apply = |u: &usize| {
            if *u == 0 && change < 0 {
                *u
            } else {
                ((*u as i32) + change) as usize
            }
        };
        match self {
            Selection::Slot { pokemon_index: Some(u), slot_index } => {
                Selection::Slot { slot_index: *slot_index, pokemon_index: Some(apply(u)) }
            },
            Selection::Hand { index } => Selection::Hand { index: apply(index) },
            Selection::Discard { index } => Selection::Discard { index: apply(index) },
            Selection::LostZone { index } => Selection::LostZone { index: apply(index) },
            Selection::Prize { index } => Selection::Prize { index: apply(index) },
            Selection::Stadium { index } => Selection::Stadium { index: apply(index) },
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SelectingState<T: Clone>
{
    pub selected: HashSet<T>,
    pub current_highlight: T,
}

impl<T: Clone + Eq + Hash> SelectingState<T>
{
    fn change_highlight(&self, new_highlight: T) -> Self
    {
        let mut s = (*self).clone();
        s.current_highlight = new_highlight;
        s
    }

    fn clear_selected(&self) -> Self
    {
        let mut s = (*self).clone();
        s.selected = HashSet::new();
        s
    }

    fn add_to_selection(&self) -> Self
    {
        let mut s = (*self).clone();
        s.selected.insert(self.current_highlight.clone());
        s
    }
}

pub type LayoutSelectingState = SelectingState<Selection>;

pub type PileSelectingState = SelectingState<usize>;

#[derive(Clone, Debug)]
pub enum PreviousMovingState
{
    Selecting(LayoutSelectingState),
    DeckSearch(PileSelectingState),
    Look(PileSelectingState),
}

#[derive(Clone, Debug)]
pub enum MoveAwaitedInput
{
    Any,
    SlotSpecific
    {
        slot: usize,
    },
}

#[derive(Clone, Debug)]
pub enum InputMode
{
    Selecting(LayoutSelectingState),
    Deck,
    DeckSearch(PileSelectingState),
    Look(PileSelectingState),
    Move
    {
        awaited_input: MoveAwaitedInput,
        previous_state: PreviousMovingState,
    },
    Swap
    {
        first_slot: Option<usize>,
    },
}

#[derive(Clone, Debug)]
pub enum Player
{
    Player1,
    Player2,
}

#[derive(Clone, Debug)]
pub enum UIAlert
{
    Shuffled,
    Roll(u8),
}

#[derive(Clone, Debug)]
pub struct State
{
    pub player1_layout: CardLayout,
    pub player2_layout: CardLayout,
    pub currently_viewing: Player,
    pub input_mode: InputMode,
    pub ui_alert: Option<UIAlert>,
    // To disallow making States without the make function
    #[allow(dead_code)]
    made: (),
}

impl State
{
    pub fn make(deck1: Pile, deck2: Pile) -> Self
    {
        assert_eq!(deck1.len(), 60);
        assert_eq!(deck2.len(), 60);
        let empty_slots: Slots<PokemonSlot> = vec![PokemonSlot { cards: vec![], damage: 0 }; 6];
        State {
            player1_layout: CardLayout {
                deck: deck1,
                slots: empty_slots.clone(),
                ..Default::default()
            },
            player2_layout: CardLayout { deck: deck2, slots: empty_slots, ..Default::default() },
            currently_viewing: Player::Player1,
            input_mode: InputMode::Selecting(Default::default()),
            ui_alert: None,
            made: (),
        }
    }

    /// Shuffle decks and put up 6 prizes.
    pub fn setup(&mut self)
    {
        let setup_layout = |layout: &mut CardLayout| {
            let mut rng = thread_rng();
            layout.deck.shuffle(&mut rng);
            for _ in 0..6 {
                if let Some(top_card) = layout.deck.pop() {
                    layout.prizes.push(PrizeCard { card: top_card, is_face_up: false });
                }
            }
        };
        setup_layout(&mut self.player1_layout);
        setup_layout(&mut self.player2_layout);
    }

    fn current_layout(&self) -> &CardLayout
    {
        match self.currently_viewing {
            Player::Player1 => &self.player1_layout,
            Player::Player2 => &self.player2_layout,
        }
    }

    fn current_layout_mut(&mut self) -> &mut CardLayout
    {
        match self.currently_viewing {
            Player::Player1 => &mut self.player1_layout,
            Player::Player2 => &mut self.player2_layout,
        }
    }

    fn highlighted_list_length(&self, h: &Selection) -> Option<usize>
    {
        let layout = self.current_layout();
        match h {
            Selection::Slot { slot_index, .. } => Some(layout.slots[*slot_index].cards.len()),
            Selection::Hand { .. } => Some(layout.hand.len()),
            Selection::Discard { .. } => Some(layout.discard.len()),
            Selection::LostZone { .. } => Some(layout.lost_zone.len()),
            Selection::Prize { .. } => Some(layout.prizes.len()),
            Selection::Stadium { .. } => Some(layout.stadium.len()),
        }
    }

    pub fn card_at(&self, selection: &Selection) -> Option<Card>
    {
        let layout = self.current_layout();
        match selection {
            Selection::Hand { index } => Some(layout.hand[*index]),
            Selection::Slot { slot_index, pokemon_index } => {
                pokemon_index.map(|pi| layout.slots[*slot_index].cards[pi])
            },
            Selection::Discard { index } => Some(layout.discard[*index]),
            Selection::Prize { index } => {
                let prize = &layout.prizes[*index];
                if prize.is_face_up {
                    Some(prize.card)
                } else {
                    None
                }
            },
            Selection::LostZone { index } => Some(layout.lost_zone[*index]),
            Selection::Stadium { index } => Some(layout.stadium[*index]),
        }
    }

    pub fn deck_card_at(&self, selection: usize) -> Option<Card>
    {
        let layout = self.current_layout();
        layout.deck.get(selection).map(|u| *u)
    }

    pub fn update(&mut self, input: &Input)
    {
        self.ui_alert = None;

        use Input as I;

        let leave_unchanged = self.input_mode.clone();

        let pile_change = |f: &dyn Fn(&CardLayout) -> usize, sel, st: &LayoutSelectingState| {
            if f(self.current_layout()) > 0 {
                let st2 = st.change_highlight(sel);
                InputMode::Selecting(st2)
            } else {
                // = leave unchanged
                self.input_mode.clone()
            }
        };

        let next_input_mode = match &self.input_mode {
            InputMode::Selecting(st) => match input {
                I::Left => {
                    let h = st.current_highlight.change_deepest_index(-1);
                    InputMode::Selecting(st.change_highlight(h))
                },
                I::Right => {
                    let l = self.highlighted_list_length(&st.current_highlight);
                    let i = st.current_highlight.deepest_index();
                    match (l, i) {
                        (Some(l), Some(i)) => {
                            if i + 1 < l {
                                let h = st.current_highlight.change_deepest_index(1);
                                InputMode::Selecting(st.change_highlight(h))
                            } else {
                                leave_unchanged
                            }
                        },
                        _ => leave_unchanged,
                    }
                },

                I::Slot(u) => {
                    let slots = &self.current_layout().slots;
                    if *u > slots.len() || *u == 0 {
                        leave_unchanged
                    } else {
                        let adjusted_u = u - 1;
                        let pokemon_index =
                            if slots[adjusted_u].cards.len() == 0 { None } else { Some(0) };
                        let h = Selection::Slot { slot_index: adjusted_u, pokemon_index };
                        InputMode::Selecting(st.change_highlight(h))
                    }
                },

                I::Hand => pile_change(&|l| l.hand.len(), Selection::Hand { index: 0 }, st),
                I::Discard => {
                    pile_change(&|l| l.discard.len(), Selection::Discard { index: 0 }, st)
                },
                I::Prizes => pile_change(&|l| l.prizes.len(), Selection::Prize { index: 0 }, st),
                I::LostZone => {
                    pile_change(&|l| l.lost_zone.len(), Selection::LostZone { index: 0 }, st)
                },
                I::Stadium => {
                    pile_change(&|l| l.stadium.len(), Selection::Stadium { index: 0 }, st)
                },

                I::Flip => {
                    let mut indices_to_flip = Vec::new();
                    for selection in st.selected.iter() {
                        match *selection {
                            Selection::Prize { index } => {
                                indices_to_flip.push(index);
                            },
                            _ => (),
                        }
                    }
                    let layout = self.current_layout_mut();
                    for index in indices_to_flip {
                        layout.prizes[index].is_face_up = !layout.prizes[index].is_face_up;
                    }
                    leave_unchanged
                },

                I::Increment | I::Decrement => {
                    let mut slots_to_affect = Vec::new();
                    for selection in st.selected.iter() {
                        match *selection {
                            Selection::Slot { slot_index, pokemon_index: Some(_) } => {
                                slots_to_affect.push(slot_index);
                            },
                            _ => (),
                        }
                    }

                    let f = match input {
                        I::Increment => |u| u + 1,
                        I::Decrement => |u| if u > 0 { u - 1 } else { u },
                        _ => unreachable!(),
                    };

                    let layout = self.current_layout_mut();
                    for slot in slots_to_affect {
                        layout.slots[slot].damage = f(layout.slots[slot].damage);
                    }
                    leave_unchanged
                },

                I::Select => InputMode::Selecting(st.add_to_selection()),

                I::Cancel => InputMode::Selecting(st.clear_selected()),

                I::Move => InputMode::Move {
                    awaited_input: MoveAwaitedInput::Any,
                    previous_state: PreviousMovingState::Selecting(st.clone()),
                },

                I::Swap => InputMode::Swap { 
                    first_slot: None, 
                },

                I::Deck => InputMode::Deck,

                I::SwitchSides => {
                    self.currently_viewing = match &self.currently_viewing {
                        Player::Player1 => Player::Player2,
                        Player::Player2 => Player::Player1,
                    };
                    InputMode::Selecting(LayoutSelectingState::default())
                },

                I::Roll => {
                    let mut rng = thread_rng();
                    let die_roll = rng.gen_range(1..=6);
                    self.ui_alert = Some(UIAlert::Roll(die_roll));
                    leave_unchanged
                },

                _ => leave_unchanged,
            },
            InputMode::Deck => match input {
                I::Cancel => {
                    let selecting_state = Default::default();
                    InputMode::Selecting(selecting_state)
                },

                I::Deck => {
                    // DRAW CARD!!!
                    let layout = self.current_layout_mut();
                    if let Some(top_card) = layout.deck.pop() {
                        layout.hand.push(top_card);
                    }
                    InputMode::Deck
                },

                I::Observe => {
                    todo!()
                },

                I::Select => {
                    let selected = HashSet::new();
                    let deck_search_st = SelectingState { selected, current_highlight: 0 };
                    InputMode::DeckSearch(deck_search_st)
                },

                I::Shuffle => {
                    let layout = self.current_layout_mut();
                    let mut rng = thread_rng();
                    layout.deck.shuffle(&mut rng);
                    self.ui_alert = Some(UIAlert::Shuffled);
                    leave_unchanged
                },

                I::Slot(u) => {
                    //let st: SelectingState<Selection> = Default::default();
                    let slots = &self.current_layout().slots;
                    if *u > slots.len() || *u == 0 {
                        leave_unchanged
                    } else {
                        let adjusted_u = u - 1;
                        let pokemon_index =
                            if slots[adjusted_u].cards.len() == 0 { None } else { Some(0) };
                        let h = Selection::Slot { slot_index: adjusted_u, pokemon_index };
                        InputMode::Selecting(LayoutSelectingState::default().change_highlight(h))
                    }
                },

                I::Hand => pile_change(
                    &|l| l.hand.len(),
                    Selection::Hand { index: 0 },
                    &LayoutSelectingState::default(),
                ),
                I::Discard => pile_change(
                    &|l| l.discard.len(),
                    Selection::Discard { index: 0 },
                    &LayoutSelectingState::default(),
                ),
                I::Prizes => pile_change(
                    &|l| l.prizes.len(),
                    Selection::Prize { index: 0 },
                    &LayoutSelectingState::default(),
                ),
                I::LostZone => pile_change(
                    &|l| l.lost_zone.len(),
                    Selection::LostZone { index: 0 },
                    &LayoutSelectingState::default(),
                ),
                I::Stadium => pile_change(
                    &|l| l.stadium.len(),
                    Selection::Stadium { index: 0 },
                    &LayoutSelectingState::default(),
                ),

                _ => leave_unchanged,
            },
            InputMode::DeckSearch(st) => match input {
                I::Cancel => InputMode::Deck,
                I::Left => {
                    let new_highlight =
                        if st.current_highlight == 0 { 0 } else { st.current_highlight - 1 };
                    InputMode::DeckSearch(st.change_highlight(new_highlight))
                },
                I::Right => {
                    let layout = self.current_layout();
                    let new_highlight = if st.current_highlight >= layout.deck.len() {
                        st.current_highlight
                    } else {
                        st.current_highlight + 1
                    };
                    InputMode::DeckSearch(st.change_highlight(new_highlight))
                },
                I::Select => InputMode::DeckSearch(st.add_to_selection()),
                I::Move => InputMode::Move {
                    awaited_input: MoveAwaitedInput::Any,
                    previous_state: PreviousMovingState::DeckSearch(st.clone()),
                },
                _ => leave_unchanged,
            },
            InputMode::Look(_st) => {
                todo!()
            },
            InputMode::Move { awaited_input, previous_state } => {
                // Dont 4get that removing from lists makes scawy index problems :3
                // Could be worth it to just make a new list for everything that gets
                // moved to avoid this problem since performance isnt rly
                // that affected by this i think
                self.handle_move(awaited_input.clone(), previous_state.clone(), input)
            },

            InputMode::Swap { first_slot } => match input {
                I::Cancel => {
                    let selecting_state = Default::default();
                    InputMode::Selecting(selecting_state)
                },
                I::Slot(slot) => {
                    if *slot == 0 || *slot > self.current_layout().slots.len() {
                        leave_unchanged
                    }
                    else {
                        if let Some(first_slot) = first_slot {
                            let first_slot = *first_slot - 1;
                            let second_slot = *slot - 1;

                            let mut slots = &mut self.current_layout_mut().slots;
                            slots.swap(first_slot, second_slot);

                            let selecting_state = Default::default();
                            InputMode::Selecting(selecting_state)
                        }
                        else {
                            InputMode::Swap { first_slot: Some(*slot) }
                        }
                    }
                },
                _ => leave_unchanged,
            },
        };
        self.input_mode = next_input_mode;
    }

    fn handle_move(
        &mut self,
        awaited_input: MoveAwaitedInput,
        previous_state: PreviousMovingState,
        input: &Input,
    ) -> InputMode
    {
        use Input as I;
        if let I::Cancel = input {
            return match previous_state {
                PreviousMovingState::Selecting(st) => InputMode::Selecting(st),
                PreviousMovingState::DeckSearch(st) => InputMode::DeckSearch(st),
                PreviousMovingState::Look(st) => InputMode::Look(st),
            };
        }

        let leave_unchanged = InputMode::Move {
            awaited_input: awaited_input.clone(),
            previous_state: previous_state.clone(),
        };

        if let I::Slot(slot) = input {
            if *slot == 0 || *slot > self.current_layout().slots.len() {
                return leave_unchanged;
            }
            return InputMode::Move {
                awaited_input: MoveAwaitedInput::SlotSpecific { slot: *slot - 1 },
                previous_state,
            };
        }

        if let MoveAwaitedInput::SlotSpecific { .. } = awaited_input {
            match input {
                I::Append | I::Prepend => (),
                _ => return leave_unchanged,
            }
        } else {
            match input {
                I::Top | I::Bottom | I::Hand | I::Discard | I::LostZone | I::Stadium => (),
                _ => return leave_unchanged,
            }
        }

        let layout = self.current_layout_mut();

        let mut moving_cards = Vec::new();
        let output = match previous_state {
            PreviousMovingState::Selecting(mut st) => {
                st.selected.insert(st.current_highlight);
                let mut selections_to_move: Vec<Selection> = st.selected.into_iter().collect();
                // Need to do removal in reverse to avoid index issues
                selections_to_move.sort();
                while let Some(selection) = selections_to_move.pop() {
                    let card_opt = match selection {
                        Selection::Slot { slot_index, pokemon_index } => {
                            pokemon_index.map(|pi| layout.slots[slot_index].cards.remove(pi))
                        },
                        Selection::Hand { index } => Some(layout.hand.remove(index)),
                        Selection::Prize { index } => Some(layout.prizes.remove(index).card),
                        Selection::Discard { index } => Some(layout.discard.remove(index)),
                        Selection::LostZone { index } => Some(layout.lost_zone.remove(index)),
                        Selection::Stadium { index } => Some(layout.stadium.remove(index)),
                    };
                    if let Some(card) = card_opt {
                        moving_cards.push(card);
                    }
                }
                InputMode::Selecting(Default::default())
            },
            PreviousMovingState::DeckSearch(mut st) => {
                st.selected.insert(st.current_highlight);
                let mut indices_to_move: Vec<usize> = st.selected.into_iter().collect();
                // Need to do removal in reverse to avoid index issues
                indices_to_move.sort();
                while let Some(index) = indices_to_move.pop() {
                    moving_cards.push(layout.deck.remove(index));
                }
                InputMode::DeckSearch(Default::default())
            },
            PreviousMovingState::Look(_) => {
                todo!()
            },
        };

        let (destination, should_prepend): (&mut Pile, bool) =
            if let MoveAwaitedInput::SlotSpecific { slot } = awaited_input {
                let should_prepend = match input {
                    I::Append => false,
                    I::Prepend => true,
                    _ => unreachable!(),
                };
                let destination = &mut layout.slots[slot].cards;
                (destination, should_prepend)
            } else {
                match input {
                    I::Top => (&mut layout.deck, false),
                    I::Bottom => (&mut layout.deck, true),
                    I::Hand => (&mut layout.hand, false),
                    I::Discard => (&mut layout.discard, false),
                    I::LostZone => (&mut layout.lost_zone, false),
                    I::Stadium => (&mut layout.stadium, false),
                    _ => unreachable!(),
                }
            };

        if should_prepend {
            for card in moving_cards.into_iter() {
                destination.insert(0, card)
            }
        } else {
            destination.append(&mut moving_cards);
        }
        output
    }
}

use crate::display_constants::*;
use sdl2::{libc::DIR, pixels::Color, rect::Rect, render::Texture, sys::div};

use crate::{load_cards, state};

type Renderer = sdl2::render::Canvas<sdl2::video::Window>;

macro_rules! rect {
    ($x:expr, $y:expr, $w:expr, $h:expr) => {
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    };
}

#[derive(Clone, Copy)]
enum CardDisplaySize
{
    Small,
    Large,
}

impl CardDisplaySize
{
    fn dims(self) -> (u32, u32)
    {
        match self {
            CardDisplaySize::Small => (CARD_SMALL_DISPLAY_WIDTH, CARD_SMALL_DISPLAY_HEIGHT),
            CardDisplaySize::Large => (CARD_LARGE_DISPLAY_WIDTH, CARD_LARGE_DISPLAY_HEIGHT),
        }
    }
}

#[derive(Clone, Copy)]
enum Side
{
    Facing,
    Opposing,
}

impl Side
{
    fn y(&self, y: i32) -> i32
    {
        match self {
            Side::Facing => (WINDOW_HEIGHT as i32) - (CARD_SMALL_DISPLAY_HEIGHT as i32) - y,
            Side::Opposing => y,
        }
    }
}

pub fn draw(
    canvas: &mut Renderer,
    st: &state::State,
    card_textures: &Vec<Texture>,
) -> Result<(), String>
{
    let (facing_layout, opposing_layout, facing_color, opposing_color) = match &st
        .currently_viewing
    {
        state::Player::Player1 => {
            (&st.player1_layout, &st.player2_layout, PLAYER1_SLEEVE_COLOR, PLAYER2_SLEEVE_COLOR)
        },
        state::Player::Player2 => {
            (&st.player2_layout, &st.player1_layout, PLAYER2_SLEEVE_COLOR, PLAYER1_SLEEVE_COLOR)
        },
    };
    draw_layout(canvas, facing_layout, Side::Facing, facing_color, card_textures)?;
    draw_layout(canvas, opposing_layout, Side::Opposing, opposing_color, card_textures)?;

    // DRAW HIGHLIGHT (AND DECK/DISCARD SEARCH IF APPLICABLE)
    use state::InputMode as IM;
    match &st.input_mode {
        IM::Selecting(st2) => {
            if let state::Selection::Discard { .. } = &st2.current_highlight {
                draw_deck_and_discard_search(&facing_layout.discard, canvas, card_textures)?;
            }

            highlight_selection(&st2.current_highlight, HIGHLIGHT_COLOR, &facing_layout, canvas)?;
            if let Some(card) = st.card_at(&st2.current_highlight) {
                draw_focused_card(card, canvas, card_textures)?;
            }
            for sel in st2.selected.iter() {
                highlight_selection(&sel, SELECTED_COLOR, &facing_layout, canvas)?;
            }
        },
        IM::Deck => {
            let (x, y) = deck_location(Side::Facing);
            highlight_card_at(x, y, HIGHLIGHT_COLOR, canvas)?;
        },
        IM::DeckSearch(st2) => {
            draw_deck_and_discard_search(&facing_layout.deck, canvas, card_textures)?;

            let (x, y) =
                deck_and_discard_card_location(st2.current_highlight, facing_layout.deck.len());
            highlight_card_at(x, y, HIGHLIGHT_COLOR, canvas)?;
            if let Some(card) = st.deck_card_at(st2.current_highlight) {
                draw_focused_card(card, canvas, card_textures)?;
            }
            for sel in st2.selected.iter() {
                let (x, y) = deck_and_discard_card_location(*sel, facing_layout.deck.len());
                highlight_card_at(x, y, SELECTED_COLOR, canvas)?;
            }
        },
        IM::Look(_) => (),
        IM::Move { .. } => (),
        IM::Swap { .. } => (),
    }

    match &st.ui_alert {
        Some(state::UIAlert::Roll(v)) => {
            let x = (WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH - DICE_ROLL_MAT_SIDELEN) / 2;
            let y = (WINDOW_HEIGHT - DICE_ROLL_MAT_SIDELEN) / 2;
            canvas.set_draw_color(DICE_ROLL_MAT_COLOR);
            canvas.fill_rect(rect!(x, y, DICE_ROLL_MAT_SIDELEN, DICE_ROLL_MAT_SIDELEN))?;
            let die_x = (WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH - DICE_SCALE * 7) / 2;
            let die_y = (WINDOW_HEIGHT - DICE_SCALE * 7) / 2;
            draw_die(die_x as i32, die_y as i32, *v, canvas)?;
        },
        Some(state::UIAlert::Shuffled) => {
            let x = (WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH - DICE_ROLL_MAT_SIDELEN) / 2;
            let y = (WINDOW_HEIGHT - DICE_ROLL_MAT_SIDELEN) / 2;
            canvas.set_draw_color(DICE_ROLL_MAT_COLOR);
            canvas.fill_rect(rect!(x, y, DICE_ROLL_MAT_SIDELEN, DICE_ROLL_MAT_SIDELEN))?;
        },
        _ => (),
    }

    Ok(())
}

fn draw_layout(
    canvas: &mut Renderer,
    layout: &state::CardLayout,
    side: Side,
    sleeve_color: Color,
    card_textures: &Vec<Texture>,
) -> Result<(), String>
{
    // DRAW HAND
    let hand_len = layout.hand.len();
    for (i, card) in layout.hand.iter().enumerate() {
        let (x, y) = hand_card_location(i, hand_len, side);
        if let Side::Facing = side {
            draw_card(*card, x, y, canvas, card_textures)?;
        } else {
            draw_flipped_card(x, y, sleeve_color, canvas)?;
        }
    }

    // DRAW PRIZES
    for (i, prize_card) in layout.prizes.iter().enumerate() {
        let (x, y) = prize_card_location(i, side);
        if prize_card.is_face_up {
            draw_card(prize_card.card, x, y, canvas, card_textures)?;
        } else {
            draw_flipped_card(x, y, sleeve_color, canvas)?;
        }
    }

    // DRAW IN-PLAY POKEMON
    for (i, slot) in layout.slots.iter().enumerate() {
        for (j, card) in slot.cards.iter().enumerate().rev() {
            let (x, y) = slot_card_location(i, j, side);
            draw_card(*card, x, y, canvas, card_textures)?;
            if j == 0 {
                draw_damage_counters(
                    x + SPACE_BETWEEN_DICE as i32,
                    y + SPACE_BETWEEN_DICE as i32,
                    side,
                    slot.damage,
                    canvas,
                )?;
            }
        }
    }

    // DRAW DECK
    let (deck_x, deck_y) = deck_location(side);
    draw_flipped_card(deck_x, deck_y, sleeve_color, canvas)?;

    let mut draw_face_up_pile = |x, y, pile: &Vec<state::Card>| {
        if pile.len() > 0 {
            let top_card = pile[pile.len() - 1];
            draw_card(top_card, x, y, canvas, card_textures)
        } else {
            Ok(())
        }
    };

    // DRAW DISCARD
    let (discard_x, discard_y) = discard_location(side);
    draw_face_up_pile(discard_x, discard_y, &layout.discard)?;

    // DRAW STADIUM
    let (stadium_x, stadium_y) = stadium_location(side);
    draw_face_up_pile(stadium_x, stadium_y, &layout.stadium)?;

    // DRAW LOST ZONE
    let (lost_zone_x, lost_zone_y) = lost_zone_location(side);
    draw_face_up_pile(lost_zone_x, lost_zone_y, &layout.lost_zone)?;

    Ok(())
}

fn draw_deck_and_discard_search(
    cards: &Vec<usize>,
    canvas: &mut Renderer,
    card_textures: &Vec<Texture>,
) -> Result<(), String>
{
    let background_y = (WINDOW_HEIGHT - SEARCH_HIGHLIGHT_HEIGHT) / 2;
    let effective_width = WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH;
    canvas.set_draw_color(SEARCH_HIGHLIGHT_COLOR);
    canvas.fill_rect(rect!(0, background_y, effective_width, SEARCH_HIGHLIGHT_HEIGHT))?;

    for (i, card) in cards.iter().enumerate() {
        let (x, y) = deck_and_discard_card_location(i, cards.len());
        draw_card(*card, x, y, canvas, card_textures)?;
    }

    Ok(())
}

fn draw_card(
    card: usize,
    x: i32,
    y: i32,
    canvas: &mut Renderer,
    card_textures: &Vec<Texture>,
) -> Result<(), String>
{
    let tex = &card_textures[card];
    //let (src_w, src_h) = card_indexer.get_dimensions(card);
    let (dst_w, dst_h) = CardDisplaySize::Small.dims();
    canvas.copy(tex, None /*rect!(0, 0, src_w, src_h)*/, rect!(x, y, dst_w, dst_h))
}

fn draw_focused_card(
    card: usize,
    canvas: &mut Renderer,
    card_textures: &Vec<Texture>,
) -> Result<(), String>
{
    let tex = &card_textures[card];
    let x = WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH;
    let y = 0;
    //let (src_w, src_h) = card_indexer.get_dimensions(card);
    let (dst_w, dst_h) = CardDisplaySize::Large.dims();
    canvas.copy(tex, None /*rect!(0, 0, src_w, src_h)*/, rect!(x, y, dst_w, dst_h))
}

fn draw_flipped_card(
    x: i32,
    y: i32,
    sleeve_color: Color,
    canvas: &mut Renderer,
) -> Result<(), String>
{
    canvas.set_draw_color(sleeve_color);
    let (w, h) = CardDisplaySize::Small.dims();
    canvas.fill_rect(rect!(x, y, w, h))
}

/*fn draw_card_list(
    x: i32,
    y: i32,
*/

fn highlight_card_at(x: i32, y: i32, color: Color, canvas: &mut Renderer) -> Result<(), String>
{
    canvas.set_draw_color(color);

    let ht = HIGHLIGHT_THICKNESS as i32;
    let x0 = x - ht;
    let y0 = y - ht;
    let x1 = x + (CARD_SMALL_DISPLAY_WIDTH as i32);
    let y1 = y + (CARD_SMALL_DISPLAY_HEIGHT as i32);
    let long_height = CARD_SMALL_DISPLAY_HEIGHT + 2 * HIGHLIGHT_THICKNESS;
    let long_width = CARD_SMALL_DISPLAY_WIDTH + 2 * HIGHLIGHT_THICKNESS;

    canvas.fill_rect(rect!(x0, y0, long_width, HIGHLIGHT_THICKNESS))?;
    canvas.fill_rect(rect!(x0, y0, HIGHLIGHT_THICKNESS, long_height))?;
    canvas.fill_rect(rect!(x1, y0, HIGHLIGHT_THICKNESS, long_height))?;
    canvas.fill_rect(rect!(x0, y1, long_width, HIGHLIGHT_THICKNESS))?;

    Ok(())
}

fn highlight_selection(
    selection: &state::Selection,
    color: Color,
    layout: &state::CardLayout,
    canvas: &mut Renderer,
) -> Result<(), String>
{
    use state::Selection as S;
    let side = Side::Facing;
    let (x, y) = match selection {
        S::Hand { index } => hand_card_location(*index, layout.hand.len(), side),
        S::Prize { index } => prize_card_location(*index, side),
        S::Slot { slot_index, pokemon_index } => {
            let pokemon_index = match pokemon_index {
                Some(i) => *i,
                None => 0,
            };
            slot_card_location(*slot_index, pokemon_index, side)
        },
        S::Discard { index } => deck_and_discard_card_location(*index, layout.discard.len()),
        S::LostZone { .. } => lost_zone_location(side),
        S::Stadium { .. } => stadium_location(side),
    };
    highlight_card_at(x, y, color, canvas)?;

    Ok(())
}

fn draw_damage_counters(
    card_x: i32,
    card_y: i32,
    side: Side,
    damage: u8,
    canvas: &mut Renderer,
) -> Result<(), String>
{
    if damage == 0 {
        return Ok(());
    }
    let min_pixels_die = 7;
    let dice_sidelength = min_pixels_die * DICE_SCALE;
    let num_full_dice = damage / 6;
    let last_die_value = damage % 6;
    let mut dice_to_draw = vec![6; num_full_dice as usize];
    dice_to_draw.push(last_die_value);
    for (i, v) in dice_to_draw.into_iter().enumerate() {
        let draw_x = card_x;
        let draw_y = card_y + i as i32 * (dice_sidelength + SPACE_BETWEEN_DICE) as i32;
        if v > 0 {
            draw_die(draw_x, draw_y, v, canvas)?;
        }
    }
    Ok(())
}

fn draw_die(x: i32, y: i32, value: u8, canvas: &mut Renderer) -> Result<(), String>
{
    let min_pixels_die = 7;
    let dice_sidelength = min_pixels_die * DICE_SCALE;
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.fill_rect(rect!(x, y, dice_sidelength, dice_sidelength))?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));

    let mut draw_spot = |px: i32, py: i32| {
        let px = px * DICE_SCALE as i32;
        let py = py * DICE_SCALE as i32;
        canvas.fill_rect(rect!(x + px, y + py, DICE_SCALE, DICE_SCALE))
    };

    match value {
        1 => draw_spot(3, 3)?,
        2 => {
            draw_spot(1, 1)?;
            draw_spot(5, 5)?;
        },
        3 => {
            draw_spot(1, 1)?;
            draw_spot(3, 3)?;
            draw_spot(5, 5)?;
        },
        4 => {
            draw_spot(1, 1)?;
            draw_spot(1, 5)?;
            draw_spot(5, 1)?;
            draw_spot(5, 5)?;
        },
        5 => {
            draw_spot(1, 1)?;
            draw_spot(1, 5)?;
            draw_spot(5, 1)?;
            draw_spot(5, 5)?;
            draw_spot(3, 3)?;
        },
        6 => {
            draw_spot(1, 1)?;
            draw_spot(1, 5)?;
            draw_spot(5, 1)?;
            draw_spot(5, 5)?;
            draw_spot(5, 3)?;
            draw_spot(1, 3)?;
        },
        _ => panic!("Can't draw a die with value {}", value),
    }
    Ok(())
}

fn card_list_x(index: usize, len: usize) -> i32
{
    let effective_width = WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH;
    let list_width = CARD_SMALL_DISPLAY_WIDTH * len as u32;
    let (start, card_display_width) = if list_width <= effective_width {
        ((effective_width as i32 - list_width as i32) / 2, CARD_SMALL_DISPLAY_WIDTH)
    } else {
        (0, effective_width / len as u32)
    };
    start + (index as i32 * card_display_width as i32)
}

fn hand_card_location(hand_index: usize, hand_len: usize, side: Side) -> (i32, i32)
{
    (card_list_x(hand_index, hand_len), side.y(0))
}

fn deck_and_discard_card_location(index: usize, len: usize) -> (i32, i32)
{
    let y = (WINDOW_HEIGHT - CARD_SMALL_DISPLAY_HEIGHT) / 2;
    (card_list_x(index, len), y as i32)
}

fn prize_card_location(i: usize, side: Side) -> (i32, i32)
{
    let start_y = CARD_SMALL_DISPLAY_HEIGHT as i32 + HAND_MAT_DISTANCE;
    let start_x = BOARD_SIDE_MARGIN;
    let grid_x = (i % 2) as i32;
    let grid_y = (i / 2) as i32;
    let extra_x = grid_x * (CARD_SMALL_DISPLAY_WIDTH as i32 + SPACE_BETWEEN_ADJACENT_PRIZES);
    let extra_y = grid_y * (CARD_SMALL_DISPLAY_HEIGHT as i32 + SPACE_BETWEEN_ADJACENT_PRIZES);
    (start_x + extra_x, side.y(start_y + extra_y))
}

fn slot_card_location(slot_index: usize, card_index: usize, side: Side) -> (i32, i32)
{
    let bench_x = BOARD_SIDE_MARGIN
        + CARD_SMALL_DISPLAY_WIDTH as i32 * 2
        + SPACE_BETWEEN_ADJACENT_PRIZES
        + PRIZES_BENCH_DISTANCE;
    let active_x = bench_x + ((BENCH_WIDTH - CARD_SMALL_DISPLAY_WIDTH) / 2) as i32;
    let bench_y = CARD_SMALL_DISPLAY_HEIGHT as i32 + HAND_MAT_DISTANCE;
    let active_y = BENCH_ACTIVE_DISTANCE + CARD_SMALL_DISPLAY_HEIGHT as i32 + bench_y;

    let (x, y, offset_x, offset_y) = if slot_index == 0
    /* Is active! */
    {
        (active_x, active_y, ATTACH_OFFSET_X_ACTIVE, ATTACH_OFFSET_Y_ACTIVE)
    } else {
        let bench_index = (slot_index - 1) as i32;
        let bench_slot_width = (BENCH_WIDTH / 5) as i32;
        (bench_x + bench_index * bench_slot_width, bench_y, ATTACH_OFFSET_X, ATTACH_OFFSET_Y)
    };

    let offset_x = card_index as i32 * offset_x;
    let offset_y = card_index as i32 * offset_y;
    (x + offset_x, side.y(y + offset_y))
}

fn discard_location(side: Side) -> (i32, i32)
{
    let x = BOARD_SIDE_MARGIN
        + CARD_SMALL_DISPLAY_WIDTH as i32 * 2
        + SPACE_BETWEEN_ADJACENT_PRIZES
        + PRIZES_BENCH_DISTANCE
        + BENCH_WIDTH as i32
        + BENCH_DECK_DISTANCE;
    let y = CARD_SMALL_DISPLAY_HEIGHT as i32 + HAND_MAT_DISTANCE;
    (x, side.y(y))
}

fn deck_location(side: Side) -> (i32, i32)
{
    let (x, _) = discard_location(side);
    let y = CARD_SMALL_DISPLAY_HEIGHT as i32 * 2 + HAND_MAT_DISTANCE + DECK_DISCARD_DISTANCE;
    (x, side.y(y))
}

fn lost_zone_location(side: Side) -> (i32, i32)
{
    let x = BOARD_SIDE_MARGIN
        + CARD_SMALL_DISPLAY_WIDTH as i32 * 2
        + SPACE_BETWEEN_ADJACENT_PRIZES
        + PRIZES_LOST_ZONE_DISTANCE;
    let y = CARD_SMALL_DISPLAY_HEIGHT as i32
        + HAND_MAT_DISTANCE
        + 2 * (CARD_SMALL_DISPLAY_HEIGHT as i32 + SPACE_BETWEEN_ADJACENT_PRIZES);
    (x, side.y(y))
}

fn stadium_location(side: Side) -> (i32, i32)
{
    let (stadium_x, y) = lost_zone_location(side);
    (stadium_x + LOST_ZONE_STADIUM_DISTANCE, y)
}

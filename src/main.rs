use tcgclient::load_cards;
use tcgclient::display_constants::*;
use tcgclient::state;
use tcgclient::input;
use tcgclient::draw_board;

use sdl2::{event::Event, pixels::Color};
const WINDOW_NAME: &str = "pokemon!!! :3";
const BGCOLOR: Color = Color::RGB(255, 255, 255);

fn main() -> Result<(), String>
{
    let sdl_context = sdl2::init()?;

    // Video init
    let video_subsys = sdl_context.video()?;
    let _image_context =
        sdl2::image::init(sdl2::image::InitFlag::PNG | sdl2::image::InitFlag::JPG)?;
    let window = video_subsys
        .window(WINDOW_NAME, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let tex_creator = canvas.texture_creator();

    let deck1_filename = "2008/empoleon_bronzong_long.txt";
    let deck2_filename = "2008/honchkrow_long.txt";
    let (card_loader, card_textures) =
        load_cards::CardIndexer::make(deck1_filename, deck2_filename, &tex_creator);
    let deck1 = card_loader.build_deck(deck1_filename);
    let deck2 = card_loader.build_deck(deck2_filename);

    let mut st = state::State::make(deck1, deck2);
    st.setup();

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        match event_pump.wait_event() {
            Event::Quit { .. } => {
                break 'running;
            },
            Event::KeyDown { keycode: Some(k), .. } => {
                if let Some(inp) = input::keycode_to_input(k) {
                    st.update(&inp);
                }
            },
            _ => (),
        }

        // Draw (specific)
        canvas.set_draw_color(BGCOLOR);
        canvas.clear();

        draw_board::draw(&mut canvas, &st, &card_textures)?;

        /* let flareon_index = card_loader.index_of("flareon-ex-delta-species-ds-108");
        let (flareon_w, flareon_h) = card_loader.get_dimensions(flareon_index);
        let flareon_tex = &mut card_textures[flareon_index];
        canvas.copy(
            flareon_tex,
            Rect::new(0, 0, flareon_w, flareon_h),
            Rect::new(
                (WINDOW_WIDTH - CARD_LARGE_DISPLAY_WIDTH) as i32,
                0,
                CARD_LARGE_DISPLAY_WIDTH,
                CARD_LARGE_DISPLAY_HEIGHT,
            ),
        )?;
        canvas.copy(
            flareon_tex,
            Rect::new(0, 0, 600, 850),
            Rect::new(0, 0, CARD_SMALL_DISPLAY_WIDTH, CARD_SMALL_DISPLAY_HEIGHT),
        )?;*/

        canvas.present();
    }

    Ok(())
}

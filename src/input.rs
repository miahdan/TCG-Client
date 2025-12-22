use sdl2::keyboard::Keycode;

pub enum Input
{
    Left,
    Right,

    Slot (usize),
    Hand,
    Discard,
    Stadium,
    LostZone,
    Prizes,

    Deck,

    Top,
    Bottom,

    Select,
    Cancel,

    Flip,
    Increment,
    Decrement,

    SwitchSides,

    Move,
    Swap,

    Append,
    Prepend,

    Observe,
    Shuffle,

    Roll,
}

pub fn keycode_to_input(k: Keycode) -> Option<Input>
{
    use Keycode as K;
    use Input as I;
    let i = match k {
        K::Semicolon => I::Left,
        K::Quote => I::Right,
        K::Num1 => I::Slot(1),
        K::Num2 => I::Slot(2),
        K::Num3 => I::Slot(3),
        K::Num4 => I::Slot(4),
        K::Num5 => I::Slot(5),
        K::Num6 => I::Slot(6),
        K::H => I::Hand,
        K::X => I::Discard,
        K::S => I::Stadium,
        K::L => I::LostZone,
        K::P => I::Prizes,

        K::D => I::Deck,

        K::T => I::Top,
        K::B => I::Bottom,

        K::Return => I::Select,
        K::Escape => I::Cancel,

        K::F => I::Flip,
        K::Equals => I::Increment,
        K::Minus => I::Decrement,

        K::Space => I::SwitchSides,

        K::M => I::Move,
        K::W => I::Swap,

        K::A => I::Append,
        K::E => I::Prepend,

        K::O => I::Observe,

        // TODO find a better letter than Q for shuffle??? Lol
        K::Q => I::Shuffle,

        K::R => I::Roll,

        _ => return None,
    };
    Some(i)
}

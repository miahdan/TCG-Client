use sdl2::image::LoadTexture;
use sdl2::video::WindowContext;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::path::Path;

pub struct CardIndexer
{
    name_to_index: HashMap<String, usize>,
    //textures: Vec<Texture<'a>>,
    dimensions: Vec<(u32, u32)>,
}

impl<'a> CardIndexer
{
    pub fn make(
        deck1_filename: &str, deck2_filename: &str,

        tex_creator: &'a TextureCreator<WindowContext>,
        ) -> (Self, Vec<Texture<'a>>)
    {
        let cards_path = Path::new("/Users/mia/fun/mon_tcg_sim/cards");
        let deck1_path = cards_path.join("decks").join(deck1_filename);
        let deck1_string = match read_to_string(&deck1_path) {
            Err(_) => panic!("Problem reading file {:?}", deck1_path),
            Ok(s) => s,
        };
        let deck2_path = cards_path.join("decks").join(deck2_filename);
        let deck2_string = match read_to_string(&deck2_path) {
            Err(_) => panic!("Problem reading file {:?}", deck2_path),
            Ok(s) => s,
        };

        let (sets_v, cards_v) : (Vec<(usize, &str)>, Vec<(usize, &str)>) = deck1_string
            .lines()
            .chain(deck2_string.lines())
            .filter(|s| s.len() > 2)
            .enumerate()
            .partition(|(i, _)| i % 2 == 0);
        let mut cards_set: HashSet<(String, String)> = HashSet::new();
        for i in 0..sets_v.len() {
            let set = sets_v[i].1.to_string();
            let card = cards_v[i].1.to_string();
            cards_set.insert((set, card));
        }

        let mut name_to_index = HashMap::new();
        let mut textures = Vec::new();
        let mut dimensions = Vec::new();

        for (set, card) in cards_set.into_iter() {
            let dims = dimensions_from_set(&set);
            let card_path = cards_path.join(set).join(&card).with_extension("jpg");
            let tex = tex_creator.load_texture(card_path).unwrap();
            name_to_index.insert(card, textures.len());
            textures.push(tex);
            dimensions.push(dims);
        }

        (CardIndexer { name_to_index, dimensions }, textures)
    }

    pub fn index_of(&self, card_name: &str) -> usize
    {
        self.name_to_index[card_name]
    }

    pub fn get_dimensions(&self, u: usize) -> (u32, u32) {
        self.dimensions[u]
    }

    pub fn build_deck(&self, deck_filename: &str) -> Vec<usize>
    {
        let cards_path = Path::new("/Users/mia/fun/mon_tcg_sim/cards");
        let deck_path = cards_path.join("decks").join(deck_filename);
        let deck_string = match read_to_string(&deck_path) {
            Err(_) => panic!("Problem reading file {:?}", deck_path),
            Ok(s) => s,
        };
        let (counts, cards) : (Vec<(usize, &str)>, Vec<(usize, &str)>) = deck_string
            .lines()
            .enumerate()
            .filter(|(i, s)| !s.is_empty() && i % 3 != 1)
            .partition(|(i, _)| i % 3 == 0);
        let mut deck = Vec::new();
        for i in 0..cards.len() {
            let count = counts[i].1.parse::<u32>().unwrap();
            let card = cards[i].1;
            let card_index = self.index_of(card);
            for _ in 0..count {
                deck.push(card_index);
            }
        }
        deck
    }
}

fn dimensions_from_set(set: &str) -> (u32, u32)
{
    // TODO!!!!
    (600, 835)
}

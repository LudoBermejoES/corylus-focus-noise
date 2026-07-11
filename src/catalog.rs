//! Static catalog of ambient sounds shipped in the bundle, plus the 10
//! built-in preset mixes. Generated from `VERIFIED_CATALOG.json` /
//! `PRESETS.json` in the repo root — keep in sync when the catalog changes.

/// One sound entry in the catalog.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SoundCatalogEntry {
    pub id: &'static str,
    pub display_name: &'static str,
    pub category: &'static str,
    /// freesound.org page, recorded for attribution/provenance only — never
    /// fetched at runtime (the app downloads the pre-built bundle instead).
    pub source_url: &'static str,
    pub license: &'static str,
    pub author: &'static str,
}

impl SoundCatalogEntry {
    /// Path of this sound's file within the extracted bundle / app data dir:
    /// `<category>/<id>.ogg`.
    pub fn relative_path(&self) -> String {
        format!("{}/{}.ogg", self.category, self.id)
    }
}

/// One sound's volume within a mix or preset.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MixSound {
    pub id: String,
    pub volume: f32,
}

/// A named, read-only built-in mix. Compiled-in static data, only ever
/// serialized outward to the frontend — never deserialized.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub sounds: &'static [(&'static str, f32)],
}

macro_rules! sound {
    ($id:expr, $name:expr, $cat:expr, $url:expr, $lic:expr, $author:expr) => {
        SoundCatalogEntry {
            id: $id,
            display_name: $name,
            category: $cat,
            source_url: $url,
            license: $lic,
            author: $author,
        }
    };
}

pub static CATALOG: &[SoundCatalogEntry] = &[
    // nature
    sound!("river", "River", "nature", "https://freesound.org/people/Tom_Kaszuba/sounds/660265/", "CC0", "Tom_Kaszuba"),
    sound!("campfire", "Campfire", "nature", "https://freesound.org/people/FunWithSound/sounds/588401/", "CC0", "FunWithSound"),
    sound!("wind_forest", "Wind Through Trees", "nature", "https://freesound.org/people/Yoyodaman234/sounds/667597/", "CC0", "Yoyodaman234"),
    sound!("ocean_waves", "Ocean Waves", "nature", "https://freesound.org/people/CVLTIV8R/sounds/803679/", "CC0", "CVLTIV8R"),
    sound!("waterfall", "Waterfall", "nature", "https://freesound.org/people/BonnyOrbit/sounds/442475/", "CC0", "BonnyOrbit"),
    sound!("forest_ambience", "Forest Ambience", "nature", "https://freesound.org/people/nickmaysoundmusic/sounds/523372/", "CC0", "nickmaysoundmusic"),
    sound!("thunder_distant", "Distant Thunder", "nature", "https://freesound.org/people/richwise/sounds/581232/", "CC0", "richwise"),
    // rain
    sound!("light_rain", "Light Rain", "rain", "https://freesound.org/people/_lynks/sounds/595717/", "CC0", "_lynks"),
    sound!("heavy_rain", "Heavy Rain", "rain", "https://freesound.org/people/yaanick/sounds/570307/", "CC0", "yaanick"),
    sound!("rain_on_window", "Rain on Window", "rain", "https://freesound.org/people/mffm/sounds/236292/", "CC0", "mffm"),
    sound!("rain_on_roof", "Rain on Tin Roof", "rain", "https://freesound.org/people/MrFossy/sounds/521772/", "CC0", "MrFossy"),
    sound!("rain_in_forest", "Rain in Forest", "rain", "https://freesound.org/people/Malte007/sounds/641871/", "CC0", "Malte007"),
    sound!("rain_thunder", "Rain with Thunder", "rain", "https://freesound.org/people/Vrymaa/sounds/810880/", "CC0", "Vrymaa"),
    // animals
    sound!("birds_chirping", "Birds Chirping", "animals", "https://freesound.org/people/hoshisato/sounds/852999/", "CC0", "hoshisato"),
    sound!("crickets_night", "Crickets at Night", "animals", "https://freesound.org/people/RyanKingArt/sounds/746365/", "CC0", "RyanKingArt"),
    sound!("seagulls", "Seagulls", "animals", "https://freesound.org/people/Johnsy/sounds/533042/", "CC0", "Johnsy"),
    sound!("owl_hoot", "Owl Hoot", "animals", "https://freesound.org/people/Patrick_Corra/sounds/745208/", "CC0", "Patrick_Corra"),
    sound!("cat_purring", "Cat Purring", "animals", "https://freesound.org/people/conleec/sounds/149487/", "CC0", "conleec"),
    sound!("farm_night", "Farm at Night", "animals", "https://freesound.org/people/felix.blume/sounds/355339/", "CC0", "felix.blume"),
    // urban
    sound!("city_ambience", "City Ambience", "urban", "https://freesound.org/people/anapb/sounds/561463/", "CC0", "anapb"),
    sound!("busy_street", "Busy Street", "urban", "https://freesound.org/people/mhtaylor67/sounds/238718/", "CC0", "mhtaylor67"),
    sound!("distant_siren", "Distant Siren", "urban", "https://freesound.org/people/brunoboselli/sounds/469363/", "CC0", "brunoboselli"),
    sound!("construction_site", "Construction Site", "urban", "https://freesound.org/people/craigsmith/sounds/479535/", "CC0", "craigsmith"),
    sound!("highway_traffic", "Highway Traffic", "urban", "https://freesound.org/people/kyles/sounds/453584/", "CC0", "kyles"),
    sound!("distant_fireworks", "Distant Fireworks", "urban", "https://freesound.org/people/ted/sounds/434224/", "CC0", "ted"),
    // places
    sound!("coffee_shop", "Coffee Shop", "places", "https://freesound.org/people/waweee/sounds/370973/", "CC0", "waweee"),
    sound!("library_quiet", "Quiet Library", "places", "https://freesound.org/people/kyles/sounds/635727/", "CC0", "kyles"),
    sound!("office_typing", "Office with Typing", "places", "https://freesound.org/people/Alex_hears_things/sounds/636861/", "CC0", "Alex_hears_things"),
    sound!("restaurant_diner", "Restaurant Diner", "places", "https://freesound.org/people/Laggardson/sounds/627079/", "CC0", "Laggardson"),
    sound!("subway_station", "Subway Station", "places", "https://freesound.org/people/florianreichelt/sounds/451720/", "CC0", "florianreichelt"),
    sound!("airport_terminal", "Airport Terminal", "places", "https://freesound.org/people/jrosin/sounds/210786/", "CC0", "jrosin"),
    // transport
    sound!("train_interior", "Train Interior", "transport", "https://freesound.org/people/Yoyodaman234/sounds/341208/", "CC0", "Yoyodaman234"),
    sound!("airplane_cabin", "Airplane Cabin", "transport", "https://freesound.org/people/FillSoko/sounds/456092/", "CC0", "FillSoko"),
    sound!("helicopter_flyby", "Helicopter Flyby", "transport", "https://freesound.org/people/Nox_Sound/sounds/546805/", "CC0", "Nox_Sound"),
    sound!("car_interior", "Car Interior Driving", "transport", "https://freesound.org/people/Kinoton/sounds/397115/", "CC0", "Kinoton"),
    sound!("rowing_boat", "Rowing Boat", "transport", "https://freesound.org/people/Fenodyrie/sounds/588307/", "CC0", "Fenodyrie"),
    sound!("kayaking", "Kayaking Calm Water", "transport", "https://freesound.org/people/AugustSandberg/sounds/509323/", "CC0", "AugustSandberg"),
    // things
    sound!("clock_ticking", "Clock Ticking", "things", "https://freesound.org/people/ZoeVixen/sounds/417593/", "CC0", "ZoeVixen"),
    sound!("typewriter", "Typewriter Typing", "things", "https://freesound.org/people/nvmbky/sounds/801119/", "CC0", "nvmbky"),
    sound!("keyboard_typing", "Mechanical Keyboard", "things", "https://freesound.org/people/simeonradivoev/sounds/638035/", "CC0", "simeonradivoev"),
    sound!("page_turn", "Page Turning", "things", "https://freesound.org/people/moai15/sounds/336374/", "CC0", "moai15"),
    sound!("fan_ambience", "Fan Ambience", "things", "https://freesound.org/people/IanStarGem/sounds/269594/", "CC0", "IanStarGem"),
    sound!("computer_noises", "Computer Noises", "things", "https://freesound.org/people/DFdirector/sounds/439627/", "CC0", "DFdirector"),
    // noise
    sound!("white_noise", "White Noise", "noise", "https://freesound.org/people/JarredGibb/sounds/249313/", "CC0", "JarredGibb"),
    sound!("pink_noise", "Pink Noise", "noise", "https://freesound.org/people/Zrte/sounds/470754/", "CC0", "Zrte"),
    sound!("brown_noise", "Brown Noise", "noise", "https://freesound.org/people/Hinoirocks/sounds/242513/", "CC0", "Hinoirocks"),
    sound!("soft_brown_noise", "Soft Brownian Noise", "noise", "https://freesound.org/people/Sadiquecat/sounds/853302/", "CC0", "Sadiquecat"),
    sound!("pink_noise_filtered", "Filtered Pink Noise", "noise", "https://freesound.org/people/kyles/sounds/637144/", "CC0", "kyles"),
    sound!("air_tone_noise", "Air Tone Ambience", "noise", "https://freesound.org/people/senorstudy/sounds/437281/", "CC0", "senorstudy"),
];

/// The 8 fixed categories, in display order.
pub static CATEGORIES: &[&str] = &[
    "nature", "rain", "animals", "urban", "places", "transport", "things", "noise",
];

pub static PRESETS: &[Preset] = &[
    Preset { id: "rainy_night", name: "Rainy Night", sounds: &[("heavy_rain", 0.8), ("wind_forest", 0.25)] },
    Preset { id: "ocean_breeze", name: "Ocean Breeze", sounds: &[("ocean_waves", 0.7), ("wind_forest", 0.3)] },
    Preset { id: "campfire_night", name: "Campfire Night", sounds: &[("campfire", 0.65), ("crickets_night", 0.4)] },
    Preset { id: "cafe_focus", name: "Café Focus", sounds: &[("coffee_shop", 0.65), ("light_rain", 0.2)] },
    Preset { id: "forest_morning", name: "Forest Morning", sounds: &[("birds_chirping", 0.6), ("river", 0.45), ("forest_ambience", 0.3)] },
    Preset { id: "thunderstorm", name: "Thunderstorm", sounds: &[("rain_thunder", 0.85), ("thunder_distant", 0.5)] },
    Preset { id: "deep_focus_noise", name: "Deep Focus Noise", sounds: &[("brown_noise", 0.5)] },
    Preset { id: "city_night", name: "City Night", sounds: &[("city_ambience", 0.4), ("rain_on_window", 0.35)] },
    Preset { id: "coastal_storm", name: "Coastal Storm", sounds: &[("ocean_waves", 0.75), ("rain_thunder", 0.55), ("wind_forest", 0.4)] },
    Preset { id: "quiet_library", name: "Quiet Library", sounds: &[("library_quiet", 0.5), ("clock_ticking", 0.2), ("page_turn", 0.15)] },
];

/// Look up a catalog entry by id.
pub fn find(id: &str) -> Option<&'static SoundCatalogEntry> {
    CATALOG.iter().find(|s| s.id == id)
}

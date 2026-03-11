use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub anime: AnimeConfig,
    #[serde(default)]
    pub manga: MangaConfig,
    #[serde(default)]
    pub novel: NovelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub show_adult_content: bool,
    pub blur_adult_content: bool,
    pub theme: Theme,
    pub accent_color: String,
    pub sidebar_collapsed: bool,
    pub disable_card_trailers: bool,
    pub auto_update_progress: bool,
    pub notifications_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
    Sepia,
    Oled,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            show_adult_content: false,
            blur_adult_content: true,
            theme: Theme::default(),
            accent_color: "#6366f1".into(),
            sidebar_collapsed: false,
            disable_card_trailers: false,
            auto_update_progress: true,
            notifications_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeConfig {
    pub autoplay_next_episode: bool,
    pub preferred_metadata_provider: String,
    pub preferred_sub_lang: String,
    pub preferred_dub_lang: String,
    pub auto_skip_intro: bool,
    pub auto_skip_outro: bool,
    pub seek_step: u8,
    pub resume_from_last_pos: bool,
    pub extension_repo_url: String,
    pub default_episode_layout: EpisodeLayout,
    pub notify_new_episodes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum EpisodeLayout {
    #[default]
    Grid,
    List,
}

impl Default for AnimeConfig {
    fn default() -> Self {
        Self {
            autoplay_next_episode: true,
            preferred_metadata_provider: "anilist".into(),
            preferred_sub_lang: "en".into(),
            preferred_dub_lang: "en".into(),
            auto_skip_intro: false,
            auto_skip_outro: false,
            seek_step: 10,
            resume_from_last_pos: true,
            extension_repo_url: String::new(),
            default_episode_layout: EpisodeLayout::default(),
            notify_new_episodes: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaConfig {
    pub layout: MangaLayout,
    pub direction: ReadingDirection,
    pub pages_per_view: u8,
    pub fit_mode: FitMode,
    pub gap_x: u8,
    pub gap_y: u8,
    pub preload_pages: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum MangaLayout {
    #[default]
    Scroll,
    Paged,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ReadingDirection {
    Ltr,
    #[default]
    Rtl,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FitMode {
    #[default]
    Width,
    Height,
}


impl Default for MangaConfig {
    fn default() -> Self {
        Self {
            layout: MangaLayout::default(),
            direction: ReadingDirection::default(),
            pages_per_view: 1,
            fit_mode: FitMode::default(),
            gap_x: 0,
            gap_y: 8,
            preload_pages: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelConfig {
    pub theme: NovelTheme,
    pub font_family: FontFamily,
    pub font_size: u8,
    pub line_height: f32,
    pub max_width: u16,
    pub text_align: TextAlign,
    pub paragraph_spacing: f32
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum NovelTheme {
    #[default]
    Light,
    Dark,
    Sepia,
    Oled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FontFamily {
    #[default]
    Sans,
    Serif,
    Mono,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    #[default]
    Left,
    Justify,
}

impl Default for NovelConfig {
    fn default() -> Self {
        Self {
            theme: NovelTheme::default(),
            font_family: FontFamily::default(),
            font_size: 16,
            line_height: 1.6,
            max_width: 700,
            paragraph_spacing: 2.0,
            text_align: TextAlign::default(),
        }
    }
}
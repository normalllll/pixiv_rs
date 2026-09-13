use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Restrict {
    Public,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllustRankingMode {
    Day,
    DayR18,
    DayMale,
    DayMaleR18,
    DayAi,
    DayR18Ai,
    DayFemale,
    DayFemaleR18,
    Week,
    WeekR18,
    WeekOriginal,
    WeekRookie,
    Month,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MangaRankingMode {
    Day,
    Week,
    Month,
    DayR18,
    WeekR18,
    WeekR18G,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NovelRankingMode {
    Day,
    DayR18,
    DayMale,
    DayFemale,
    Week,
    WeekR18,
    WeekRookie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IllustType {
    Illust,
    Manga,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkType {
    Illust,
    Manga,
    Novel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchSort {
    DateDesc,
    DateAsc,
    PopularDesc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchTarget {
    PartialMatchForTags,
    ExactMatchForTags,
    TitleAndCaption,
}

pub trait PixivEnumParam {
    fn as_pixiv_param(&self) -> &'static str;
}

impl PixivEnumParam for Restrict {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
}

impl PixivEnumParam for IllustRankingMode {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::DayR18 => "day_r18",
            Self::DayMale => "day_male",
            Self::DayMaleR18 => "day_male_r18",
            Self::DayAi => "day_ai",
            Self::DayR18Ai => "day_r18_ai",
            Self::DayFemale => "day_female",
            Self::DayFemaleR18 => "day_female_r18",
            Self::Week => "week",
            Self::WeekR18 => "week_r18",
            Self::WeekOriginal => "week_original",
            Self::WeekRookie => "week_rookie",
            Self::Month => "month",
        }
    }
}

impl PixivEnumParam for MangaRankingMode {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::Day => "day_manga",
            Self::Week => "week_manga",
            Self::Month => "month_manga",
            Self::DayR18 => "day_r18_manga",
            Self::WeekR18 => "week_r18_manga",
            Self::WeekR18G => "week_r18g_manga",
        }
    }
}

impl PixivEnumParam for NovelRankingMode {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::DayR18 => "day_r18",
            Self::DayMale => "day_male",
            Self::DayFemale => "day_female",
            Self::Week => "week",
            Self::WeekR18 => "week_r18",
            Self::WeekRookie => "week_rookie",
        }
    }
}

impl PixivEnumParam for IllustType {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::Illust => "illust",
            Self::Manga => "manga",
        }
    }
}

impl PixivEnumParam for WorkType {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::Illust => "illust",
            Self::Manga => "manga",
            Self::Novel => "novel",
        }
    }
}

impl PixivEnumParam for SearchSort {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::DateDesc => "date_desc",
            Self::DateAsc => "date_asc",
            Self::PopularDesc => "popular_desc",
        }
    }
}

impl PixivEnumParam for SearchTarget {
    fn as_pixiv_param(&self) -> &'static str {
        match self {
            Self::PartialMatchForTags => "partial_match_for_tags",
            Self::ExactMatchForTags => "exact_match_for_tags",
            Self::TitleAndCaption => "title_and_caption",
        }
    }
}

macro_rules! impl_display_as_pixiv_param {
    ($($ty:ty),* $(,)?) => {
        $(
            impl fmt::Display for $ty {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(self.as_pixiv_param())
                }
            }
        )*
    };
}

impl_display_as_pixiv_param!(
    Restrict,
    IllustRankingMode,
    MangaRankingMode,
    NovelRankingMode,
    IllustType,
    WorkType,
    SearchSort,
    SearchTarget
);

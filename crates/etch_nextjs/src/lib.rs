use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::{
    path::Path,
    str::FromStr,
};
use strum::{AsRefStr, EnumString};
use ts_rs::TS;
use walkdir::WalkDir;
pub struct Cli;

#[derive(Debug, thiserror::Error)]
pub enum AppRouterError {
    #[error("Unknown asset type")]
    UnknownAssetType,
    #[error("Unknown directory type")]
    UnknownDirectoryType,
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("Failed to convert path to string")]
    PathConversionError,
    #[error(transparent)]
    AssetFolderKindConversionError(#[from] strum::ParseError),
}

#[derive(Debug, Serialize, Deserialize, EnumString, AsRefStr, TS)]
#[ts(export)]
pub enum AssetFolderKind {
    Font,
    Image,
    Video,
    Audio,
    Other,
}

impl AssetFolderKind {
    pub fn is_asset_folder_str(folder: &str) -> bool {
        AssetFolderKind::try_from(folder).is_ok()
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "directory_kind")]
pub enum AppRouterDirectoryKind {
    AssetFolder { asset_kind: AssetFolderKind },
    StaticRoute,
    DynamicRoute,
    CatchAllRoute,
    PrivateRoute,
    RouteGroup,
    DynamicRouteWithParams,
    DynamicRouteWithOptionalParams,
    DynamicRouteWithOptionalParamsAndCatchAll,
    NamedSlot,
    InterceptSameLevel,
    InterceptOneLevelAbove,
    InterceptTwoLevelsAbove,
    InterceptFromRoot,
}

impl TryFrom<&str> for AppRouterDirectoryKind {
    type Error = AppRouterError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split('/').collect();
        match parts.as_slice() {
            [folder] if AssetFolderKind::is_asset_folder_str(folder) => {
                let asset_kind = AssetFolderKind::from_str(folder)?;
                Ok(AppRouterDirectoryKind::AssetFolder { asset_kind })
            }
            [folder] if folder.starts_with('[') && folder.ends_with(']') => {
                Ok(AppRouterDirectoryKind::DynamicRouteWithParams)
            }
            [folder] if folder.starts_with("[...") && folder.ends_with(']') => {
                Ok(AppRouterDirectoryKind::CatchAllRoute)
            }
            [folder] if folder.starts_with("[[...") && folder.ends_with("]]") => {
                Ok(AppRouterDirectoryKind::DynamicRouteWithOptionalParamsAndCatchAll)
            }
            [folder] if folder.starts_with('(') && folder.ends_with(')') => {
                Ok(AppRouterDirectoryKind::RouteGroup)
            }
            [folder] if folder.starts_with('_') => Ok(AppRouterDirectoryKind::PrivateRoute),
            [folder] if folder.starts_with('@') => Ok(AppRouterDirectoryKind::NamedSlot),
            [folder] if folder.starts_with("(.)") => Ok(AppRouterDirectoryKind::InterceptSameLevel),
            [folder] if folder.starts_with("(..)") => {
                Ok(AppRouterDirectoryKind::InterceptOneLevelAbove)
            }
            [folder] if folder.starts_with("(..)(..)") => {
                Ok(AppRouterDirectoryKind::InterceptTwoLevelsAbove)
            }
            [folder] if folder.starts_with("(...)") => {
                Ok(AppRouterDirectoryKind::InterceptFromRoot)
            }
            [folder] => Ok(AppRouterDirectoryKind::StaticRoute),
            _ => Err(AppRouterError::UnknownDirectoryType),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "file_kind")]
pub enum AppRouterFileKind {
    Layout,
    Page,
    Loading,
    NotFound,
    Error,
    GlobalError,
    Route,
    Template,
    Default,
    Other,
}

impl TryFrom<&str> for AppRouterFileKind {
    type Error = AppRouterError;

    fn try_from(value: &str) -> Result<Self, <AppRouterFileKind as TryFrom<&str>>::Error> {
        match value {
            "layout" => Ok(AppRouterFileKind::Layout),
            "page" => Ok(AppRouterFileKind::Page),
            "loading" => Ok(AppRouterFileKind::Loading),
            "not-found" => Ok(AppRouterFileKind::NotFound),
            "error" => Ok(AppRouterFileKind::Error),
            "global-error" => Ok(AppRouterFileKind::GlobalError),
            "route" => Ok(AppRouterFileKind::Route),
            "template" => Ok(AppRouterFileKind::Template),
            _ => Ok(AppRouterFileKind::Other),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Directory<T> {
    #[serde(flatten)]
    pub directory_kind: AppRouterDirectoryKind,
    pub path_segment: String,
    // pub relative_path: String,
    pub children: Vec<AppRouterEntry<T>>,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct File<T> {
    #[serde(flatten)]
    pub file_kind: AppRouterFileKind,
    pub path_segment: String,
    // pub relative_path: String,
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export)]
pub enum AppRouterEntry<T> {
    Directory(Directory<T>),
    File(File<T>),
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Self {
        Self
    }

    pub fn get_directory_structure<T:Default>(
        dir: &Path,
        base_dir: &Path,
    ) -> Result<Vec<AppRouterEntry<T>>, AppRouterError> {
        let mut app_router_entries = Vec::new();

        for entry in WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path() != dir)
        {
            let app_router_entry = build_tree::<T>(entry.path(), base_dir)?;
            app_router_entries.push(app_router_entry);
        }

        Ok(app_router_entries)
    }
}

fn build_tree<T: Default>(path: &Path, base_dir: &Path) -> Result<AppRouterEntry<T>, AppRouterError> {
    let relative_path = path.strip_prefix(base_dir).unwrap_or(path);
    let path_segment = path.file_name().unwrap().to_string_lossy().into_owned();

    if path.is_dir() {
        let directory_kind = AppRouterDirectoryKind::try_from(
            path.file_stem().unwrap().to_string_lossy().as_ref(),
        )?;
        let mut children = Vec::new();

        for entry in WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path() != path)
        // Skip the current directory itself
        {
            let child_entry = build_tree(entry.path(), base_dir)?;
            children.push(child_entry);
        }

        Ok(AppRouterEntry::Directory(Directory {
            directory_kind,
            path_segment,
            // relative_path: relative_path.to_string_lossy().into_owned(),
            children,
        }))
    } else {
        let file_kind = AppRouterFileKind::try_from(
            path.file_stem().unwrap().to_string_lossy().as_ref(),
        )?;
        Ok(AppRouterEntry::File(File {
            file_kind,
            path_segment,
            // relative_path: relative_path.to_string_lossy().into_owned(),
            data: T::default(),
        }))
    }
}

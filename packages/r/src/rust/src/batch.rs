//! Batch extraction functions

use crate::config::parse_config;
use crate::error::{kreuzberg_error, to_r_error};
use crate::result::extraction_result_to_list;
use extendr_api::prelude::*;

pub fn batch_extract_files_sync_impl(paths: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let config = parse_config(config_json)?;
        let path_vec: Vec<String> = paths.iter().map(|s| s.to_string()).collect();
        let results = kreuzberg::batch_extract_file_sync(path_vec, &config).map_err(kreuzberg_error)?;
        let r_results: Vec<Robj> = results.into_iter()
            .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
            .collect::<extendr_api::Result<Vec<_>>>()?;
        Ok(List::from_values(r_results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (paths, config_json);
        Err("Batch file extraction is not supported on WebAssembly".into())
    }
}

pub fn batch_extract_files_impl(paths: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let config = parse_config(config_json)?;
        let path_vec: Vec<String> = paths.iter().map(|s| s.to_string()).collect();
        let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
        let results = runtime
            .block_on(async { kreuzberg::batch_extract_file(path_vec, &config).await })
            .map_err(kreuzberg_error)?;
        let r_results: Vec<Robj> = results.into_iter()
            .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
            .collect::<extendr_api::Result<Vec<_>>>()?;
        Ok(List::from_values(r_results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (paths, config_json);
        Err("Async batch file extraction is not supported on WebAssembly".into())
    }
}

pub fn batch_extract_bytes_sync_impl(data_list: List, mime_types: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let mime_vec: Vec<String> = mime_types.iter().map(|s| s.to_string()).collect();
    let contents: Vec<(Vec<u8>, String)> = data_list.values()
        .zip(mime_vec.into_iter())
        .map(|(v, mime)| {
            let raw = Raw::try_from(v).map_err(to_r_error)?;
            Ok((raw.as_slice().to_vec(), mime))
        })
        .collect::<extendr_api::Result<Vec<_>>>()?;
    let results = kreuzberg::batch_extract_bytes_sync(contents, &config).map_err(kreuzberg_error)?;
    let r_results: Vec<Robj> = results.into_iter()
        .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
        .collect::<extendr_api::Result<Vec<_>>>()?;
    Ok(List::from_values(r_results))
}

pub fn batch_extract_bytes_impl(data_list: List, mime_types: Strings, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let config = parse_config(config_json)?;
        let mime_vec: Vec<String> = mime_types.iter().map(|s| s.to_string()).collect();
        let contents: Vec<(Vec<u8>, String)> = data_list.values()
            .zip(mime_vec.into_iter())
            .map(|(v, mime)| {
                let raw = Raw::try_from(v).map_err(to_r_error)?;
                Ok((raw.as_slice().to_vec(), mime))
            })
            .collect::<extendr_api::Result<Vec<_>>>()?;
        let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
        let results = runtime
            .block_on(async { kreuzberg::batch_extract_bytes(contents, &config).await })
            .map_err(kreuzberg_error)?;
        let r_results: Vec<Robj> = results.into_iter()
            .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
            .collect::<extendr_api::Result<Vec<_>>>()?;
        Ok(List::from_values(r_results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        batch_extract_bytes_sync_impl(data_list, mime_types, config_json)
    }
}

pub fn batch_extract_files_with_configs_sync_impl(paths: Strings, file_configs: List, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let config = parse_config(config_json)?;
        let path_vec: Vec<String> = paths.iter().map(|s| s.to_string()).collect();

        if path_vec.len() != file_configs.len() {
            return Err(format!(
                "paths length ({}) must match file_configs length ({})",
                path_vec.len(),
                file_configs.len()
            ).into());
        }

        let items: Vec<(std::path::PathBuf, Option<kreuzberg::FileExtractionConfig>)> = path_vec
            .into_iter()
            .zip(file_configs.values())
            .map(|(path, fc_val)| {
                let fc = if fc_val.is_null() {
                    None
                } else {
                    let fc_str: String = fc_val.try_into().map_err(to_r_error)?;
                    let fc: kreuzberg::FileExtractionConfig =
                        serde_json::from_str(&fc_str).map_err(to_r_error)?;
                    Some(fc)
                };
                Ok((std::path::PathBuf::from(path), fc))
            })
            .collect::<extendr_api::Result<Vec<_>>>()?;

        let results = kreuzberg::batch_extract_file_with_configs_sync(items, &config).map_err(kreuzberg_error)?;
        let r_results: Vec<Robj> = results.into_iter()
            .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
            .collect::<extendr_api::Result<Vec<_>>>()?;
        Ok(List::from_values(r_results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (paths, file_configs, config_json);
        Err("Batch file extraction with configs is not supported on WebAssembly".into())
    }
}

pub fn batch_extract_files_with_configs_impl(paths: Strings, file_configs: List, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let config = parse_config(config_json)?;
        let path_vec: Vec<String> = paths.iter().map(|s| s.to_string()).collect();

        if path_vec.len() != file_configs.len() {
            return Err(format!(
                "paths length ({}) must match file_configs length ({})",
                path_vec.len(),
                file_configs.len()
            ).into());
        }

        let items: Vec<(std::path::PathBuf, Option<kreuzberg::FileExtractionConfig>)> = path_vec
            .into_iter()
            .zip(file_configs.values())
            .map(|(path, fc_val)| {
                let fc = if fc_val.is_null() {
                    None
                } else {
                    let fc_str: String = fc_val.try_into().map_err(to_r_error)?;
                    let fc: kreuzberg::FileExtractionConfig =
                        serde_json::from_str(&fc_str).map_err(to_r_error)?;
                    Some(fc)
                };
                Ok((std::path::PathBuf::from(path), fc))
            })
            .collect::<extendr_api::Result<Vec<_>>>()?;

        let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
        let results = runtime
            .block_on(async { kreuzberg::batch_extract_file_with_configs(items, &config).await })
            .map_err(kreuzberg_error)?;
        let r_results: Vec<Robj> = results.into_iter()
            .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
            .collect::<extendr_api::Result<Vec<_>>>()?;
        Ok(List::from_values(r_results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (paths, file_configs, config_json);
        Err("Async batch file extraction with configs is not supported on WebAssembly".into())
    }
}

pub fn batch_extract_bytes_with_configs_sync_impl(data_list: List, mime_types: Strings, file_configs: List, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    let config = parse_config(config_json)?;
    let mime_vec: Vec<String> = mime_types.iter().map(|s| s.to_string()).collect();

    if data_list.len() != mime_vec.len() || data_list.len() != file_configs.len() {
        return Err(format!(
            "data_list length ({}), mime_types length ({}), and file_configs length ({}) must all match",
            data_list.len(),
            mime_vec.len(),
            file_configs.len()
        ).into());
    }

    let items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> = data_list.values()
        .zip(mime_vec.into_iter())
        .zip(file_configs.values())
        .map(|((v, mime), fc_val)| {
            let raw = Raw::try_from(v).map_err(to_r_error)?;
            let fc = if fc_val.is_null() {
                None
            } else {
                let fc_str: String = fc_val.try_into().map_err(to_r_error)?;
                let fc: kreuzberg::FileExtractionConfig =
                    serde_json::from_str(&fc_str).map_err(to_r_error)?;
                Some(fc)
            };
            Ok((raw.as_slice().to_vec(), mime, fc))
        })
        .collect::<extendr_api::Result<Vec<_>>>()?;

    let results = kreuzberg::batch_extract_bytes_with_configs_sync(items, &config).map_err(kreuzberg_error)?;
    let r_results: Vec<Robj> = results.into_iter()
        .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
        .collect::<extendr_api::Result<Vec<_>>>()?;
    Ok(List::from_values(r_results))
}

pub fn batch_extract_bytes_with_configs_impl(data_list: List, mime_types: Strings, file_configs: List, config_json: Nullable<&str>) -> extendr_api::Result<List> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let config = parse_config(config_json)?;
        let mime_vec: Vec<String> = mime_types.iter().map(|s| s.to_string()).collect();

        if data_list.len() != mime_vec.len() || data_list.len() != file_configs.len() {
            return Err(format!(
                "data_list length ({}), mime_types length ({}), and file_configs length ({}) must all match",
                data_list.len(),
                mime_vec.len(),
                file_configs.len()
            ).into());
        }

        let items: Vec<(Vec<u8>, String, Option<kreuzberg::FileExtractionConfig>)> = data_list.values()
            .zip(mime_vec.into_iter())
            .zip(file_configs.values())
            .map(|((v, mime), fc_val)| {
                let raw = Raw::try_from(v).map_err(to_r_error)?;
                let fc = if fc_val.is_null() {
                    None
                } else {
                    let fc_str: String = fc_val.try_into().map_err(to_r_error)?;
                    let fc: kreuzberg::FileExtractionConfig =
                        serde_json::from_str(&fc_str).map_err(to_r_error)?;
                    Some(fc)
                };
                Ok((raw.as_slice().to_vec(), mime, fc))
            })
            .collect::<extendr_api::Result<Vec<_>>>()?;

        let runtime = tokio::runtime::Runtime::new().map_err(to_r_error)?;
        let results = runtime
            .block_on(async { kreuzberg::batch_extract_bytes_with_configs(items, &config).await })
            .map_err(kreuzberg_error)?;
        let r_results: Vec<Robj> = results.into_iter()
            .map(|r| extraction_result_to_list(r).map(|l| l.into_robj()))
            .collect::<extendr_api::Result<Vec<_>>>()?;
        Ok(List::from_values(r_results))
    }
    #[cfg(target_arch = "wasm32")]
    {
        batch_extract_bytes_with_configs_sync_impl(data_list, mime_types, file_configs, config_json)
    }
}

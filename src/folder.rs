use std::{fs, path::Path};

use crate::{
    chunks::ChunksDiskFormat,
    errors::RustyChunkEncError,
    index::{read_index_disk_format, IndexDiskFormat},
};

/// A Prometheus data folder, containing an index file and a chunks folder.
#[derive(Debug)]
pub struct Folder {
    #[allow(dead_code)]
    path: String,
    #[allow(dead_code)]
    chunks_files: Vec<ChunksDiskFormat>,
    #[allow(dead_code)]
    index: IndexDiskFormat,
}

impl Folder {
    /// Parse a Prometheus data folder.
    pub fn parse_folder(folder_path: &str) -> Result<Self, RustyChunkEncError> {
        // First we try to read the index file
        let index_file_path = Path::new(folder_path).join("index");
        let index_data = fs::read(index_file_path)?;

        let (_, _index) = read_index_disk_format(&index_data)?;

        // Check if the chunk folders exist and lists it in one go
        let chunks_folder_path = Path::new(folder_path).join("chunks");
        let mut chunk_files = fs::read_dir(chunks_folder_path)?
            .map(|entry| {
                let entry = entry?;
                let file_name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| RustyChunkEncError::InvalidFileName())?;
                let file_type = entry.file_type()?;
                if file_type.is_file() && file_name.chars().all(|c| c.is_ascii_digit()) {
                    Ok(Some(file_name))
                } else {
                    Ok(None)
                }
            })
            .filter_map(|result| match result {
                Ok(None) => None,
                Ok(Some(path)) => Some(Ok(path)),
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<String>, RustyChunkEncError>>()?;

        // Sort by file name
        chunk_files.sort();

        println!("chunk_files: {:?}", chunk_files);

        Ok(Folder {
            path: folder_path.to_string(),
            chunks_files: Vec::new(),
            index: IndexDiskFormat::new(Vec::new()),
        })
    }
}

use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::disk_utils::{FileInfo, FileType};

pub fn analyze_file_categories(files: &Vec<FileInfo>) -> HashMap<FileType, u64> {
    files
        .par_iter()
        .fold(HashMap::new, |mut acc, info| {
            *acc.entry(info.file_type).or_insert(0) += info.size;
            acc
        })
        .reduce(HashMap::new, |mut acc, partial| {
            for (key, value) in partial {
                *acc.entry(key).or_insert(0) += value;
            }
            acc
        })
}

pub fn analyze_largest_files(files: &Vec<FileInfo>, n: usize) -> Vec<FileInfo> {
    files
        .par_iter()
        .fold(
            BTreeMap::new, // Thread-local map for top `n`
            |mut tree, file| {
                tree.insert(Reverse(file.size), file); // Sort by file.size in descending order
                if tree.len() > n {
                    tree.pop_last(); // Keep only the largest `n` elements
                }
                tree
            },
        )
        .reduce(
            BTreeMap::new, // Combine maps from all threads
            |mut acc, mut tree| {
                acc.append(&mut tree);
                while acc.len() > n {
                    acc.pop_last();
                }
                acc
            },
        )
        // Convert the map into a sorted Vec
        .iter()
        .map(|(_, file)| (**file).clone())
        .collect()
}

pub fn analyze_largest_dirs(dirs: &Vec<FileInfo>, n: usize) -> Vec<(u64, String)> {
    dirs.par_iter()
        .fold(
            HashMap::new, // Thread-local map for directory
            |mut acc, file| {
                *acc.entry(file.directory.clone()).or_insert(0) += file.size;
                acc
            },
        )
        .reduce(
            HashMap::new, // Combine maps from all threads
            |mut acc, partial| {
                for (dir, size) in partial {
                    *acc.entry(dir).or_insert(0) += size;
                }
                acc
            },
        )
        .into_iter()
        .fold(BTreeMap::new(), |mut tree, (dir, size)| {
            tree.insert(Reverse(size), dir); // Sort by file.size in descending order
            if tree.len() > n {
                tree.pop_last(); // Keep only the largest `n` elements
            }
            tree
        })
        // Convert the map into a sorted Vec
        .into_iter()
        .map(|(size, dir)| (size.0, dir))
        .collect()
}

use needletail::parse_fastx_file;

use std::collections::HashSet;
use std::error::Error;
use std::time::Instant;

use log::debug;
use log::info;

use crate::encode_single;
use crate::get_distances;
use crate::get_hit_sequence;
use crate::WindowSet;

pub fn cluster(
    input_fasta: &str,
    max_divergence: u32,
    print_stream: &mut dyn std::io::Write,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let max_divergence_usize = max_divergence as usize;

    // Create vec of centroids
    let mut centroids = WindowSet {
        version: 0, // not actually used
        windows: Vec::new(),
    };

    let mut seen_sequences = HashSet::<Vec<bool>>::new();

    // Iterate input sequences
    // Open the query file as a fasta file.
    let mut query_reader = parse_fastx_file(input_fasta).expect("valid path/file of input fasta");

    // Pre-initialise the distances vector so don't have to continually reallocate.
    let mut distances = vec![];

    info!("Clustering ..");
    let mut query_number: u32 = 0;
    while let Some(record) = query_reader.next() {
        query_number += 1;

        // Encode as vec of bools
        let record_unwrapped = record.expect("Failed to parse input sequence");
        let seq = record_unwrapped.seq();
        let query_vec = seq
            .iter()
            .flat_map(|c| encode_single(*c))
            .collect::<Vec<_>>();

        // Skip if sequence has already been seen
        if seen_sequences.contains(&query_vec) {
            continue;
        } else {
            seen_sequences.insert(query_vec.clone());
        }

        // Get distances
        get_distances(&centroids, &query_vec, &mut distances);

        // Find min distance and index of it
        let min_distance = if distances.is_empty() {
            max_divergence_usize * 2 + 2
        } else {
            *(distances.iter().min().unwrap())
        };

        // If distance < max_divergence then add to centroid
        let mut assigned_centroid = 0;
        if min_distance / 2 <= max_divergence_usize {
            for (i, distance) in distances.iter().enumerate() {
                if *distance == min_distance {
                    assigned_centroid = i;
                    break;
                }
            }
        } else {
            // If distance >= max_divergence then add to new centroid
            assigned_centroid = centroids.windows.len();
            centroids.windows.push(query_vec.to_vec());
            distances.push(0); // Adding another entry so that distances.len() == centroids.windows.len()
        }
        debug!("Assigned centroid: {}", assigned_centroid);
        debug!("windows len: {}", centroids.windows.len());

        // Print the sequence and the centroid it belongs to
        writeln!(
            print_stream,
            "{}\t{}",
            std::str::from_utf8(&seq).unwrap(),
            get_hit_sequence(&centroids.windows[assigned_centroid])
        )?;
    }

    info!(
        "Clustering complete, took {} seconds. Clustered {} sequences into {} clusters.",
        start.elapsed().as_secs(),
        query_number,
        centroids.windows.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_simple() {
        let mut stream = Cursor::new(Vec::new());
        cluster("tests/data/cluster_dummy1.fna", 1, &mut stream).unwrap();
        assert_eq!(
            "ATGC\tATGC
ATGG\tATGC
AAAA\tAAAA
",
            std::str::from_utf8(stream.get_ref()).unwrap()
        )
    }

    #[test]
    fn test_bug1() {
        let mut stream = Cursor::new(Vec::new());
        cluster("tests/data/cluster_bug1.fna", 2, &mut stream).unwrap();
        assert_eq!(
            "ATGCAAAAA\tATGCAAAAA\n\
             ATAAAAAAA\tATGCAAAAA\n\
             TTAAAAAAA\tTTAAAAAAA\n",
            std::str::from_utf8(stream.get_ref()).unwrap()
        )
    }

    #[test]
    fn test_best_hit_changes_bug() {
        // seq4 in the file shouldn't be reported otherwise there are two
        // sequences that are the same but are given different centroids.
        let mut stream = Cursor::new(Vec::new());
        cluster("tests/data/cluster_best_hit_changes.fna", 2, &mut stream).unwrap();
        assert_eq!(
            "ATGCAAAAA\tATGCAAAAA\n\
             ATAAAAAAA\tATGCAAAAA\n\
             TTAAAAAAA\tTTAAAAAAA\n",
            std::str::from_utf8(stream.get_ref()).unwrap()
        )
    }
}

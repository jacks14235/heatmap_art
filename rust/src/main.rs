use std::env;
use std::fs;

use fitcoords::parse_fit; // use the library's module

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());

    let mut total_points: usize = 0;

    for entry in fs::read_dir(&dir_path)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() { continue; }

        let is_fit = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("fit"))
            .unwrap_or(false);
        if !is_fit { continue; }

        match parse_fit::parse_fit_coords_from_path(&path) {
            Ok(coords) => {
                total_points += coords.len();
            }
            Err(_) => {
                // Skip unreadable/corrupt files silently
            }
        }
    }

    println!("{}", total_points);
    Ok(())
}



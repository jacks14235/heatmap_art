pub mod parse_fit;

use ndarray::Array2;
use numpy::{IntoPyArray, PyArray2};
use pyo3::prelude::*;
use std::fs;
use std::path::Path;

#[pyfunction]
fn parse_fit_to_numpy<'py>(py: Python<'py>, path: &str) -> PyResult<&'py PyArray2<f32>> {
    let coords = parse_fit::parse_fit_coords_from_path(path)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

    let n = coords.len();
    let mut flat: Vec<f32> = Vec::with_capacity(n * 2);
    for [lat, lon] in coords {
        flat.push(lat as f32);
        flat.push(lon as f32);
    }

    // Safe because we control the flat vector length (n*2)
    let arr: Array2<f32> = Array2::from_shape_vec((n, 2), flat)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

#[pyfunction]
fn parse_fit_dir_to_numpy<'py>(py: Python<'py>, dir_path: &str) -> PyResult<&'py PyArray2<f32>> {
    let p = Path::new(dir_path);
    let entries = fs::read_dir(p)
        .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;

    let mut flat: Vec<f32> = Vec::with_capacity(1 << 20);
    let mut n: usize = 0;

    for entry in entries {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        let path = entry.path();
        if !path.is_file() { continue; }
        let is_fit = path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("fit"))
            .unwrap_or(false);
        if !is_fit { continue; }

        if let Ok(coords) = parse_fit::parse_fit_coords_from_path(&path) {
            n += coords.len();
            for [lat, lon] in coords {
                flat.push(lat as f32);
                flat.push(lon as f32);
            }
        }
    }

    let arr: Array2<f32> = Array2::from_shape_vec((n, 2), flat)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    Ok(arr.into_pyarray(py))
}

#[pymodule]
fn fitcoords(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(pyo3::wrap_pyfunction!(parse_fit_to_numpy, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(parse_fit_dir_to_numpy, m)?)?;
    Ok(())
}



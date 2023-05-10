use core_affinity::CoreId;
use errors::{Error, Result};

/// Retrieves a list of `CoreId`s based on the number of requested CPUs.
///
/// # Arguments
///
/// * `requested_cpus` - The number of requested CPUs.
///
/// # Returns
///
/// A vector of `CoreId`s if the request is successful.
///
/// # Errors
///
/// * `Error::CoreIdsUnavailable` if the core IDs cannot be retrieved.
/// * `Error` with a custom message if there are not enough cores available.
///
/// # Examples
///
/// ```no_run
/// use utils::get_core_affinity;
/// use errors::{Error, Result};
///
/// let requested_cpus = 4;
/// match get_core_affinity(requested_cpus) {
///     Ok(core_ids) => println!("Core IDs: {:?}", core_ids),
///     Err(Error::CoreIdsUnavailable) => eprintln!("Core IDs are unavailable."),
///     Err(err) => eprintln!("Error: {}", err),
/// }
/// ```
pub fn get_core_affinity(requested_cpus: usize) -> Result<Vec<CoreId>> {
    let Some(cpus) = core_affinity::get_core_ids() else { return Err(Error::CoreIdsUnavailable) };
    if cpus.len() < requested_cpus {
        return Err(Error::new(&format!(
            "Not enough cores available. Requested: {}, available: {}",
            requested_cpus,
            cpus.len()
        )));
    }
    Ok(cpus
        .iter()
        .copied()
        .take(requested_cpus)
        .collect::<Vec<_>>())
}

/// Sets the core affinity for the current thread.
///
/// # Arguments
///
/// * `core_id` - A reference to the `CoreId` to which the current thread should be affinitized.
///
/// # Returns
///
/// An empty `Result` if the operation is successful.
///
/// # Errors
///
/// This function does not produce any errors.
///
/// # Examples
///
/// ```no_run
/// use utils::{get_core_affinity, set_core_affinity};
/// use errors::{Error, Result};
///
/// let requested_cpus = 1;
/// if let Ok(core_ids) = get_core_affinity(requested_cpus) {
///     if let Err(err) = set_core_affinity(&core_ids[0]) {
///         eprintln!("Error setting core affinity: {}", err);
///     }
/// }
/// ```
pub fn set_core_affinity(core_id: &CoreId) -> Result<()> {
    core_affinity::set_for_current(*core_id);
    Ok(())
}

/// Converts a hexadecimal string to an integer.
/// Allows for the string to be prefixed with `0x`.
///
/// # Arguments
///
/// * `inp` - The input hexadecimal string.
///
/// # Returns
///
/// An `Option` containing the integer value if the conversion is successful, or `None` if the conversion fails.
///
/// # Errors
///
/// This function does not produce any errors.
///
/// # Examples
///
/// ```
/// use utils::hstr_to_int;
///
/// let hex_str = "0x1A";
/// let int_val = hstr_to_int(hex_str);
/// assert_eq!(int_val, Some(26));
///
/// let hex_str = "1A";
/// let int_val = hstr_to_int(hex_str);
/// assert_eq!(int_val, Some(26));
///
/// let hex_str = "26";
/// let int_val = hstr_to_int(hex_str);
/// assert_eq!(int_val, Some(38));
/// ```
pub fn hstr_to_int(inp: &str) -> Option<usize> {
    // Convert hex string to integer
    let mut inp = inp;
    if inp.starts_with("0x") {
        inp = &inp[2..];
    }
    usize::from_str_radix(inp, 16).ok()
}

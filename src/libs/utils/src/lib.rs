use core_affinity::CoreId;
use errors::{Error, Result};

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

pub fn set_core_affinity(core_id: &CoreId) -> Result<()> {
    core_affinity::set_for_current(*core_id);
    Ok(())
}

pub fn hstr_to_int(inp: &str) -> Option<usize> {
    // Convert hex string to integer
    let mut inp = inp;
    if inp.starts_with("0x") {
        inp = &inp[2..];
    }
    usize::from_str_radix(inp, 16).ok()
}

use std::{
    collections::HashMap,
    error::Error,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use tokio::fs;

use crate::prelude::internal::*;

use super::{Cache, CpuDescription, Frequency};

/// Grabs info about the computer's processor(s).
#[tracing::instrument]
pub async fn cpu() -> GhrResult<Vec<ComponentInfo>> {
    use procfs::{CpuInfo, FromBufRead};
    use std::{fs::File, io::BufReader};

    // init the `procfs` side of things for model info
    let rdr = BufReader::new(
        File::open("/proc/cpuinfo")
            .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?,
    );
    let info = CpuInfo::from_buf_read(rdr)
        .map_err(|e| GhrError::ComponentInfoInaccessible(e.to_string()))?;

    // grab specific cpu info
    let cpu_info = cpu_info().await?;

    // parse out important values.
    // key is the processor id, value is a `CpuDescription`
    let mut cpus: HashMap<u32, Vec<Core>> = HashMap::new();

    // map each core to a processor
    for core in cpu_info.into_iter() {
        cpus.entry(core.processor_id).or_default().push(core);
    }

    // make each processor into a ComponentInfo
    let mut components = Vec::new();
    for (cpu_id, cores) in cpus {
        let mut cpu_best_speeds = Frequency {
            min: None,
            max: None,
        };

        let mut cpu_cache: Vec<Cache> = Vec::new();
        let mut cpu_cores: Vec<super::Core> = Vec::new();
        let core_ct = Some(cores.len() as u32);

        // convert each linux::Core into a cpu::Core
        for core in cores {
            // use the highest-clocked core as the "cpu" speed
            if core.speeds.max > cpu_best_speeds.max {
                cpu_best_speeds = core.speeds.clone();
            }

            // add core cache to the cpu cache
            for cache in core.cache.clone() {
                if let Ok(cache) = Cache::try_from(cache) {
                    cpu_cache.push(cache);
                }
            }

            cpu_cores.push(super::Core::from(core));
        }

        // now we'll actually make the ComponentInfo.
        //
        // let's start by grabbing required info
        let bus = ComponentBus::Sys;
        let id = info.model_name(cpu_id as usize).map(|s| s.to_string());
        let vendor_id = info.vendor_id(cpu_id as usize).map(|s| s.to_string());

        tracing::debug!("{vendor_id:?}");
        tracing::debug!("{id:?}");

        // create the ComponentDescription
        let desc = ComponentDescription::CpuDescription(CpuDescription {
            clock_speed: cpu_best_speeds,
            core_ct,
            cache: Some(cpu_cache),
            cores: Some(cpu_cores),
        });

        // finally, push it to the list
        components.push(ComponentInfo {
            bus,
            id,
            class: None,
            vendor_id,
            status: None,
            desc,
        });
    }

    Ok(components)
}

#[derive(Clone, Debug)]
struct Core {
    /// the processor index that this core belongs to
    processor_id: u32,
    /// the identifier for this core
    _core_num: u32,
    /// a list of caches belonging to this core
    cache: Vec<CoreCache>,
    /// the speeds for this core
    speeds: Frequency,
}

impl From<Core> for super::Core {
    #[tracing::instrument(skip(value))]
    fn from(value: Core) -> Self {
        Self {
            cache: Some(
                value
                    .cache
                    .into_iter()
                    .flat_map(super::Cache::try_from)
                    .collect(),
            ),
            speeds: value.speeds,
        }
    }
}

/// one of a core's cache, made from a folder like `cpu/coreN/cache/indexM/`
#[derive(Clone, Debug)]
struct CoreCache {
    /// the kind of cache we're working with
    level: u32,
    /// the size of this cache in kB
    size: u32,
    /// the kind of cache we're working with
    _kind: CacheKind,
}

impl TryFrom<CoreCache> for super::Cache {
    type Error = GhrError;

    #[tracing::instrument]
    fn try_from(value: CoreCache) -> Result<Self, Self::Error> {
        match value.level {
            1 => Ok(Cache::L1 {
                size: value.size,
                speed: None,
            }),
            2 => Ok(Cache::L2 {
                size: value.size,
                speed: None,
            }),
            3 => Ok(Cache::L3 {
                size: value.size,
                speed: None,
            }),
            weird => {
                tracing::error!(
                    "The cache level was reported as `{weird}`! This is an unexpected value."
                );
                Err(GhrError::ComponentInfoWeirdInfo(format!(
                    "Cache level was reported to be `{weird}`. This cache will be ignored."
                )))
            }
        }
    }
}

#[derive(Clone, Debug)]
enum CacheKind {
    /// cache that only holds instructions
    Instruction,
    /// cache that only caches data
    Data,
    /// cache that holds both data and instructions
    Unified,
}

#[tracing::instrument]
/// grabs info about CPU cores, then returns a vector of them.
///
/// # Errors
/// if no info is available, or a weird error occurs, it returns an `Err`
async fn cpu_info() -> GhrResult<Vec<Core>> {
    // the cpu dir in the linux `sysfs`
    let sysfs_cpu_path = PathBuf::from("/sys/devices/system/cpu");

    // let's see how many core's we're working with
    let core_ct = core_ct(&sysfs_cpu_path).await.map_err(|e| {
        tracing::error!("Failed to get CPU core count (err: {e}");
        GhrError::ComponentInfoInaccessible(format!(
            "Couldn't read info about the number of available cores. (err: {e}"
        ))
    })?;

    // iterate over each core
    let mut cores: Vec<Core> = Vec::new();
    for n in core_ct {
        // grab the core's `sysfs` path
        let core_path = sysfs_cpu_path.join(format!("cpu{n}"));

        // we can now read important info.
        // let's start with the cache
        let cache = core_cache(&core_path).await;

        // get the unique processor identifier, for systems with multiple
        // processors.
        //
        // sometimes known as the "socket" id
        let processor_id = core_processor_id(&core_path).await;

        // get the frequencies of this core
        let speeds = core_freq(&core_path).await;

        // create + insert the core into the list
        cores.push(Core {
            processor_id,
            _core_num: n,
            cache,
            speeds,
        })
    }

    Ok(cores)
}

/// gets the number of cores the system has.
///
/// # Errors
/// returns an `Err` if the file fails to read or parse
#[tracing::instrument]
async fn core_ct(sysfs_cpu_path: &Path) -> Result<RangeInclusive<u32>, Box<dyn Error>> {
    // read the file
    let file_path = sysfs_cpu_path.join("present");
    let core_ct_str = fs::read_to_string(file_path).await?;

    // the file has a format that's like: `0-7`, so we need to split on the `-`
    // and just read the numbers
    let (first, last) = {
        let split = core_ct_str.trim().split('-').collect::<Vec<_>>();

        // get the two strings from the split vec
        let get_strs = || -> Option<(&&str, &&str)> { Some((split.first()?, split.get(1)?)) };
        let (first_str, last_str) = get_strs().ok_or(GhrError::ComponentInfoWeirdInfo(
            "The parsed CPU index was unavailable.".into(),
        ))?;

        // parse them into numbers
        (first_str.parse::<u32>()?, last_str.parse::<u32>()?)
    };

    // return a range
    Ok(first..=last)
}

#[tracing::instrument(skip(sysfs_core_path))]
/// creates `CoreCache` structures (as many as needed for each entry).
///
/// note that you'll need to combine these for totals across the CPU. also,
/// this returns `None` if the `cache` directory doesn't exist for this CPU.
async fn core_cache(sysfs_core_path: &Path) -> Vec<CoreCache> {
    // ensure our path is correct
    debug_assert!(sysfs_core_path
        .to_string_lossy()
        .contains("/sys/devices/system/cpu/cpu"));

    // read into a stream
    let path = sysfs_core_path.join("cache");
    let Ok(mut entries) = fs::read_dir(path).await else {
        tracing::error!("Couldn't find any cache entries for this core.");
        return Vec::new(); // empty vec if there is no listing
    };

    let mut caches: Vec<CoreCache> = Vec::new();
    while let Ok(Some(cache)) = entries.next_entry().await {
        // only read if our path contains "index"
        let cache_path = cache.path();
        if !cache_path.to_string_lossy().contains("index") {
            continue;
        }

        // grab the level from file
        let Ok(level_str) = fs::read_to_string(cache_path.join("level")).await else {
            tracing::warn!("Failed to read CPU cache (`{cache_path:?}`) level for core.");
            continue;
        };

        // parse level
        let Ok(level) = level_str.trim().parse::<u32>() else {
            tracing::warn!(
                "CPU cache (`{cache_path:?}`) for core `{sysfs_core_path:?}` didn't have a number in its level file (contained `{level_str}`)."
            );
            continue;
        };

        // grab the size from file
        let Ok(mut size_str) = fs::read_to_string(cache_path.join("size")).await else {
            tracing::warn!(
                "Failed to read CPU cache (`{cache_path:?}`) size for core `{sysfs_core_path:?}`."
            );
            continue;
        };

        // get the size unit.
        //
        // note: i'm not aware if anything but "K" (KiB) is ever used, so this
        //       is static for now.
        let size_unit = 'K';
        let size_multiplier = 1_024_u32;

        // remove the unit from the str
        size_str = size_str.replace(size_unit, "");

        // parse size
        let Ok(size) = size_str.trim().parse::<u32>() else {
            tracing::warn!(
                        "CPU cache (`{cache_path:?}`) for core `{sysfs_core_path:?}` didn't have a number in its level file (contained `{level_str}`)."
                    );
            continue;
        };

        // finally, we'll grab the "type" (kind)
        let Ok(type_str) = fs::read_to_string(cache_path.join("type")).await else {
            tracing::warn!(
                "Failed to read CPU cache (`{cache_path:?}`) type for core `{sysfs_core_path:?}`."
            );
            continue;
        };

        let _kind = match type_str.trim() {
            "Instruction" => CacheKind::Instruction,
            "Data" => CacheKind::Data,
            "Unified" => CacheKind::Unified,
            unexpected => {
                tracing::error!("Encountered unexpected cache type for the CPU: `{unexpected}`.");
                continue;
            }
        };

        caches.push(CoreCache {
            level,
            size: size * size_multiplier,
            _kind,
        })
    }

    caches
}

/// finds a unique identifier for this cpu core's processor.
///
/// this is helpful for multiprocessor computing environments.
#[tracing::instrument]
async fn core_processor_id(sysfs_core_path: &Path) -> u32 {
    // read from disk
    let Ok(processor_id_str) =
        fs::read_to_string(sysfs_core_path.join("topology/physical_package_id")).await
    else {
        tracing::warn!("Failed to read processor ID for core. Assuming `0`.");
        return 0_u32; //  if the file isn't present, assume we're on processor 0
    };

    // parse it into a number
    processor_id_str.trim().parse::<u32>().unwrap_or_else(|e| {
        tracing::warn!("Failed to parse processor ID for core. Assuming `0`. (err: {e}");
        0_u32
    })
}

/// gets the min and max frequencies of the cpu.
#[tracing::instrument]
async fn core_freq(sysfs_core_path: &Path) -> Frequency {
    let path = sysfs_core_path.join("cpufreq");

    // frequencies, to return the corefrequency
    let mut min: Option<u32> = None;
    let mut max: Option<u32> = None;

    // grab + parse the min frequency
    if let Ok(min_str) = fs::read_to_string(path.join("cpuinfo_min_freq")).await {
        if let Ok(min_freq) = min_str.trim().parse::<u32>() {
            min = Some(min_freq / 1000_u32);
        }
    } else {
        tracing::warn!("Failed to read minimum frequency for core");
    }

    // ...and now the max freq
    if let Ok(max_str) = fs::read_to_string(path.join("cpuinfo_max_freq")).await {
        if let Ok(max_freq) = max_str.trim().parse::<u32>() {
            max = Some(max_freq / 1000_u32);
        }
    } else {
        tracing::warn!("Failed to read maximum frequency for core");
    }

    // return it, wrapped up all nice
    Frequency { min, max }
}

#[cfg(test)]
mod tests {
    use super::{core_freq, cpu_info};
    use std::path::PathBuf;

    #[tokio::test]
    async fn check_get_cores() {
        logger();

        let cores = cpu_info().await.unwrap();
        assert!(!cores.is_empty());
    }

    #[tokio::test]
    async fn check_freqs() {
        logger();

        let core = PathBuf::from("/sys/devices/system/cpu/cpu0");
        let freqs = core_freq(&core).await;

        // check that the freqs are in a good place
        assert!(freqs.max.unwrap() != 0_u32);
        assert!(freqs.max.unwrap() < 10_000_u32); // mhz
        _ = freqs.min.unwrap();
    }

    #[tracing::instrument]
    fn logger() {
        _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }
}

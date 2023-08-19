use std::io::Read;
use std::ops::RangeInclusive;

/// Reads stdin and retusn a string with its content.
pub fn read_from_stdin() -> Result<String, std::io::Error> {
    let mut stdin = Vec::new();
    tracing::event!(tracing::Level::INFO, "Reading from stdin...");
    let mut lock = std::io::stdin().lock();
    lock.read_to_end(&mut stdin)?;
    Ok(String::from_utf8_lossy(&stdin).to_string())
}

/// The range of values for the `temperature` option which goes from 0 to 1.
const TEMPERATURE_RANGE: RangeInclusive<f32> = 0.0..=2.0;
/// The range of values for the `top_p` option which goes from 0 to 1.
const TOP_P_RANGE: RangeInclusive<f32> = 0.0..=1.0;
/// The range of values for the `top_k` option which goes from 0 to Infinity.
const TOP_K_RANGE: RangeInclusive<f32> = 0.0..=f32::INFINITY;
/// The range of values for the `repetition_penalty` option wich goes from 0.0 to 8.0.
const REPETITION_PENALTY_RANGE: RangeInclusive<f32> = 0.0..=8.0;
/// The range of values for the `repetition_penalty_range` option wich goes from 0 to 2048.
const REPETITION_PENALTY_RANGE_RANGE: RangeInclusive<u32> = 0..=2048;

/// Parses the temperature value.
pub fn parse_temperature(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            TEMPERATURE_RANGE.start(),
            TEMPERATURE_RANGE.end()
        )
    })?;
    if !TEMPERATURE_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            TEMPERATURE_RANGE.start(),
            TEMPERATURE_RANGE.end()
        ));
    }
    Ok(value)
}

/// Parses the top_p value.
pub fn parse_top_p(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            TOP_P_RANGE.start(),
            TOP_P_RANGE.end()
        )
    })?;
    if !TOP_P_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            TOP_P_RANGE.start(),
            TOP_P_RANGE.end()
        ));
    }
    Ok(value)
}

/// Parse a single key-value pair
pub fn parse_key_val<T, U>(
    s: &str,
) -> Result<(T, U), Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid key-value pair: {}", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// Parses the top_k value.
pub fn parse_top_k(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            TOP_K_RANGE.start(),
            TOP_K_RANGE.end()
        )
    })?;
    if !TOP_K_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            TOP_K_RANGE.start(),
            TOP_K_RANGE.end()
        ));
    }
    Ok(value)
}

/// Parses the repetition_penalty value.
pub fn parse_repetition_penalty(s: &str) -> std::result::Result<f32, String> {
    let value = s.parse::<f32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            REPETITION_PENALTY_RANGE.start(),
            REPETITION_PENALTY_RANGE.end()
        )
    })?;
    if !REPETITION_PENALTY_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            REPETITION_PENALTY_RANGE.start(),
            REPETITION_PENALTY_RANGE.end()
        ));
    }
    Ok(value)
}

/// Parses the repetition_penalty_range value.
pub fn parse_repetition_penalty_range(s: &str) -> std::result::Result<u32, String> {
    let value = s.parse::<u32>().map_err(|_| {
        format!(
            "`{s}` must be a number between {} and {}",
            REPETITION_PENALTY_RANGE_RANGE.start(),
            REPETITION_PENALTY_RANGE_RANGE.end()
        )
    })?;
    if !REPETITION_PENALTY_RANGE_RANGE.contains(&value) {
        return Err(format!(
            "`{s}` must be a number between {} and {}",
            REPETITION_PENALTY_RANGE_RANGE.start(),
            REPETITION_PENALTY_RANGE_RANGE.end()
        ));
    }
    Ok(value)
}

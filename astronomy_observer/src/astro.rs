use crate::error::{err, AppResult};
use crate::models::{AstronomySample, BodyPosition, Location};
use chrono::{DateTime, TimeZone, Utc};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn calculate(location: &Location, times: &[DateTime<Utc>]) -> AppResult<Vec<AstronomySample>> {
    if times.is_empty() {
        return Ok(Vec::new());
    }
    let mut child = Command::new("/usr/local/bin/astro-helper")
        .arg(format!("{:.10}", location.latitude))
        .arg(format!("{:.10}", location.longitude))
        .arg(format!("{:.2}", location.elevation_m))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| err("could not open astronomy helper stdin"))?;
        for t in times {
            writeln!(stdin, "{}", t.timestamp())?;
        }
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(err(format!(
            "astronomy helper failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    parse_output(&String::from_utf8(output.stdout)?)
}

fn parse_output(text: &str) -> AppResult<Vec<AstronomySample>> {
    let mut map: BTreeMap<i64, AstronomySample> = BTreeMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split(',').collect();
        match p.first().copied() {
            Some("E") if p.len() == 5 => {
                let ts: i64 = p[1].parse()?;
                let time = Utc
                    .timestamp_opt(ts, 0)
                    .single()
                    .ok_or_else(|| err("invalid astronomy timestamp"))?;
                let entry = map.entry(ts).or_insert_with(|| AstronomySample {
                    time,
                    earth_ecliptic_au: [0.0; 3],
                    bodies: HashMap::new(),
                });
                entry.earth_ecliptic_au = [p[2].parse()?, p[3].parse()?, p[4].parse()?];
            }
            Some("B") if p.len() == 9 => {
                let ts: i64 = p[1].parse()?;
                let time = Utc
                    .timestamp_opt(ts, 0)
                    .single()
                    .ok_or_else(|| err("invalid astronomy timestamp"))?;
                let mag = p[7].parse::<f64>().ok().filter(|v| v.is_finite());
                let frac = p[8].parse::<f64>().ok().filter(|v| v.is_finite());
                let entry = map.entry(ts).or_insert_with(|| AstronomySample {
                    time,
                    earth_ecliptic_au: [0.0; 3],
                    bodies: HashMap::new(),
                });
                entry.bodies.insert(
                    p[2].to_string(),
                    BodyPosition {
                        ra_hours: p[3].parse()?,
                        dec_deg: p[4].parse()?,
                        azimuth_deg: p[5].parse()?,
                        altitude_deg: p[6].parse()?,
                        magnitude: mag,
                        illuminated_fraction: frac,
                    },
                );
            }
            _ => return Err(err(format!("unrecognized astronomy helper output: {line}"))),
        }
    }
    Ok(map.into_values().collect())
}

pub fn sample_nearest<'a>(
    samples: &'a [AstronomySample],
    t: DateTime<Utc>,
) -> Option<&'a AstronomySample> {
    samples
        .iter()
        .min_by_key(|s| (s.time.timestamp() - t.timestamp()).abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_helper_output() {
        let s = parse_output("E,0,1,2,3\nB,0,Moon,12,4,180,30,-10,0.25\n").unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].bodies["Moon"].altitude_deg, 30.0);
    }
}

use crate::config::AppConfig;
use crate::coordinates::{alt_az_from_j2000, angular_separation_deg};
use crate::models::{AstronomySample, Recommendation, SkyBrightness, WeatherSeries};
use crate::{scoring, weather};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct Shower{name:String,start:(u32,u32),peak:(u32,u32),end:(u32,u32),ra:f64,dec:f64,zhr:f64,speed:f64}

fn parse(path:&Path)->Vec<Shower>{
    let Ok(text)=fs::read_to_string(path) else{return Vec::new()};
    text.lines().skip(1).filter_map(|line|{
        let p:Vec<&str>=line.split(',').collect(); if p.len()<12{return None;}
        Some(Shower{name:p[0].to_string(),start:(p[2].parse().ok()?,p[3].parse().ok()?),peak:(p[4].parse().ok()?,p[5].parse().ok()?),end:(p[6].parse().ok()?,p[7].parse().ok()?),ra:p[8].parse().ok()?,dec:p[9].parse().ok()?,zhr:p[10].parse().ok()?,speed:p[11].parse().ok()?})
    }).collect()
}

fn dates_for_year(sh:&Shower,year:i32)->Option<(NaiveDate,NaiveDate,NaiveDate)>{
    let mut start=NaiveDate::from_ymd_opt(year,sh.start.0,sh.start.1)?;
    let mut peak=NaiveDate::from_ymd_opt(year,sh.peak.0,sh.peak.1)?;
    let mut end=NaiveDate::from_ymd_opt(year,sh.end.0,sh.end.1)?;
    if sh.start.0>sh.end.0 { if sh.peak.0<=sh.end.0{peak=NaiveDate::from_ymd_opt(year+1,sh.peak.0,sh.peak.1)?;end=NaiveDate::from_ymd_opt(year+1,sh.end.0,sh.end.1)?;} else {start=NaiveDate::from_ymd_opt(year-1,sh.start.0,sh.start.1)?;} }
    Some((start,peak,end))
}

fn active_peak(sh:&Shower,date:NaiveDate)->Option<(NaiveDate,f64)>{
    for y in [date.year()-1,date.year(),date.year()+1]{
        let (start,peak,end)=dates_for_year(sh,y)?;
        if date>=start&&date<=end{
            let dist=(date-peak).num_days().abs() as f64;
            let span=((end-start).num_days() as f64/5.0).max(1.0);
            return Some((peak,(-0.5*(dist/span).powi(2)).exp()));
        }
    } None
}

pub fn recommendations(cfg:&AppConfig,path:&Path,weather_series:&WeatherSeries,samples:&[AstronomySample],sky:&SkyBrightness,now:DateTime<Utc>,lat:f64,lon:f64)->Vec<Recommendation>{
    let showers=parse(path); let end=now+Duration::hours(cfg.options.observing_window_hours as i64); let mut out=Vec::new();
    for sh in showers{
        let mut winner=None;
        for sample in samples.iter().filter(|s|s.time>=now&&s.time<=end){
            let Some((peak,activity))=active_peak(&sh,sample.time.date_naive()) else{continue};
            let sun=sample.bodies.get("Sun").map(|x|x.altitude_deg).unwrap_or(90.0); if sun>-8.0{continue;}
            let (alt,az)=alt_az_from_j2000(sh.ra,sh.dec,lat,lon,sample.time); if alt<cfg.options.minimum_target_altitude.max(cfg.horizon.altitude_at(az)){continue;}
            let Some(w)=weather::nearest(weather_series,sample.time) else{continue}; let c=scoring::score_hour(w,sample,sky);
            let moon=sample.bodies.get("Moon");
            let (moon_pen,sep)=if let Some(m)=moon{let (rr,dd)=crate::coordinates::precess_j2000(sh.ra,sh.dec,sample.time);let s=angular_separation_deg(rr,dd,m.ra_hours*15.0,m.dec_deg);let p=m.illuminated_fraction.unwrap_or(0.5)*if m.altitude_deg>0.0{1.0}else{0.0}*(1.0-s/180.0).clamp(0.1,1.0);(p.clamp(0.0,0.9),Some(s))}else{(0.2,None)};
            let radiant=(alt/70.0).clamp(0.15,1.0); let activity_strength=(sh.zhr/100.0).clamp(0.15,1.0)*activity;
            let score=((c.overall/100.0*0.55+activity_strength*0.25+radiant*0.20)*(1.0-moon_pen*0.65)*100.0).clamp(0.0,100.0);
            let rec=Recommendation{name:sh.name.clone(),category:"meteor shower".to_string(),score,best_time:sample.time,altitude_deg:alt,azimuth_deg:az,magnitude:None,moon_separation_deg:sep,equipment:"naked eye".to_string(),note:format!("radiant {:.0}° high; nominal ZHR {:.0}; peak {}; speed {:.0} km/s",alt,sh.zhr,peak,sh.speed)};
            if winner.as_ref().map(|x:&Recommendation|score>x.score).unwrap_or(true){winner=Some(rec)}
        }
        if let Some(r)=winner{out.push(r)}
    } out
}

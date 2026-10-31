use crate::desktop::{Area, Monitor, Power, Window, parse_snowfall};
use snow_core::simulation::Snowfall;

#[derive(Debug, PartialEq)]
pub enum Command {
    Monitors(Vec<Monitor>),
    Windows { moving: bool, list: Vec<Window> },
    Elsewhere(Vec<u64>),
    Workspace,
    Cursor(Option<(f32, f32)>),
    Covered { all: bool, fullscreen: Vec<usize> },
    Snowfall(Snowfall),
    Accumulate(bool),
    Melt(bool),
    SnowOnWindows(bool),
    OverWindows(bool),
    CalmFps(u32),
    Sharp(bool),
    Power(Power),
    ClearSnow(bool),
    Halloween(bool),
    Slide(f32),
    Pause(bool),
    Quit,
}

pub fn parse(line: &str) -> Result<Command, String> {
    let mut words = line.split_whitespace();
    let verb = words.next().ok_or("pusta linia")?;
    let rest: Vec<&str> = words.collect();
    let bad = || format!("niezrozumiałe polecenie: {line}");
    let flag = |word: Option<&&str>| match word {
        Some(&"1") | Some(&"true") => Ok(true),
        Some(&"0") | Some(&"false") => Ok(false),
        _ => Err(bad()),
    };
    match verb {
        "monitors" => rest.iter().map(|entry| parse_monitor(entry).ok_or_else(bad)).collect::<Result<_, _>>().map(Command::Monitors),
        "windows" => {
            let moving = flag(rest.first())?;
            let list = rest.iter().skip(1).map(|entry| parse_window(entry).ok_or_else(bad)).collect::<Result<_, _>>()?;
            Ok(Command::Windows { moving, list })
        }
        "elsewhere" => rest.iter().map(|id| id.parse().map_err(|_| bad())).collect::<Result<_, _>>().map(Command::Elsewhere),
        "workspace" => Ok(Command::Workspace),
        "slide" => rest.first().and_then(|v| v.parse::<f32>().ok()).filter(|v| v.is_finite()).map(Command::Slide).ok_or_else(bad),
        "cursor" => match rest.as_slice() {
            ["none"] => Ok(Command::Cursor(None)),
            [x, y] => Ok(Command::Cursor(Some((x.parse().map_err(|_| bad())?, y.parse().map_err(|_| bad())?)))),
            _ => Err(bad()),
        },
        "covered" => {
            let all = flag(rest.first())?;
            let fullscreen = rest.iter().skip(1).map(|i| i.parse().map_err(|_| bad())).collect::<Result<_, _>>()?;
            Ok(Command::Covered { all, fullscreen })
        }
        "set" => match rest.as_slice() {
            ["snowfall", value] => parse_snowfall(value).map(Command::Snowfall).ok_or_else(bad),
            ["accumulate", _] => flag(rest.get(1)).map(Command::Accumulate),
            ["melt", _] => flag(rest.get(1)).map(Command::Melt),
            ["snow-on-windows", _] => flag(rest.get(1)).map(Command::SnowOnWindows),
            ["over-windows", _] => flag(rest.get(1)).map(Command::OverWindows),
            ["calm-fps", value] => value.parse::<u32>().map(|v| Command::CalmFps(v.clamp(10, 60))).map_err(|_| bad()),
            ["sharp", _] => flag(rest.get(1)).map(Command::Sharp),
            ["halloween", _] => flag(rest.get(1)).map(Command::Halloween),
            _ => Err(bad()),
        },
        "power" => match rest.as_slice() {
            ["saver"] => Ok(Command::Power(Power::Saver)),
            ["normal"] | ["full"] => Ok(Command::Power(Power::Full)),
            _ => Err(bad()),
        },
        "clear" => flag(rest.first()).map(Command::ClearSnow),
        "pause" => flag(rest.first()).map(Command::Pause),
        "quit" => Ok(Command::Quit),
        _ => Err(bad()),
    }
}

fn numbers(text: &str) -> Option<Vec<f64>> {
    text.split(',').map(|n| n.parse().ok()).collect()
}

fn parse_monitor(entry: &str) -> Option<Monitor> {
    let (index, values) = entry.split_once(':')?;
    let v = numbers(values)?;
    if !(v.len() == 5 || v.len() == 9) || v[2] <= 0.0 || v[3] <= 0.0 {
        return None;
    }
    let area = Area { x: v[0] as i32, y: v[1] as i32, width: v[2] as i32, height: v[3] as i32 };
    let bounds = if v.len() == 9 { Area { x: v[5] as i32, y: v[6] as i32, width: v[7] as i32, height: v[8] as i32 } } else { area };
    Some(Monitor { index: index.parse().ok()?, area, bounds, scale: v[4].clamp(0.5, 4.0) as f32 })
}

fn parse_window(entry: &str) -> Option<Window> {
    let (id, values) = entry.split_once(':')?;
    let v = numbers(values)?;
    if v.len() != 5 {
        return None;
    }
    Some(Window { id: id.parse().ok()?, x: v[0] as i32, y: v[1] as i32, width: v[2] as i32, height: v[3] as i32, flags: v[4] as u32 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_command() {
        assert_eq!(
            parse("monitors 0:0,32,1920,1048,1.25").unwrap(),
            Command::Monitors(vec![Monitor {
                index: 0,
                area: Area { x: 0, y: 32, width: 1920, height: 1048 },
                bounds: Area { x: 0, y: 32, width: 1920, height: 1048 },
                scale: 1.25
            }])
        );
        assert_eq!(
            parse("windows 1 7:10,20,300,400,8").unwrap(),
            Command::Windows { moving: true, list: vec![Window { id: 7, x: 10, y: 20, width: 300, height: 400, flags: 8 }] }
        );
        assert_eq!(
            parse("monitors 0:0,32,1920,1048,1,0,0,1920,1080").unwrap(),
            Command::Monitors(vec![Monitor {
                index: 0,
                area: Area { x: 0, y: 32, width: 1920, height: 1048 },
                bounds: Area { x: 0, y: 0, width: 1920, height: 1080 },
                scale: 1.0
            }])
        );
        assert_eq!(parse("clear 0").unwrap(), Command::ClearSnow(false));
        assert_eq!(parse("set halloween 0").unwrap(), Command::Halloween(false));
        assert_eq!(parse("slide -6000").unwrap(), Command::Slide(-6000.0));
        assert!(parse("slide nan").is_err());
        assert_eq!(parse("clear 1").unwrap(), Command::ClearSnow(true));
        assert_eq!(parse("windows 0").unwrap(), Command::Windows { moving: false, list: vec![] });
        assert_eq!(parse("elsewhere 3 4").unwrap(), Command::Elsewhere(vec![3, 4]));
        assert_eq!(parse("cursor 1.5 2").unwrap(), Command::Cursor(Some((1.5, 2.0))));
        assert_eq!(parse("cursor none").unwrap(), Command::Cursor(None));
        assert_eq!(parse("covered 0 1").unwrap(), Command::Covered { all: false, fullscreen: vec![1] });
        assert_eq!(parse("set snowfall Heavy").unwrap(), Command::Snowfall(Snowfall::Heavy));
        assert_eq!(parse("set melt 0").unwrap(), Command::Melt(false));
        assert_eq!(parse("set snow-on-windows 1").unwrap(), Command::SnowOnWindows(true));
        assert_eq!(parse("set over-windows 0").unwrap(), Command::OverWindows(false));
        assert_eq!(parse("set calm-fps 20").unwrap(), Command::CalmFps(20));
        assert_eq!(parse("set calm-fps 500").unwrap(), Command::CalmFps(60));
        assert_eq!(parse("power saver").unwrap(), Command::Power(Power::Saver));
        assert_eq!(parse("quit").unwrap(), Command::Quit);
        assert!(parse("windows x").is_err());
        assert!(parse("set snowfall blizzard").is_err());
        assert!(parse("monitors 0:1,2,3").is_err());
    }
}

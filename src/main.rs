use soloud::*;
use std::cmp::Ordering;
use std::env;
use std::io::Write;

use chrono::{format, Local, NaiveTime};

const MIN_WAKEUP_TIME: &str = "6:30";
const MAX_WAKEUP_TIME: &str = "8:22";
const SUNRISE_MODIFIER_FOR_WAKE_UP_TIME_IN_MINUTES: i64 = 15;
const DINNER_TIME_SINCE_SUNRISE_IN_HOURS: f32 = 11.5;
const SUNRISE_MODIFIER_FOR_BED_TIME_IN_HOURS: f32 = 15.5;

#[derive(PartialEq)]
enum TimestampType {
    WakeUpTime(NaiveTime),
    BedTime(NaiveTime),
    DinnerTime(NaiveTime),
}

impl std::fmt::Debug for TimestampType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimestampType::WakeUpTime(_) => write!(f, "Wake up time"),
            TimestampType::DinnerTime(_) => {
                write!(f, "Optimal evening dinner time")
            }
            TimestampType::BedTime(_) => write!(f, "Bedtime"),
        }
    }
}

impl TimestampType {
    fn get_naive_time(&self) -> NaiveTime {
        match self {
            TimestampType::WakeUpTime(v)
            | TimestampType::BedTime(v)
            | TimestampType::DinnerTime(v) => *v,
        }
    }
}

struct Timestamps {
    wake_up_time: TimestampType,
    bed_time: TimestampType,
    dinner_time: TimestampType,
}

impl Timestamps {
    fn new() -> Timestamps {
        let sunrise = gather_input();
        Timestamps {
            wake_up_time: TimestampType::WakeUpTime(
                sunrise
                    .overflowing_sub_signed(chrono::Duration::minutes(
                        SUNRISE_MODIFIER_FOR_WAKE_UP_TIME_IN_MINUTES,
                    ))
                    .0,
            ),
            dinner_time: TimestampType::DinnerTime(
                sunrise
                    .overflowing_add_signed(chrono::Duration::seconds(
                        (DINNER_TIME_SINCE_SUNRISE_IN_HOURS * 3600.0) as i64,
                    ))
                    .0,
            ),
            bed_time: TimestampType::BedTime(
                sunrise
                    .overflowing_add_signed(chrono::Duration::seconds(
                        (SUNRISE_MODIFIER_FOR_BED_TIME_IN_HOURS * 3600.0) as i64,
                    ))
                    .0,
            ),
        }
    }
    fn get_upcomming_timestamp(&self) -> &TimestampType {
        let now = Local::now().time();
        let mut upcomming_timestamp = &self.bed_time;
        if let TimestampType::DinnerTime(value) = self.dinner_time {
            if chrono::Duration::seconds(1).cmp(&now.signed_duration_since(value))
                == Ordering::Greater
            {
                upcomming_timestamp = &self.dinner_time;
            }
        }
        if let TimestampType::WakeUpTime(value) = self.wake_up_time {
            if chrono::Duration::seconds(1).cmp(&now.signed_duration_since(value))
                == Ordering::Greater
            {
                upcomming_timestamp = &self.wake_up_time;
            }
        }
        upcomming_timestamp
    }
    fn get_abs_time_diff(&self, first: NaiveTime, second: NaiveTime) -> chrono::Duration {
        second.signed_duration_since(first)
    }
}

fn gather_input() -> NaiveTime {
    let min_wakeup_time = NaiveTime::parse_from_str(MIN_WAKEUP_TIME, "%H:%M").unwrap();
    let max_wakeup_time = NaiveTime::parse_from_str(MAX_WAKEUP_TIME, "%H:%M").unwrap();
    if env::args().len() == 2 {
        NaiveTime::parse_from_str(
            env::args().collect::<Vec<String>>()[1]
                .replace("\n", "")
                .as_str(),
            "%H:%M",
        )
        .expect("Wrong parameter. Expected %H:%M (9:47) as first arg.")
        .clamp(min_wakeup_time, max_wakeup_time)
    } else {
        panic!("Wrong time format. Expected %H:%M (9:47) as first arg.")
    }
}

fn alert() {
    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("../audio.mp3")).unwrap();
    sl.play(&wav);
}

fn countdown_next_events(timestamps: Timestamps) {
    loop {
        let upcomming = timestamps.get_upcomming_timestamp();
        let now = Local::now().time();
        let diff_to_upcomming = timestamps.get_abs_time_diff(now, upcomming.get_naive_time());
        if diff_to_upcomming.num_hours() == 0
            && diff_to_upcomming.num_minutes() == 10
            && diff_to_upcomming.num_seconds() == 0
        {
            alert();
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
        print!(
            "{}",
            format!(
                "\r Upcomming event: '{:?}' in {:02}:{:02}:{:02}                                           ",
                upcomming,
                diff_to_upcomming.num_hours(),
                diff_to_upcomming.num_minutes() % 60,
                diff_to_upcomming.num_seconds() % 60
            )
        );
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn main() {
    let timestamps = Timestamps::new();

    print!(
        " Wake up time:   {} (-{}m)\n Evening dinner: {} (+{}h)\n Bed time:       {} (+{}h)\n",
        if let TimestampType::WakeUpTime(value) = timestamps.wake_up_time {
            value.format("%H:%M")
        } else {
            format::DelayedFormat::new(None, None, format::StrftimeItems::new("moin"))
        },
        SUNRISE_MODIFIER_FOR_WAKE_UP_TIME_IN_MINUTES,
        if let TimestampType::DinnerTime(value) = timestamps.dinner_time {
            value.format("%H:%M")
        } else {
            format::DelayedFormat::new(None, None, format::StrftimeItems::new("moin"))
        },
        DINNER_TIME_SINCE_SUNRISE_IN_HOURS,
        if let TimestampType::BedTime(value) = timestamps.bed_time {
            value.format("%H:%M")
        } else {
            format::DelayedFormat::new(None, None, format::StrftimeItems::new("moin"))
        },
        SUNRISE_MODIFIER_FOR_BED_TIME_IN_HOURS
    );

    countdown_next_events(timestamps);
}

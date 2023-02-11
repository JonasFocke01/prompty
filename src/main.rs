use std::io::Write;
use std::{cmp::Ordering, io};

use chrono::{format, Local, NaiveTime};

const MIN_WAKEUP_TIME: &str = "6:30";
const MAX_WAKEUP_TIME: &str = "8:22";
const SUNRISE_MODIFIER_FOR_WAKE_UP_TIME_IN_MINUTES: i64 = 15;
const OPTIMAL_EVENING_DINNER_TIME_SINCE_SUNRISE_IN_HOURS: f32 = 11.5;
const OPTIMAL_SPORTS_TIME_SINCE_SUNRISE_IN_HOURS: i64 = 11;
const SUNRISE_MODIFIER_FOR_BED_TIME_IN_HOURS: f32 = 15.5;

#[derive(PartialEq)]
enum TimestampType {
    WakeUpTime(NaiveTime),
    OptimalSportTime(NaiveTime),
    BedTime(NaiveTime),
    OptimalEveningDinnerTime(NaiveTime),
    Sunrise(NaiveTime),
}

impl std::fmt::Debug for TimestampType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimestampType::WakeUpTime(_) => write!(f, "Wake up time"),
            TimestampType::OptimalSportTime(_) => write!(f, "Optimal sports time"),
            TimestampType::OptimalEveningDinnerTime(_) => {
                write!(f, "Optimal evening dinner time")
            }
            TimestampType::BedTime(_) => write!(f, "Bedtime"),
            TimestampType::Sunrise(_) => write!(f, "Sunrise"),
        }
    }
}

impl TimestampType {
    fn get_naive_time(&self) -> NaiveTime {
        match self {
            TimestampType::WakeUpTime(v)
            | TimestampType::OptimalSportTime(v)
            | TimestampType::BedTime(v)
            | TimestampType::OptimalEveningDinnerTime(v)
            | TimestampType::Sunrise(v) => *v,
        }
    }
}

struct Timestamps {
    wake_up_time: TimestampType,
    optimal_sport_time: TimestampType,
    bed_time: TimestampType,
    optimal_evening_dinner_time: TimestampType,
    sunrise: TimestampType,
}

impl Timestamps {
    fn new() -> Timestamps {
        let sunrise = gather_input();
        Timestamps {
            sunrise: TimestampType::Sunrise(sunrise),
            wake_up_time: TimestampType::WakeUpTime(
                sunrise
                    .overflowing_sub_signed(chrono::Duration::minutes(
                        SUNRISE_MODIFIER_FOR_WAKE_UP_TIME_IN_MINUTES,
                    ))
                    .0,
            ),
            optimal_sport_time: TimestampType::OptimalSportTime(
                sunrise
                    .overflowing_add_signed(chrono::Duration::hours(
                        OPTIMAL_SPORTS_TIME_SINCE_SUNRISE_IN_HOURS,
                    ))
                    .0,
            ),
            optimal_evening_dinner_time: TimestampType::OptimalEveningDinnerTime(
                sunrise
                    .overflowing_add_signed(chrono::Duration::seconds(
                        (OPTIMAL_EVENING_DINNER_TIME_SINCE_SUNRISE_IN_HOURS * 3600.0) as i64,
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
        if let TimestampType::OptimalEveningDinnerTime(value) = self.optimal_evening_dinner_time {
            if chrono::Duration::seconds(1).cmp(&now.signed_duration_since(value))
                == Ordering::Greater
            {
                upcomming_timestamp = &self.optimal_evening_dinner_time;
            }
        }
        if let TimestampType::OptimalSportTime(value) = self.optimal_sport_time {
            if chrono::Duration::seconds(1).cmp(&now.signed_duration_since(value))
                == Ordering::Greater
            {
                upcomming_timestamp = &self.optimal_sport_time;
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

// Todo: this should be options instead of prompts
fn gather_input() -> NaiveTime {
    println!(" Please enter sunrise time");
    let mut sunrise = String::new();
    let min_wakeup_time = NaiveTime::parse_from_str(MIN_WAKEUP_TIME, "%H:%M").unwrap();
    let max_wakeup_time = NaiveTime::parse_from_str(MAX_WAKEUP_TIME, "%H:%M").unwrap();
    match io::stdin().read_line(&mut sunrise) {
        Ok(_) => {
            sunrise = sunrise.replace("\n", "");
            let sunrise = NaiveTime::parse_from_str(sunrise.as_str(), "%H:%M")
                .expect("The given input string has the wrong format. HH:MM expected");
            return sunrise.clamp(min_wakeup_time, max_wakeup_time);
        }
        Err(_) => panic!("wrong input"),
    }
}

fn countdown_next_events(timestamps: Timestamps) {
    loop {
        let upcomming = timestamps.get_upcomming_timestamp();
        let now = Local::now().time();
        let diff_to_upcomming = timestamps.get_abs_time_diff(now, upcomming.get_naive_time());
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

    print!(" Wake up time:                    {} (-{}m)\n Optimal time to sport:           {} (+{}h)\n Optimal time for evening dinner: {} (+{}h)\n Bed time:                        {} (+{}h)\n",
        if let TimestampType::WakeUpTime(value) = timestamps.wake_up_time { value.format("%H:%M") } else { format::DelayedFormat::new(None, None     , format::StrftimeItems::new("moin")) },
        SUNRISE_MODIFIER_FOR_WAKE_UP_TIME_IN_MINUTES,
        if let TimestampType::OptimalSportTime(value) = timestamps.optimal_sport_time { value.format("%H:%M") } else { format::DelayedFormat::new(None, None     , format::StrftimeItems::new("moin")) },
        OPTIMAL_SPORTS_TIME_SINCE_SUNRISE_IN_HOURS,
        if let TimestampType::OptimalEveningDinnerTime(value) = timestamps.optimal_evening_dinner_time { value.format("%H:%M") } else { format::DelayedFormat::new(None, None     , format::StrftimeItems::new("moin")) },
        OPTIMAL_EVENING_DINNER_TIME_SINCE_SUNRISE_IN_HOURS,
        if let TimestampType::BedTime(value) = timestamps.bed_time { value.format("%H:%M") } else { format::DelayedFormat::new(None, None     , format::StrftimeItems::new("moin")) },
        SUNRISE_MODIFIER_FOR_BED_TIME_IN_HOURS
    );

    countdown_next_events(timestamps);
}

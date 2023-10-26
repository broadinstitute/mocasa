use std::time::{Duration, SystemTime};
use crate::train::param_meta_stats::Summary;
use crate::util::duration_format::format_duration;

pub(crate) struct Reporter {
    start_time: SystemTime,
    start_time_round: SystemTime,
    time_since_last_report: SystemTime
}

const TIME_BETWEEN_OPTIONAL_REPORTS_SECS: u64 = 10;

impl Reporter {
    pub(crate) fn new() -> Reporter {
        let start_time = SystemTime::now();
        let start_time_round = SystemTime::now();
        let time_since_last_report = SystemTime::now();
        Reporter { start_time, start_time_round, time_since_last_report }
    }
    pub(crate) fn reset_round_timer(&mut self) {
        self.start_time_round = SystemTime::now();
    }
    pub(crate) fn maybe_report(&mut self, summary: Summary, i_cycle: usize, i_iteration: usize,
                               n_steps_per_iteration: usize) {
        let secs_passed =
            self.time_since_last_report.elapsed().unwrap_or(Duration::ZERO).as_secs();
        if secs_passed >= TIME_BETWEEN_OPTIONAL_REPORTS_SECS {
            self.report(summary, i_cycle, i_iteration, n_steps_per_iteration)
        }
    }
    pub(crate) fn report(&mut self, summary: Summary, i_cycle: usize, i_iteration: usize,
                         n_steps_per_iteration: usize) {
        let duration_round =
            self.start_time_round.elapsed().unwrap_or(Duration::ZERO);
        let secs_elapsed = (duration_round.as_millis() as f64) / 1000.0;
        let elapsed_round = format_duration(duration_round);
        let duration_total = self.start_time.elapsed().unwrap_or(Duration::ZERO);
        let elapsed_total = format_duration(duration_total);
        let steps_per_sec = (n_steps_per_iteration as f64) / secs_elapsed;
        println!("Cycle {}, iteration {}: completed {} steps in {}, \
        which is {} iterations per second and thread. Total time is {}",
                 i_cycle, i_iteration, n_steps_per_iteration, elapsed_round, steps_per_sec,
                 elapsed_total);
        println!("{}", summary);
        self.time_since_last_report = SystemTime::now();
    }
}

fn duration_or_zero<E>(duration: Result<Duration, E>) -> Duration {
    duration.unwrap_or(Duration::ZERO)
}
//! waycat — runcat module for polybar (or waybar).
//! Rust rewrite of https://github.com/zzqmt/polycat

mod args;
mod cpu;
mod formatter;
mod framer;
mod rate_poll;
mod smoother;

use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use clap::Parser;
use formatter::Formatter;
use framer::Framer;
use rate_poll::RatePoll;
use smoother::Smoother;

fn get_period(low_rate: u64, high_rate: u64, cpu_load: f32) -> u64 {
    let diff = high_rate - low_rate;
    let rate = low_rate + (cpu_load * diff as f32) as u64;
    1000 / rate.max(1)
}

fn run(args: &args::Args) -> ExitCode {
    if let Err(errors) = args.validate() {
        for msg in errors {
            eprintln!("Config error: {}", msg);
        }
        return ExitCode::FAILURE;
    }

    let mut formatter = Formatter::new();
    if let Err(e) = formatter.set(&args.format) {
        eprintln!("Format error: {}", e);
        return ExitCode::FAILURE;
    }

    let mut framer = Framer::new(&args.frames);
    let mut sleeping_framer = Framer::new(&args.sleeping_frames);
    let rate_poll = RatePoll::new(args.poll_period, &args.stat_path);
    let mut smoother = Smoother::new(args.smoothing_value);

    let low_rate = args.low_rate as u64;
    let high_rate = args.high_rate as u64;

    let poll_handle = {
        let rp = rate_poll.clone();
        thread::spawn(move || rp.run())
    };

    let mut period_prev = get_period(low_rate, high_rate, 0.0);
    let mut sleeping = false;

    loop {
        let point = std::time::Instant::now();

        if let Some(what) = rate_poll.io_err() {
            eprintln!("{}: CPU polling error: {}", args.stat_path, what);
            break;
        }
        if let Some(what) = rate_poll.fmt_err() {
            eprintln!("{}: {}", args.stat_path, what);
            break;
        }

        let load = rate_poll.poll();
        smoother.set_target(load);
        let load_smoothed = smoother.value(period_prev);
        let load_displayed = if args.smoothing_enabled {
            load_smoothed
        } else {
            load
        };

        // Change sleeping state.
        if !sleeping {
            sleeping =
                args.sleeping_enabled && load_displayed <= args.sleeping_threshold as f32 / 100.0;
        } else {
            sleeping = load_displayed <= args.wakeup_threshold as f32 / 100.0;
        }

        let frame;
        let period;
        if !sleeping {
            period = get_period(low_rate, high_rate, load_displayed);
            period_prev = period;
            frame = framer.get();
        } else {
            period = 1000 / args.sleeping_rate as u64;
            frame = sleeping_framer.get();
        }

        if args.format_enabled {
            let format_load = (load * 100.0).round() as u8;
            println!("{}", formatter.format(&frame, format_load));
        } else {
            println!("{}", frame);
        }

        let target = point + Duration::from_millis(period);
        let now = std::time::Instant::now();
        if now < target {
            thread::sleep(target - now);
        }
    }

    poll_handle.join().unwrap();
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let args = args::Args::parse();
    run(&args)
}
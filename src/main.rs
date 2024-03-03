// Read space separated valus from log
use clap::Parser;
use hound;
use plotters::backend::BitMapBackend;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::ChartBuilder;
use plotters::prelude::{LineSeries, RED, WHITE};
use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_values(log_file_name: &String) -> Vec<(f64, f64)> {
    let mut prev: Option<f64> = None;
    let mut result: Vec<(f64, f64)> = Vec::new();
    // open a file
    if let Ok(lines) = read_lines(log_file_name) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            let mut values = line.split_whitespace();
            let x: f64 = values.next().unwrap().parse().unwrap();
            let y: f64 = values.next().unwrap().parse().unwrap();
            if prev.is_some() {
                result.push((x, prev.unwrap()));
            }
            prev = Some(y);
            result.push((x, y))
        }
    }
    result
}

fn generate_png(log_file_name: &String, png_file_name: &String) {
    // Your data points
    let data = read_values(log_file_name);
    let min_x = data.iter().next().unwrap().0;
    let max_x = data.iter().last().unwrap().0;
    let min_y = *data
        .iter()
        .map(|(_, y)| y)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_y = *data
        .iter()
        .map(|(_, y)| y)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let w = (max_x - min_x) * 3000.0 / (max_y - min_y);

    let root_area = BitMapBackend::new(png_file_name, (w as u32, 1280)).into_drawing_area();

    root_area.fill(&WHITE).unwrap();

    let root_area = root_area.titled(log_file_name, ("sans-serif", 60)).unwrap();

    let mut cc = ChartBuilder::on(&root_area)
        .margin(5)
        .set_all_label_area_size(20)
        .build_cartesian_2d(min_x..max_x, min_y..max_y * 1.05)
        .unwrap();

    cc.configure_mesh()
        .x_labels(max_x as usize + 1)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()
        .unwrap();
    //
    // And we can draw something in the drawing area
    cc.draw_series(LineSeries::new(data, &RED)).unwrap();

    // To avoid the IO failure being ignored silently, we manually call the present function
    root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
}

fn generate_wav(log_file_name: &String, wav_file_name: &String) {
    let data = read_values(log_file_name);
    let mut result: Vec<f64> = Vec::new();
    let max_x = data.iter().last().unwrap().0;
    let max_y = *data
        .iter()
        .map(|(_, y)| y)
        .max_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap())
        .unwrap();

    let mut iter = data.iter();
    let mut y = iter.next().unwrap().1;
    let mut curr = iter.next().unwrap();
    let mut t = 0.0;
    let p = 1.0 / 192000.0;
    result.push(y);
    while t <= max_x {
        t += p;
        if t <= curr.0 {
            result.push(y);
            continue;
        }
        while t > curr.0 {
            let next = iter.next();
            if next.is_none() {
                break;
            }
            curr = next.unwrap();
            y = curr.1;
        }
    }
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 192000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(wav_file_name, spec).unwrap();
    let ratio = 32768.0 / max_y;
    result
        .iter()
        .map(|y| (ratio * y) as i16)
        .for_each(|value| writer.write_sample(value).unwrap());

    writer.finalize().unwrap();
}

#[derive(Parser)]
#[clap(name = "My Program")]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    #[clap(about = "Generates a plot")]
    Plot(Plot),
    #[clap(about = "Generates a wav file")]
    Wav(Wav),
}

#[derive(Parser)]
struct Plot {
    #[clap(help = "The input file to use")]
    filename: String,
    #[clap(long, help = "The output file to use")]
    output: Option<String>,
}

#[derive(Parser)]
struct Wav {
    #[clap(help = "The input file to use")]
    filename: String,
    #[clap(long, help = "The output file to use")]
    output: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.command {
        Command::Plot(plot) => {
            println!("Plotting from file: {}", plot.filename);
            if let Some(ref o) = plot.output {
                println!("Outputting to file: {}", o);
            }
            generate_png(
                &plot.filename,
                &plot.output.unwrap_or("output.png".to_string()),
            );
        }
        Command::Wav(wav) => {
            println!("Generating wav from file: {}", wav.filename);
            if let Some(ref o) = wav.output {
                println!("Outputting to file: {}", o);
            }
            generate_wav(
                &wav.filename,
                &wav.output.unwrap_or("output.wav".to_string()),
            );
        }
    }
}

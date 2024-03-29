// Read space separated valus from log
use clap::Parser;
use hound;
use plotters::backend::BitMapBackend;
use plotters::chart::SeriesLabelPosition;
use plotters::drawing::IntoDrawingArea;
use plotters::element::PathElement;
use plotters::prelude::ChartBuilder;
use plotters::prelude::{LineSeries, WHITE};
use plotters::style::{register_font, Color, RGBColor, TextStyle, BLACK};
use rodio::{OutputStream, Sink};
use std::collections::VecDeque;
use std::io::Write;
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

fn generate_png(log_file_names: &Vec<String>, png_file_name: &String, zoom: f64, start: Option<f64>) {
    let colors = vec![
        RGBColor(200, 0, 0),
        RGBColor(0, 150, 0),
        RGBColor(0, 0, 150),
        RGBColor(0, 150, 150),
        RGBColor(200, 0, 150),
        RGBColor(200, 150, 0),
        RGBColor(50, 50, 50),
    ];
    let _ = register_font("sans-serif", plotters::style::FontStyle::Normal, 
    include_bytes!("Montserrat-Regular.ttf"));
    
    let mut all_data: VecDeque<Vec<(f64, f64)>> = log_file_names.iter().map(|name| read_values(name)).collect();

    let min_x = start.unwrap_or(
        all_data.iter().map(|data| data.iter().next().unwrap().0).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
    );
    let max_x = all_data.iter().map(|data| data.iter().last().unwrap().0).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let min_y = all_data.iter().map(|data| *data.iter().map(|(_, y)| y).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_y = all_data.iter().map(|data| *data.iter().map(|(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();   

    let w = (max_x - min_x) * 3000.0 *zoom / (max_y - min_y);
    let w = w as u32;
    let h = (1280.0 * zoom) as u32;

  
    let root_area = BitMapBackend::new(png_file_name, (w, h)).into_drawing_area();

    root_area.fill(&WHITE).unwrap();

    let mut cc = ChartBuilder::on(&root_area)
        .margin(5)
        .set_all_label_area_size(20)
        .build_cartesian_2d(min_x..max_x, min_y..max_y * 1.05)
        .unwrap();

    cc.configure_mesh()
        .x_labels((w / 50) as usize)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()
        .unwrap();

    let mut counter = 0;
    while let Some(data) = all_data.pop_front() {
        let color = colors[counter].clone();
        cc.draw_series(LineSeries::new(data, colors[counter].stroke_width(2)))
            .unwrap()
            .label(log_file_names.get(counter).unwrap())
            .legend(move|(x,y)| PathElement::new(vec![(x,y), (x + 20,y)], &color));
        counter += 1;
    }

    let style: TextStyle= ("sans-serif", 20 * zoom as i32).into();
    cc.configure_series_labels()
    .position(SeriesLabelPosition::LowerLeft)
    .label_font(style)
    .background_style(&WHITE)
    .border_style(&BLACK)
    .draw().unwrap();

    // To avoid the IO failure being ignored silently, we manually call the present function
    root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
}

fn generate_samples(log_file_name: &String, sampling_freq: i32, start: Option<f64>) -> Vec<i16>  {
    let data = read_values(log_file_name);
    let mut result: Vec<f64> = Vec::new();
    let max_x = data.iter().last().unwrap().0;
    let max_y = *data
        .iter()
        .map(|(_, y)| y)
        .max_by(|a, b| a.abs().partial_cmp(&b.abs()).unwrap())
        .unwrap();

    let start = start.unwrap_or(0.0);
    println!("Start time: {}", start);
    let mut iter = data.iter();
    let mut y = iter.next().unwrap().1;
    let mut curr = iter.next().unwrap();
    let mut t = 0.0;
    let p = 1.0 / sampling_freq as f64;
    if t >= start { 
        result.push(y);
    }
    while t <= max_x {
        t += p;
        if t <= curr.0 {
            if t >= start { 
                result.push(y);
            }
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
    let ratio = 32768.0 / max_y;
    let mut samples: Vec<i16> = Vec::new();
    result
        .iter()
        .map(|y| (ratio * y) as i16)
        .for_each(|value| samples.push(value));
    println!("End time: {}", t);
    samples
}

fn generate_wav(log_file_name: &String, wav_file_name: &String, start: Option<f64>) {
    let samples = generate_samples(log_file_name, 192000, start);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 192000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(wav_file_name, spec).unwrap();
    for sample in samples {
        writer.write_sample(sample).unwrap();
    }

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
    #[clap(about = "Generates a C++ array")]
    CppArray(CppArray),
    #[clap(about = "Plays the sound")]
    Play(Play),
}

#[derive(Parser)]
struct Play {
    #[clap(help = "The input file to use")]
    filenames: Vec<String>,
    #[clap(long, help = "Start time")]
    #[arg(short, long)]
    start: Option<f64>,
}

#[derive(Parser)]
struct Plot {
    #[clap(help = "The input file to use")]
    filenames: Vec<String>,
    #[clap(help = "The zoom factor", name = "zoom")]
    #[arg(long)]
    zoom: Option<f64>,
    #[clap(long, help = "The output file to use")]
    #[arg(short, long)]
    output: Option<String>,
    #[clap(long, help = "Start time")]
    #[arg(short, long)]
    start: Option<f64>,
}

#[derive(Parser)]
struct Wav {
    #[clap(help = "The input file to use")]
    filename: String,
    #[clap(long, help = "The output file to use")]
    #[arg(short, long)]
    output: Option<String>,
    #[clap(long, help = "Start time")]
    #[arg(short, long)]
    start: Option<f64>,
}

#[derive(Parser)]
struct CppArray {
    #[clap(help = "The input file to use")]
    filename: String,
    #[clap(long, help = "The output file to use")]
    #[arg(short, long)]
    output: Option<String>,
    #[clap(long, help = "Start time")]
    #[arg(short, long)]
    start: Option<f64>,
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.command {
        Command::Plot(plot) => {
            //println!("Plotting from file: {}", plot.filename);
            let filename = &plot.output.unwrap_or("output.png".to_string());
            println!("Outputting to file: {}", filename);
            generate_png(
                &plot.filenames,
                filename,
                plot.zoom.unwrap_or(1.0),
                plot.start,
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
                wav.start,
            );
        }
        Command::CppArray(array) => {
            println!("Generating C++ array from file: {}", array.filename);
            if let Some(ref o) = array.output {
                println!("Outputting to file: {}", o);
            }
            let samples = generate_samples(&array.filename, 48000, array.start);
            let mut file = File::create(array.output.unwrap_or("output.cpp".to_string())).unwrap();
            file.write_all(b"const int16_t samples[] = {").unwrap();
            if let Some((last, elements)) = samples.split_last() {
                for (i, sample) in elements.iter().enumerate() {
                    file.write_all(format!("{}, ", sample).as_bytes()).unwrap();
                    if (i + 1) % 16 == 0 {
                        file.write_all(b"\n").unwrap();
                    }
                }
                file.write_all(format!("{}", last).as_bytes()).unwrap();
            }
                
            file.write_all(b"};").unwrap();
        },
        Command::Play(play) => {
            println!("Playing from file: {:?}", play.filenames);
            let samples = generate_samples(&play.filenames[0], 192000, play.start);
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let source = rodio::buffer::SamplesBuffer::new(1, 192000, samples);

            sink.append(source);
            sink.sleep_until_end();
        }
    }
}

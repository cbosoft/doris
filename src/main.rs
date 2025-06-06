mod app;
mod command_box;
mod event_handler;
mod frame_renderable;
mod keyboard;
mod patch;
mod sequence;
mod track;

use assert_no_alloc::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::hacker::*;

use patch::Patch;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("Failed to find a default output device");
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
        _ => panic!("Unsupported format"),
    }
}


fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f64;
    let channels = config.channels as usize;

    let patch = Patch::new();
    for r in patch.branch_reprs()? {
        eprintln!("{r}");
    }
    let pn = patch.create_net().unwrap();
    // let p = unit::<U2, U1>(Box::new(pn));

    //let pitch = shared(150.0);

    let mut net = Net::new(0, 2);
    // let (mut s, su) = snoop(32);
    // net.chain(Box::new(
    //         (var(&pitch) | constant(1.0)) >> p >> pan(0.0)
    // ));

    net.check();
    net.set_sample_rate(sample_rate);
    let mut backend = BlockRateAdapter::new(Box::new(net.backend()));
    let mut next_value = move || assert_no_alloc(|| backend.get_stereo() );

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;
    // for _ in 0..100 {
    //     std::thread::sleep(Duration::from_secs(1));
    //     match s.get() {
    //         Some(b) => {
    //             let values: Vec<_> = (0..b.len()).into_iter().map(|i| format!("{:.1}", b.at(i))).collect();
    //             let line = values.join(", ");
    //             eprintln!("{line}");
    //         }
    //         None => (),
    //     }
    // }

    app::App::new(net, sample_rate).run().unwrap();
    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right: T = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel & 1 == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}

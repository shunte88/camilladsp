use crate::filters::Filter;
use config;
use fifoqueue::FifoQueue;

use PrcFmt;
use Res;

#[derive(Copy, Clone, Debug)]
pub struct Gain {
    pub gain: PrcFmt,
}

pub struct Delay {
    pub queue: FifoQueue<PrcFmt>,
}


impl Gain {
    /// A simple filter providing gain in dB, and can also invert the signal.
    pub fn new(gain_db: PrcFmt, inverted: bool) -> Self {
        let mut gain: PrcFmt = 10.0;
        gain = gain.powf(gain_db/20.0);
        if inverted {
            gain = -gain;
        }
        Gain {
            gain: gain,
        }
    }

    pub fn from_config(conf: config::GainParameters) -> Self {
        let gain = conf.gain;
        let inverted = conf.inverted;
        Gain::new(gain, inverted)
    }
}

impl Filter for Gain {
    fn process_waveform(&mut self, waveform: &mut Vec<PrcFmt>) -> Res<()> {
        for n in 0..waveform.len() {
            waveform[n] = self.gain*waveform[n];
        }
        Ok(())
    }
}

impl Delay {
    /// Creates a delay filter with delay in samples
    /// Will be improved as it gets slow for long delays
    pub fn new(delay: usize) -> Self {
        let mut queue = FifoQueue::filled_with(delay+1, 0.0);
        let _elem = queue.pop();
        Delay {
            queue: queue,
        }
    }


    pub fn from_config(samplerate: usize, conf: config::DelayParameters) -> Self {
        let delay_samples = (conf.delay/1000.0 * (samplerate as PrcFmt)) as usize;
        Delay::new(delay_samples)
    }
}

impl Filter for Delay {
    fn process_waveform(&mut self, waveform: &mut Vec<PrcFmt>) -> Res<()> {
        for n in 0..waveform.len() {
            self.queue.push(waveform[n])?;
            waveform[n] = self.queue.pop().unwrap();
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use basicfilters::{Gain, Delay};
    use filters::Filter;

    #[test]
    fn gain_invert() {
        let mut waveform = vec![-0.5, 0.0, 0.5];
        let waveform_inv = vec![0.5, 0.0, -0.5];
        let mut gain = Gain::new(0.0, true);
        gain.process_waveform(&mut waveform).unwrap();
        assert_eq!(waveform, waveform_inv);
    }

    #[test]
    fn gain_ampl() {
        let mut waveform = vec![-0.5, 0.0, 0.5];
        let waveform_ampl = vec![-5.0, 0.0, 5.0];
        let mut gain = Gain::new(20.0, false);
        gain.process_waveform(&mut waveform).unwrap();
        assert_eq!(waveform, waveform_ampl);
    }

    #[test]
    fn delay_small() {
        let mut waveform = vec![0.0, -0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let waveform_delayed = vec![0.0, 0.0, 0.0, 0.0, -0.5, 1.0, 0.0, 0.0];
        let mut delay = Delay::new(3);
        delay.process_waveform(&mut waveform).unwrap();
        assert_eq!(waveform, waveform_delayed);
    }

    #[test]
    fn delay_large() {
        let mut waveform1 = vec![0.0, -0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut waveform2 = vec![0.0; 8];
        let waveform_delayed = vec![0.0, 0.0, -0.5, 1.0, 0.0, 0.0, 0.0, 0.0];
        let mut delay = Delay::new(9);
        delay.process_waveform(&mut waveform1).unwrap();
        delay.process_waveform(&mut waveform2).unwrap();
        assert_eq!(waveform1, vec![0.0; 8]);
        assert_eq!(waveform2, waveform_delayed);
    }




}
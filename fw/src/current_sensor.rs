use core::ptr;
use heapless::Deque;
use stm32f3xx_hal::adc::{Adc, OneShot, VoltageInternalReference};
use stm32f3xx_hal::pac::{ADC1, ADC1_2};
use stm32f3xx_hal::prelude::_embedded_hal_adc_OneShot;
use crate::board;

type ValuesRing = Deque<f32, 10>;

pub struct CurrentReader {
    adc_current: Adc<ADC1, OneShot>,
    v_in: VoltageInternalReference<ADC1_2>,
    current_pin: board::Current,
}

impl CurrentReader {
    pub fn new(adc_current: Adc<ADC1, OneShot>, v_in: VoltageInternalReference<ADC1_2>, current_pin: board::Current) -> Self {
        CurrentReader{adc_current, v_in, current_pin}
    }

    pub fn read(&mut self ) -> f32 {
        let value: u16 = self.adc_current.read(&mut self.current_pin).unwrap();
        let vref: u16 = self.adc_current.read(&mut self.v_in).unwrap();
        let vrefint_cal_ptr = 0x1FFF_F7BA as *const u16;
        let vrefint_cal = unsafe { ptr::read(vrefint_cal_ptr) };
        (3.3 * (vrefint_cal as f32) * (value as f32)) / (vref as f32 * 4095.0) // Convert to volts
    }
}

pub struct CurrentSensor {
    sensor_values: ValuesRing,
}

fn average(values: &ValuesRing) -> Option<f32>{
    if values.is_empty() || values.len() < 100 {
        None
    } else {
        Some(values.iter().sum::<f32>()/values.len() as f32)
    }
}

impl CurrentSensor {
    pub fn new() -> Self {
        CurrentSensor {sensor_values: Deque::new()}
    }

    pub fn add_value(&mut self, volts: f32) {
        if self.sensor_values.is_full() {
            self.sensor_values.pop_front();
        }
        if let Some(last_value) = self.sensor_values.front() {
            self.sensor_values.push_back((last_value*0.99) + (1.0-0.99)*volts).unwrap_or_default();
        } else {
            self.sensor_values.push_back(volts).unwrap_or_default();
        }
        //defmt::println!("Current averaged value: {}", self.sensor_values.iter().last().unwrap_or(&-1.0));
        /*let mut it = self.sensor_values.iter().rev();
        let last_value =  it.next().unwrap_or(&-1.0);
        let second_last_value =  it.next().unwrap_or(&-1.0);*/
        /*if last_value - second_last_value >= 0.002 {
            defmt::println!("Motor is on!")
        }
        if second_last_value - last_value >= 0.002 {
            defmt::println!("Motor is off!")
        }*/
    }

    pub fn get_sensor(&self) -> f32 {
        average(&self.sensor_values).unwrap_or(-1.0)
    }

    pub fn is_error(&self) -> bool {
        //average(&self.sensor_values).map(|v| v < 0.01).unwrap_or(false)
        false
    }

    pub fn is_standby(&self) -> bool {
        //defmt::println!("Idel value: {}, running value: {}", self.idle_current, average(&self.sensor_values));
        //average(&self.sensor_values).map(|v| fabsf(v - self.idle_current) < 0.001).unwrap_or(true)
        true
    }

    pub fn is_running(&self) -> bool {
        //defmt::println!("Idel value: {}, running value: {}", self.idle_current, average(&self.sensor_values));
        //average(&self.sensor_values).map(|v| fabsf(v - self.idle_current) < 0.005).unwrap_or(false)
        true
    }

    pub fn is_overloaded(&self) -> bool {
        //average(&self.sensor_values).map(|v| fabsf(v - self.idle_current)>0.1 && !self.is_error()).unwrap_or(false)
        false
    }

}
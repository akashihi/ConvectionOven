use heapless::Deque;
use max31855::{Max31855, Unit};
use crate::board::{SpiBus, TcCs};

type ValuesRing = Deque<f32, 10>;

const TEMP_OFFSET: f32 = 5.0;

pub struct TempSensor {
    tc_cs: TcCs,
    tc_spi: SpiBus,
    sensor_values: ValuesRing,
    internal_values: ValuesRing,
    error: u8
}

fn average(values: &ValuesRing) -> Option<f32>{
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f32>()/values.len() as f32)
    }
}

impl TempSensor {
    pub fn new(tc_cs: TcCs, tc_spi: SpiBus) -> Self {
        TempSensor{tc_cs, tc_spi, sensor_values: Deque::new(), internal_values: Deque::new(), error: 0}
    }

    pub fn poll_sensor(&mut self) {
        match self.tc_spi.read_all(&mut self.tc_cs, Unit::Celsius) {
            Ok(v) => {
                if self.sensor_values.is_full() {
                    self.sensor_values.pop_front();
                }
                self.sensor_values.push_back(v.thermocouple + TEMP_OFFSET).unwrap_or_default();
                if self.internal_values.is_full() {
                    self.internal_values.pop_front();
                }
                self.internal_values.push_back(v.internal).unwrap_or_default();
                self.error = 0;
            },
            Err(_) => self.error += 1
        }
    }

    pub fn get_sensor(&self) -> Option<f32> {
        average(&self.sensor_values)
    }
    
    pub fn get_internal_temperature(&self) -> Option<f32> {
        average(&self.internal_values)
    }

    pub fn is_error(&self) -> bool {
        self.error > 20 //2 consecutive second of unresponsive sensor means error
    }

    pub fn is_overheating(&self) -> bool {
        //defmt::println!("Inner temp: {}", average(&self.internal_values));
        average(&self.internal_values).map(|v| v> 60.0).unwrap_or(false) //60 on the thermocouple driver means that ambient temperature is too high for TRIACs
    }
}
use crate::button::Button;
use crate::hal::gpio::PinId;
use crate::{NUM_CHANNELS, NUM_STEPS, Step, VELOCITY_LEVELS};

pub struct StepButtonSet<
    P00: PinId,
    P01: PinId,
    P02: PinId,
    P03: PinId,
    P04: PinId,
    P05: PinId,
    P06: PinId,
    P07: PinId,
    P08: PinId,
    P09: PinId,
    P10: PinId,
    P11: PinId,
    P12: PinId,
    P13: PinId,
    P14: PinId,
    P15: PinId,
    
    V00: PinId,
    V01: PinId,
    V02: PinId,
    V03: PinId,
> 
{
    pub b00: Button<P00>,
    pub b01: Button<P01>,
    pub b02: Button<P02>,
    pub b03: Button<P03>,
    pub b04: Button<P04>,
    pub b05: Button<P05>,
    pub b06: Button<P06>,
    pub b07: Button<P07>,
    pub b08: Button<P08>,
    pub b09: Button<P09>,
    pub b10: Button<P10>,
    pub b11: Button<P11>,
    pub b12: Button<P12>,
    pub b13: Button<P13>,
    pub b14: Button<P14>,
    pub b15: Button<P15>,
    
    pub v00: Button<V00>,
    pub v01: Button<V01>,
    pub v02: Button<V02>,
    pub v03: Button<V03>,
}

impl <
    P00: PinId, 
    P01: PinId, 
    P02: PinId, 
    P03: PinId,
    P04: PinId,
    P05: PinId,
    P06: PinId,
    P07: PinId,
    P08: PinId,
    P09: PinId,
    P10: PinId,
    P11: PinId,
    P12: PinId,
    P13: PinId,
    P14: PinId,
    P15: PinId,
    
    V00: PinId,
    V01: PinId,
    V02: PinId,
    V03: PinId,
>
StepButtonSet<
    P00, 
    P01, 
    P02, 
    P03,
    P04,
    P05,
    P06,
    P07,
    P08,
    P09,
    P10,
    P11,
    P12,
    P13,
    P14,
    P15,
    
    V00,
    V01,
    V02,
    V03,
>
{
    pub fn poll(&mut self, time: u32) {
        self.b00.poll(time);
        self.b01.poll(time);
        self.b02.poll(time);
        self.b03.poll(time);
        self.b04.poll(time);
        self.b05.poll(time);
        self.b06.poll(time);
        self.b07.poll(time);
        self.b08.poll(time);
        self.b09.poll(time);
        self.b10.poll(time);
        self.b11.poll(time);
        self.b12.poll(time);
        self.b13.poll(time);
        self.b14.poll(time);
        self.b15.poll(time);
        
        self.v00.poll(time);
        self.v01.poll(time);
        self.v02.poll(time);
        self.v03.poll(time);
    }
    
    pub fn update_steps(&self, steps: &mut [[Step; NUM_STEPS]; NUM_CHANNELS], step_offset: usize, step_channel: usize) {
        if self.v00.state() == true || self.v01.state() == true || self.v02.state() == true || self.v03.state() == true {
            let mut vel = 0;
            if self.v00.state() == true { vel = VELOCITY_LEVELS[0] }
            if self.v01.state() == true { vel = VELOCITY_LEVELS[1] }
            if self.v02.state() == true { vel = VELOCITY_LEVELS[2] }
            if self.v03.state() == true { vel = VELOCITY_LEVELS[3] }
            
            if self.b00.rising_edge() == true { steps[step_channel][step_offset +  0].vel = vel }
            if self.b01.rising_edge() == true { steps[step_channel][step_offset +  1].vel = vel }
            if self.b02.rising_edge() == true { steps[step_channel][step_offset +  2].vel = vel }
            if self.b03.rising_edge() == true { steps[step_channel][step_offset +  3].vel = vel }
            if self.b04.rising_edge() == true { steps[step_channel][step_offset +  4].vel = vel }
            if self.b05.rising_edge() == true { steps[step_channel][step_offset +  5].vel = vel }
            if self.b06.rising_edge() == true { steps[step_channel][step_offset +  6].vel = vel }
            if self.b07.rising_edge() == true { steps[step_channel][step_offset +  7].vel = vel }
            if self.b08.rising_edge() == true { steps[step_channel][step_offset +  8].vel = vel }
            if self.b09.rising_edge() == true { steps[step_channel][step_offset +  9].vel = vel }
            if self.b10.rising_edge() == true { steps[step_channel][step_offset + 10].vel = vel }
            if self.b11.rising_edge() == true { steps[step_channel][step_offset + 11].vel = vel }
            if self.b12.rising_edge() == true { steps[step_channel][step_offset + 12].vel = vel }
            if self.b13.rising_edge() == true { steps[step_channel][step_offset + 13].vel = vel }
            if self.b14.rising_edge() == true { steps[step_channel][step_offset + 14].vel = vel }
            if self.b15.rising_edge() == true { steps[step_channel][step_offset + 15].vel = vel }
        } else {
            if self.b00.rising_edge() == true { steps[step_channel][step_offset +  0].gate = !steps[step_channel][step_offset +  0].gate }
            if self.b01.rising_edge() == true { steps[step_channel][step_offset +  1].gate = !steps[step_channel][step_offset +  1].gate }
            if self.b02.rising_edge() == true { steps[step_channel][step_offset +  2].gate = !steps[step_channel][step_offset +  2].gate }
            if self.b03.rising_edge() == true { steps[step_channel][step_offset +  3].gate = !steps[step_channel][step_offset +  3].gate }
            if self.b04.rising_edge() == true { steps[step_channel][step_offset +  4].gate = !steps[step_channel][step_offset +  4].gate }
            if self.b05.rising_edge() == true { steps[step_channel][step_offset +  5].gate = !steps[step_channel][step_offset +  5].gate }
            if self.b06.rising_edge() == true { steps[step_channel][step_offset +  6].gate = !steps[step_channel][step_offset +  6].gate }
            if self.b07.rising_edge() == true { steps[step_channel][step_offset +  7].gate = !steps[step_channel][step_offset +  7].gate }
            if self.b08.rising_edge() == true { steps[step_channel][step_offset +  8].gate = !steps[step_channel][step_offset +  8].gate }
            if self.b09.rising_edge() == true { steps[step_channel][step_offset +  9].gate = !steps[step_channel][step_offset +  9].gate }
            if self.b10.rising_edge() == true { steps[step_channel][step_offset + 10].gate = !steps[step_channel][step_offset + 10].gate }
            if self.b11.rising_edge() == true { steps[step_channel][step_offset + 11].gate = !steps[step_channel][step_offset + 11].gate }
            if self.b12.rising_edge() == true { steps[step_channel][step_offset + 12].gate = !steps[step_channel][step_offset + 12].gate }
            if self.b13.rising_edge() == true { steps[step_channel][step_offset + 13].gate = !steps[step_channel][step_offset + 13].gate }
            if self.b14.rising_edge() == true { steps[step_channel][step_offset + 14].gate = !steps[step_channel][step_offset + 14].gate }
            if self.b15.rising_edge() == true { steps[step_channel][step_offset + 15].gate = !steps[step_channel][step_offset + 15].gate }
        }
    }
}


pub struct ChannelButtonSet<
    P00: PinId,
    P01: PinId,
    P02: PinId,
    P03: PinId,
    P04: PinId,
    P05: PinId,
    P06: PinId,
    P07: PinId,
> 
{
    pub p00: Button<P00>,
    pub p01: Button<P01>,
    pub p02: Button<P02>,
    pub p03: Button<P03>,
    pub p04: Button<P04>,
    pub p05: Button<P05>,
    pub p06: Button<P06>,
    pub p07: Button<P07>,
}

impl <
    P00: PinId, 
    P01: PinId, 
    P02: PinId, 
    P03: PinId,
    P04: PinId,
    P05: PinId,
    P06: PinId,
    P07: PinId,
>
ChannelButtonSet<
    P00, 
    P01, 
    P02, 
    P03,
    P04,
    P05,
    P06,
    P07,
>
{
    pub fn poll(&mut self, time: u32) {
        self.p00.poll(time);
        self.p01.poll(time);
        self.p02.poll(time);
        self.p03.poll(time);
        self.p04.poll(time);
        self.p05.poll(time);
        self.p06.poll(time);
        self.p07.poll(time);
    }
    
    pub fn update_channel(&self, step_channel: &mut usize) {
        if self.p00.rising_edge() == true { *step_channel = 0 }
        if self.p01.rising_edge() == true { *step_channel = 1 }
        if self.p02.rising_edge() == true { *step_channel = 2 }
        if self.p03.rising_edge() == true { *step_channel = 3 }
        if self.p04.rising_edge() == true { *step_channel = 4 }
        if self.p05.rising_edge() == true { *step_channel = 5 }
        if self.p06.rising_edge() == true { *step_channel = 6 }
        if self.p07.rising_edge() == true { *step_channel = 7 }
    }
}

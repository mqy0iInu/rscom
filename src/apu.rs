// ==================================================================================
// APU Register
const APU_SQUARE_WAVE_1_CTRL_1_REG_ADDR: u16 = 0x4000;
const APU_SQUARE_WAVE_1_CTRL_2_REG_ADDR: u16 = 0x4001;
const APU_SQUARE_WAVE_1_FREQ_1_REG_ADDR: u16 = 0x4002;
const APU_SQUARE_WAVE_1_FREQ_2_REG_ADDR: u16 = 0x4003;
const APU_SQUARE_WAVE_2_CTRL_1_REG_ADDR: u16 = 0x4004;
const APU_SQUARE_WAVE_2_CTRL_2_REG_ADDR: u16 = 0x4005;
const APU_SQUARE_WAVE_2_FREQ_1_REG_ADDR: u16 = 0x4006;
const APU_SQUARE_WAVE_2_FREQ_2_REG_ADDR: u16 = 0x4007;
const APU_TRIANGLE_WAVE_CTRL_REG_ADDR: u16 = 0x4008;
const APU_TRIANGLE_WAVE_FREQ_1_REG_ADDR: u16 = 0x400A;
const APU_TRIANGLE_WAVE_FREQ_2_REG_ADDR: u16 = 0x400B;
const APU_NOISE_CTRL_REG_ADDR: u16 = 0x400C;
const APU_NOISE_FREQ_1_REG_ADDR: u16 = 0x400E;
const APU_NOISE_FREQ_2_REG_ADDR: u16 = 0x400F;
const APU_DMC_CTRL_1_REG_ADDR: u16 = 0x4010;
const APU_DMC_CTRL_2_REG_ADDR: u16 = 0x4011;
const APU_DMC_ADDR_REG_ADDR: u16 = 0x4012;
const APU_DMC_DATA_LENGTH_REG_ADDR: u16 = 0x4013;
const APU_SOUND_CTRL_REG_ADDR: u16 = 0x4015;
const APU_SOUND_FRAME_REG_ADDR: u16 = 0x4017;
// ==================================================================================
pub const APU_REG_READ: u8 = 0x00;
pub const APU_REG_WRITE: u8 = 0x01;

#[derive(Clone)]
pub struct APUReg {
    square_wave_1_ctrl_1_reg: u8,      // ($4000) Square Wave 1 Control 1
    square_wave_1_ctrl_2_reg: u8,      // ($4001) Square Wave 1 Control 2
    square_wave_1_freq_1_reg: u8,      // ($4002) Square Wave 1 Frequency 1
    square_wave_1_freq_2_reg: u8,      // ($4003) Square Wave 1 Frequency 2
    square_wave_2_ctrl_1_reg: u8,      // ($4004) Square Wave 2 Control 1
    square_wave_2_ctrl_2_reg: u8,      // ($4005) Square Wave 2 Control 2
    square_wave_2_freq_1_reg: u8,      // ($4006) Square Wave 2 Frequency 1
    square_wave_2_freq_2_reg: u8,      // ($4007) Square Wave 2 Frequency 2
    triangle_wave_ctrl_reg: u8,        // ($4008) Triangle Wave Control
    triangle_wave_freq_1_reg: u8,      // ($400A) Triangle Wave Frequency 1
    triangle_wave_freq_2_reg: u8,      // ($400B) Triangle Wave Frequency 2
    noise_ctrl_reg: u8,                // ($400C) Noise Control
    noise_freq_1_reg: u8,              // ($400E) Noise Frequency 1
    noise_freq_2_reg: u8,              // ($400F) Noise Frequency 2
    dmc_ctrl_1_reg: u8,                // ($4010) DMC Control 1
    dmc_ctrl_2_reg: u8,                // ($4011) DMC Control 2
    dmc_addr_reg: u8,               // ($4012) DMC Address
    dmc_data_length_reg: u8,           // ($4013) DMC Data Length
    sound_ctrl_reg: u8,             // ($4015) Sound Control
    sound_frame_reg: u8,               // ($4017) Sound Frame
}

impl APUReg {
    pub fn new() -> Self {
        APUReg {
            square_wave_1_ctrl_1_reg: 0,
            square_wave_1_ctrl_2_reg: 0,
            square_wave_1_freq_1_reg: 0,
            square_wave_1_freq_2_reg: 0,
            square_wave_2_ctrl_1_reg: 0,
            square_wave_2_ctrl_2_reg: 0,
            square_wave_2_freq_1_reg: 0,
            square_wave_2_freq_2_reg: 0,
            triangle_wave_ctrl_reg: 0,
            triangle_wave_freq_1_reg: 0,
            triangle_wave_freq_2_reg: 0,
            noise_ctrl_reg: 0,
            noise_freq_1_reg: 0,
            noise_freq_2_reg: 0,
            dmc_ctrl_1_reg: 0,
            dmc_ctrl_2_reg: 0,
            dmc_addr_reg: 0,
            dmc_data_length_reg: 0,
            sound_ctrl_reg: 0,
            sound_frame_reg: 0,
        }
    }

    fn apu_reg_read(&self, address: u16) -> u8 {
        match address {
            APU_SQUARE_WAVE_1_CTRL_1_REG_ADDR => self.square_wave_1_ctrl_1_reg,
            APU_SQUARE_WAVE_1_CTRL_2_REG_ADDR => self.square_wave_1_ctrl_2_reg,
            APU_SQUARE_WAVE_1_FREQ_1_REG_ADDR => self.square_wave_1_freq_1_reg,
            APU_SQUARE_WAVE_1_FREQ_2_REG_ADDR => self.square_wave_1_freq_2_reg,
            APU_SQUARE_WAVE_2_CTRL_1_REG_ADDR => self.square_wave_2_ctrl_1_reg,
            APU_SQUARE_WAVE_2_CTRL_2_REG_ADDR => self.square_wave_2_ctrl_2_reg,
            APU_SQUARE_WAVE_2_FREQ_1_REG_ADDR => self.square_wave_2_freq_1_reg,
            APU_SQUARE_WAVE_2_FREQ_2_REG_ADDR => self.square_wave_2_freq_2_reg,
            APU_TRIANGLE_WAVE_CTRL_REG_ADDR => self.triangle_wave_ctrl_reg,
            APU_TRIANGLE_WAVE_FREQ_1_REG_ADDR => self.triangle_wave_freq_1_reg,
            APU_TRIANGLE_WAVE_FREQ_2_REG_ADDR => self.triangle_wave_freq_2_reg,
            APU_NOISE_CTRL_REG_ADDR => self.noise_ctrl_reg,
            APU_NOISE_FREQ_1_REG_ADDR => self.noise_freq_1_reg,
            APU_NOISE_FREQ_2_REG_ADDR => self.noise_freq_2_reg,
            APU_DMC_CTRL_1_REG_ADDR => self.dmc_ctrl_1_reg,
            APU_DMC_CTRL_2_REG_ADDR => self.dmc_ctrl_2_reg,
            APU_DMC_ADDR_REG_ADDR => self.dmc_addr_reg,
            APU_DMC_DATA_LENGTH_REG_ADDR => self.dmc_data_length_reg,
            APU_SOUND_CTRL_REG_ADDR => self.sound_ctrl_reg,
            APU_SOUND_FRAME_REG_ADDR => self.sound_frame_reg,
            _ => panic!("Invalid APU Register Address: 0x{:04X}", address),
        }
    }

    fn apu_reg_write(&mut self, address: u16, data: u8) {
        match address {
            APU_SQUARE_WAVE_1_CTRL_1_REG_ADDR => self.square_wave_1_ctrl_1_reg = data,
            APU_SQUARE_WAVE_1_CTRL_2_REG_ADDR => self.square_wave_1_ctrl_2_reg = data,
            APU_SQUARE_WAVE_1_FREQ_1_REG_ADDR => self.square_wave_1_freq_1_reg = data,
            APU_SQUARE_WAVE_1_FREQ_2_REG_ADDR => self.square_wave_1_freq_2_reg = data,
            APU_SQUARE_WAVE_2_CTRL_1_REG_ADDR => self.square_wave_2_ctrl_1_reg = data,
            APU_SQUARE_WAVE_2_CTRL_2_REG_ADDR => self.square_wave_2_ctrl_2_reg = data,
            APU_SQUARE_WAVE_2_FREQ_1_REG_ADDR => self.square_wave_2_freq_1_reg = data,
            APU_SQUARE_WAVE_2_FREQ_2_REG_ADDR => self.square_wave_2_freq_2_reg = data,
            APU_TRIANGLE_WAVE_CTRL_REG_ADDR => self.triangle_wave_ctrl_reg = data,
            APU_TRIANGLE_WAVE_FREQ_1_REG_ADDR => self.triangle_wave_freq_1_reg = data,
            APU_TRIANGLE_WAVE_FREQ_2_REG_ADDR => self.triangle_wave_freq_2_reg = data,
            APU_NOISE_CTRL_REG_ADDR => self.noise_ctrl_reg = data,
            APU_NOISE_FREQ_1_REG_ADDR => self.noise_freq_1_reg = data,
            APU_NOISE_FREQ_2_REG_ADDR => self.noise_freq_2_reg = data,
            APU_DMC_CTRL_1_REG_ADDR => self.dmc_ctrl_1_reg = data,
            APU_DMC_CTRL_2_REG_ADDR => self.dmc_ctrl_2_reg = data,
            APU_DMC_ADDR_REG_ADDR => self.dmc_addr_reg = data,
            APU_DMC_DATA_LENGTH_REG_ADDR => self.dmc_data_length_reg = data,
            APU_SOUND_CTRL_REG_ADDR => self.sound_ctrl_reg = data,
            APU_SOUND_FRAME_REG_ADDR => self.sound_frame_reg = data,
            _ => panic!("Invalid APU Register Address: 0x{:04X}", address),
        }
    }

    pub fn apu_reg_ctrl(&mut self, addr: u16, wr: u8, data: u8) -> u8
    {
        if wr != APU_REG_WRITE {
            self.apu_reg_write(addr, data);
            0
        }else{
            self.apu_reg_read(addr)
        }
    }
}

pub fn apu_reset()
{
    // TODO :APU Init
}

pub fn apu_main()
{
    // println!("[DEBUG] : APU Main Loop");
}

// ====================================== TEST ======================================
#[cfg(test)]
mod apu_test {

    #[test]
    fn apu_test() {
        // TODO : APU Test
    }
}
// ==================================================================================
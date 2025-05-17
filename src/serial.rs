pub mod packets {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct PacketChecksumError;

    fn to_bool(num: u8) -> bool {
        num != 0
    }

    fn bool_or_not_to_bool(num: u16) -> bool {
        num != 0
    }

    fn get_checksum(bytes: &[u8; 15]) -> u8 {
        let mut sum: u8 = 0;

        for i in 0..14 {
            sum = sum.overflowing_add(bytes[i]).0;
        }

        return sum;
    }

    impl fmt::Display for PacketChecksumError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "packet checksum failed, probably corrupte")
        }
    }

    pub struct PedalPacket {
        pub pedal_value: u16,
        pub baseline_value: u16,
    }

    impl PedalPacket {
        const FLOORVALUE: u16 = 545;
        const UPPERVALUE: u16 = 478; // These should allways add up to 1023

        pub fn from_bytes(bytes: &[u8; 15]) -> Result<PedalPacket, PacketChecksumError> {
            if get_checksum(bytes) != bytes[14] {
                return Err(PacketChecksumError);
            }

            let baseline_value = u16::from_be_bytes([bytes[0], bytes[1]]);

            let pedal_value = u16::from_be_bytes([bytes[2], bytes[3]]).clamp(0, 1023);

            Ok(PedalPacket {
                pedal_value,
                baseline_value,
            })
        }

        pub fn get_throttle_percentage(&self) -> u8 {
            let pedal_value = (self.pedal_value as u32).clamp(PedalPacket::FLOORVALUE as u32, 1023)
                - PedalPacket::FLOORVALUE as u32;
            let throttle: u32 = 100 - ((pedal_value as u32 * 100) / PedalPacket::UPPERVALUE as u32);
            throttle.clamp(0, 100) as u8
        }
    }

    pub struct VelocityPacket {
        pub rpm: f32,
    }

    impl VelocityPacket {
        pub fn from_bytes(bytes: &[u8; 15]) -> Result<VelocityPacket, PacketChecksumError> {
            if get_checksum(bytes) != bytes[14] {
                return Err(PacketChecksumError);
            }
            let rpm = f32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

            return Ok(VelocityPacket { rpm });
        }

        pub fn to_mph(self) -> f32 {
            self.rpm * 69.3 * 60.0 / 63360.0
        }
    }

    pub struct LightsPacket {
        pub headlights: bool,
        pub right_blinkers: bool,
        pub left_blinkers: bool,
        pub brake_lights: bool,
    }

    impl LightsPacket {
        pub fn from_bytes(bytes: &[u8; 15]) -> Result<LightsPacket, PacketChecksumError> {
            if get_checksum(bytes) != bytes[14] {
                return Err(PacketChecksumError);
            }

            let headlights: bool = to_bool(bytes[0]);
            let right_blinkers: bool = to_bool(bytes[1]);
            let left_blinkers: bool = to_bool(bytes[2]);
            let brake_lights: bool = to_bool(bytes[3]);

            Ok(LightsPacket {
                headlights,
                right_blinkers,
                left_blinkers,
                brake_lights,
            })
        }
    }

    pub struct MotorTempaturePacket {
        pub heatsink_temp: f32, // WaveSculptor temp
        pub motor_temp: f32,
    }

    impl MotorTempaturePacket {
        pub fn from_bytes(bytes: &[u8; 15]) -> Result<MotorTempaturePacket, PacketChecksumError> {
            if get_checksum(bytes) != bytes[14] {
                return Err(PacketChecksumError);
            }

            let motor_temp = f32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
            let heatsink_temp = f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

            Ok(MotorTempaturePacket {
                heatsink_temp,
                motor_temp,
            })
        }
    }

    pub struct MotorStatusPacket {
        pub rx_error_count: u8,
        pub tx_error_count: u8,
        pub active_motor: u16,
        pub error_flags: u16,
        pub limit_flags: u16,
    }

    impl MotorStatusPacket {
        pub fn from_bytes(bytes: &[u8; 15]) -> Result<MotorStatusPacket, PacketChecksumError> {
            if get_checksum(bytes) != bytes[14] {
                return Err(PacketChecksumError);
            }

            let limit_flags = u16::from_be_bytes([bytes[6], bytes[7]]);
            let error_flags = u16::from_be_bytes([bytes[4], bytes[5]]);
            let active_motor = u16::from_be_bytes([bytes[2], bytes[3]]);
            let tx_error_count = bytes[1];
            let rx_error_count = bytes[0];

            Ok(MotorStatusPacket {
                rx_error_count,
                tx_error_count,
                active_motor,
                error_flags,
                limit_flags,
            })
        }

        pub fn decode_limit_flags(&self) -> LimitFlags {
            let ipm_tempature_or_motor_tempature =
                bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0100_0000);
            let bus_voltage_lower_limit =
                bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0010_0000);
            let bus_voltage_upper_limit =
                bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0001_0000);
            let bus_current = bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0000_1000);
            let velocity = bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0000_0100);
            let motor_current = bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0000_0010);
            let ouput_voltage_pwm = bool_or_not_to_bool(self.limit_flags & 0b0000_0000_0000_0001);

            LimitFlags {
                ipm_tempature_or_motor_tempature,
                bus_voltage_lower_limit,
                bus_voltage_upper_limit,
                bus_current,
                velocity,
                motor_current,
                ouput_voltage_pwm,
            }
        }
    }

    pub struct LimitFlags {
        pub ipm_tempature_or_motor_tempature: bool,
        pub bus_voltage_lower_limit: bool,
        pub bus_voltage_upper_limit: bool,
        pub bus_current: bool,
        pub velocity: bool,
        pub motor_current: bool,
        pub ouput_voltage_pwm: bool,
    }
}

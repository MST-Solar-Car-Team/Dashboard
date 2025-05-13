pub mod packets {
    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct PacketChecksumError;

    impl fmt::Display for PacketChecksumError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "packet checksum failed, probably corrupte")
        }
    }

    pub struct PedalPacket {
        pedal_value: u16,
        baseline_value: u16,
    }

    impl PedalPacket {
        pub fn from_bytes(bytes: &[u8; 15]) -> Result<PedalPacket, PacketChecksumError> {
            let baseline_value = u16::from_be_bytes([bytes[0], bytes[1]]);

            let pedal_value = u16::from_be_bytes([bytes[2], bytes[3]]);

            //TODO implement checksum

            Ok(PedalPacket {
                pedal_value,
                baseline_value,
            })
        }

        pub fn get_throttle_percentage(&self) -> u8 {
            let throttle: u32 = ((1023 - (self.pedal_value as u32)) * 100) / 1023;
            throttle.clamp(0, 100) as u8
        }
    }

    pub struct SpeedPacket {
        rpm: f32,
    }

    impl SpeedPacket {
        pub fn from_bytes(bytes: &[u8; 15]) -> Result<SpeedPacket, PacketChecksumError> {
            let rpm = f32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

            //TODO implement checksum

            Ok(SpeedPacket { rpm })
        }

        pub fn to_mph(self) -> f32 {
            self.rpm * 64.4 * 60.0 / 63360.0
        }
    }
}

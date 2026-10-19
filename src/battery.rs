use flipperzero::furi::hal::power::Power;

#[derive(Clone)]
pub struct Battery {
    power: Power,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BatteryStatus {
    pub percent: u8,
    pub charging: bool,
}

impl Battery {
    #[must_use]
    pub fn open() -> Self {
        Self {
            power: Power::open(),
        }
    }

    #[must_use]
    pub fn status(&self) -> BatteryStatus {
        let info = self.power.get_info();
        BatteryStatus {
            percent: info.charge,
            charging: info.is_charging,
        }
    }
}

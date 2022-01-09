use bal::stepper::MCDelay;

pub struct AxisUserSettings {
    pub homing_speed: MCDelay,
    pub max_speed: MCDelay,
    pub home_position: i32,
    pub range: i32,
}
pub struct BlindsUserSettings {
    pub angle_closed: i32,
    pub angle_open: i32,
    pub lateral_closed: i32,
    pub lateral_open: i32,
}
pub struct UserSettings {
    pub axis_angle_settings: AxisUserSettings,
    pub axis_open_settings: AxisUserSettings,
    pub blinds_settings: BlindsUserSettings,
}
impl Default for UserSettings {
    fn default() -> Self {
        UserSettings{
            axis_open_settings: AxisUserSettings{
                homing_speed: MCDelay::from_num(10),
                max_speed: MCDelay::from_num(100000),
                home_position: -10,
                range: 100000
            },
            axis_angle_settings: AxisUserSettings{
                homing_speed: MCDelay::from_num(10),
                max_speed: MCDelay::from_num(100000),
                home_position: -10,
                range: 100000
            },
            blinds_settings: BlindsUserSettings{
                angle_closed: 0,
                angle_open: 100,
                lateral_closed: 1000,
                lateral_open: 0
            }
        }
    }
}

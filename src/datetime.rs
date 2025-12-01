pub struct DateTime {
    pub year: String,
    pub month: String,
    pub day: String,
    pub time: String
}

impl DateTime{
    pub fn now() -> DateTime { // Mainly for testing, actual value should be passed
        let dt = chrono::offset::Utc::now().to_string(); // 2025-07-24T08:08:26.280253665Z
        let dt_values: Vec<String> = dt.split(&['-', '.', ' '])
                                        .filter(|&r| r != "")
                                        .map(|v| v.to_string())
                                        .collect(); // 2025-07-25 07:04:58.302342296 UTC -> ["2025", "07", "25", "07:04:58", "302342296", "UTC"]
        let year = dt_values[0].clone();
        let month = dt_values[1].clone();
        let day = dt_values[2].clone();
        let time = dt_values[3].clone();

        let dt: DateTime = DateTime{year, month, day, time};
        dt
    }
    pub fn encode(&self) -> Vec<u8> {
        let joined = self.to_string();
        joined.into_bytes()
    }
    pub fn decode(bytes: Vec<u8>) -> DateTime {
        let datetime = String::from_utf8(bytes).unwrap();
        let times: Vec<&str> = datetime.split(" ").collect();
        
        let year = times[0].to_string();
        let month = times[1].to_string();
        let day = times[2].to_string();
        let time = times[3].to_string();
        let dt: DateTime = DateTime{year, month, day, time};
        dt
    }
    pub fn to_string(&self) -> String {
        let joined = (&self.year).to_string() + " " + &self.month + " " + &self.day + " " + &self.time;
        joined
    }
}
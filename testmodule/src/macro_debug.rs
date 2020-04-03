#[cfg(test)]
mod tests {
    #[fml_macro::fml_macro_debug]
    pub mod handles {
        #[exported]
        pub trait WeatherRequest {
            fn weather(&self, date: String) -> Weather;
        }

        #[imported]
        pub trait WeatherResponse {
            fn weather(&self, date: String) -> Weather;
        }
    }

    #[test]
    fn this_is_just_for_macro_expansion() {
        println!("------------------------")
    }
}

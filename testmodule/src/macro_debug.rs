#[cfg(test)]
mod tests {
    #[fml_macro::fml_macro_debug]
    pub mod handles {
        #[exported]
        pub trait WeatherResponse {
            fn weather(&self, date: String) -> crate::fmltest2::cleric::core::Weather;
        }

        #[imported]
        pub trait WeatherForecast {
            fn weather(&self, date: String) -> crate::fmltest2::cleric::core::Weather;
        }
    }

    #[test]
    fn this_is_just_for_macro_expansion() {
        println!("------------------------")
    }
}

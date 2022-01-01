use super::consts::DEVELOPMENT;

pub fn get_db_name(environment: &str) -> String {
    if environment.eq(DEVELOPMENT) {
        "sailing-channels-dev".to_string()
    } else {
        "sailing-channels".to_string()
    }
}

use super::utils::{alphanumeric, alphanumeric_space_slash, BoxedParser};

pub fn data_structure<'a>(id: &str) -> BoxedParser<'a> {
    match id {
        "10" => alphanumeric_space_slash(0, 38),
        "20" => alphanumeric_space_slash(0, 38),
        "21" => alphanumeric_space_slash(0, 38),
        "22" => alphanumeric_space_slash(0, 38),
        "23" => alphanumeric_space_slash(0, 38),
        "25" => alphanumeric_space_slash(0, 32),
        "24" => alphanumeric(5, 5),
        "26" => alphanumeric(2, 2),
        _ => panic!("Unknwonw data id {}", id),
    }
}

use super::utils::{alphanumeric, alphanumeric_space_slash, digit, digit_and_comma, BoxedParser};

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
        "41" => digit_and_comma(0, 12),
        "43" => digit_and_comma(1, 5),
        "44" => alphanumeric(13, 13),
        "45" => digit(4, 4),
        "46" => alphanumeric_space_slash(0, 38),
        "47" => digit(13, 13),
        "48" => alphanumeric_space_slash(0, 38),
        "49" => digit(13, 13),
        "4A" => digit(8, 8),
        _ => panic!("Unknwonw data id {}", id),
    }
}

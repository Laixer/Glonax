pub fn shortest_rotation(distance: f32) -> f32 {
    let dist_normal = (distance + (2.0 * std::f32::consts::PI)) % (2.0 * std::f32::consts::PI);

    if dist_normal > std::f32::consts::PI {
        dist_normal - (2.0 * std::f32::consts::PI)
    } else {
        dist_normal
    }
}

#[test]
fn left_and_right() {
    let rad_10 = 0.17453;
    let rad_30 = 0.523599;
    let rad_100 = 1.74533;
    let rad_200 = 3.49065;

    assert_eq!(shortest_rotation(rad_100 - rad_200), -1.7453198);
    assert_eq!(shortest_rotation(rad_10 - rad_200), 2.9670656);
    assert_eq!(shortest_rotation(rad_30 - rad_200), -2.967051);
}

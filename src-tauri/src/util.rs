pub fn track_with_start_time_exists(start_time: &String) -> bool {
    let track_analysis = crate::load_track_analysis();
    for t in track_analysis {
        if t.start_time.eq(&Some(start_time.clone())) {
            return true;
        }
    }
    false
}
pub fn make_satellite_problem_from(pddl_file: &str) -> io::Result<(BlockState<usize>, BlockGoals<usize>)> {
    let contents = fs::read_to_string(pddl_file)?.to_lowercase();
    match Define::parse(contents.as_str()) {
        Ok(parsed) => Ok(parsed.init_and_goal()),
        Err(e) => {println!("{}", e); Err(err!(Other, "oops"))}
    }
}
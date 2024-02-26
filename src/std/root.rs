fn to_exit_code(i: i64) -> std::process::ExitCode {
    (i as u8).into()
}
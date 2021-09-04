
pub fn reexec(command: String) {
    log::info!("Running exec {}", &command)
    // we need to re-exec in Go because we want to change the hostname before the child process actually begins running
    
}
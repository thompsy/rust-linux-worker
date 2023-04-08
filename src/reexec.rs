use hostname;
use nix::mount::MsFlags;
use unshare::{Command, Namespace, Stdio};
use std::io::prelude::*;

//TODO: handle errors here in a better way. We're just unwrapping rather than actually handling them
pub fn fork_child(command: String) {
    log::info!("forking child with command {}", command);

    let parts = command.split_ascii_whitespace().collect::<Vec<&str>>();

    // this has to be unsafe because of the call to pre_exec
    unsafe {
        // TODO extract the rootfs to a newly created tmpDir so it's fresh everytime

        let mut child = Command::new(parts[0])
            .args(&parts[1..])
            .stdout(Stdio::Pipe) //TODO how do we get this out?
            .env_clear()
            .pre_exec(|| {
                log::debug!("pre_exec");
                // Set the hostname
                log::debug!("current hostname {:?}", hostname::get().unwrap());
                hostname::set("container").unwrap();
                log::debug!("new hostname {:?}", hostname::get().unwrap());

                // Mount the proc filesystem
                let result = nix::mount::mount(Some("proc"), "proc", Some("proc"), MsFlags::empty(), Some(""));
                match result {
                    Ok(_) => log::info!("/proc mounted"),
                    Err(e) => log::error!("error mounting /proc: {}", e),
                }

                Ok(())
            })
            .unshare(&[
                Namespace::Pid,
                Namespace::Uts,
                Namespace::Ipc,
                Namespace::Mount,
                Namespace::Net,
                Namespace::User,
            ])
            // NOTE: this should use pivot_root
            .chroot_dir("/tmp/alpine/") //TODO use a tmpDir here
            .spawn();

        if child.is_err() {
            log::info!("error spawning child process: {}", child.unwrap_err());
            return;
        }

        // This section works to get the stdout from the child process. I should be able to wire
        // this back into service to return the child and then get the stdout on demand.
        let mut child = child.unwrap();
        
        //TODO how do we get the output now that I'm using Stdio::Pipe for stdout?
        let mut pipe = child.stdout.as_mut().unwrap();
        
        let mut buffer = String::new();
        let output = pipe.read_to_string(&mut buffer);

        let status = child.wait();
        if status.is_err() {
            log::info!("error waiting for child process: {}", status.unwrap_err());
            return;
        }

        log::info!("output: {}", buffer);
        log::info!("child exited with status {}", status.unwrap());
    }
}

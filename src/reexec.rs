use hostname;
use nix::{mount::MsFlags, sched::CloneFlags};
use nix::mount;
use unshare::{Command, Namespace};

//TODO: handle errors here in a better way. We're just unwrapping rather than actually handling them
pub fn fork_child(command: String) {
    log::info!("forking child with command {}", command);

    let parts = command.split_ascii_whitespace().collect::<Vec<&str>>();

    // this has to be unsafe because of the call to pre_exec
    unsafe {
        // TODO extract the rootfs to a newly created tmpDir so it's fresh everytime

        let mut child = Command::new(parts[0])
            .args(&parts[1..])
            .env_clear()
            .pre_exec(|| {
                log::debug!("pre_exec");
                // Set the hostname
                log::debug!("current hostname {:?}", hostname::get().unwrap());
                hostname::set("container").unwrap();
                log::debug!("new hostname {:?}", hostname::get().unwrap());

                // Mount the proc filesystem
                nix::mount::mount(Some("proc"), "proc", Some("proc"), MsFlags::empty(), Some("")).unwrap();

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
            .chroot_dir("/tmp/alpine/") //TODO use a tmpDir here
            .spawn()
            .unwrap();

        let status = child.wait().unwrap();

        log::info!("child exited with status {}", status);
    }
}

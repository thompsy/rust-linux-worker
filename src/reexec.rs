use std::fs;

use hostname;
use nix::mount::MsFlags;
use unshare::{Command, Namespace, Stdio};

pub fn get_child(command: String) -> Result<unshare::Child, unshare::Error> {
    log::info!("forking child with command {}", command);

    let parts = command.split_ascii_whitespace().collect::<Vec<&str>>();

    log::info!("command parts {}", parts.len());

    // TODO I need to deal with errors here more sensibly! I need a higher level error type that
    // incorporates unshare and whatever custom stuff I might want to return here
    let alpine_fs = "./alpine-fs/";
    let alpine_fs_result = fs::canonicalize(alpine_fs);
    match alpine_fs_result {
        Ok(_) => {
            log::info!("fs_ok");
        }
        Err(_) => log::error!("fs_err"),
    }
    let fs = alpine_fs_result.unwrap();

    // NOTE: this has to be unsafe because of the call to pre_exec
    unsafe {
        // TODO extract the rootfs to a newly created tmpDir so it's fresh everytime
        Command::new(parts[0])
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
            // NOTE: this should use pivot_root and a tmp directory
            .chroot_dir(fs)
            // Ideally I'd like to let the caller spawn, but I'm having trouble returning the &mut
            // Command.
            .spawn()
    }
}

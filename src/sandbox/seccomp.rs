use seccomp_sys::*;

use sandbox::syscalls::Syscall;
use sandbox::error::SeccompError;

pub struct Context {
    ctx: *mut scmp_filter_ctx,
}

impl Context {
    fn init() -> Result<Context, SeccompError> {
        let ctx = unsafe { seccomp_init(SCMP_ACT_KILL) };

        if ctx.is_null() {
            return Err(SeccompError::FFI);
        }

        Ok(Context {
            ctx,
        })
    }

    fn allow_syscall(&mut self, syscall: Syscall) -> Result<(), SeccompError> {
        debug!("seccomp: allowing syscall={:?}", syscall);
        let ret = unsafe { seccomp_rule_add(self.ctx, SCMP_ACT_ALLOW, syscall.into_i32(), 0) };

        if ret != 0 {
            Err(SeccompError::FFI)
        } else {
            Ok(())
        }
    }

    fn load(&self) -> Result<(), SeccompError> {
        let ret = unsafe { seccomp_load(self.ctx) };

        if ret != 0 {
            Err(SeccompError::FFI)
        } else {
            Ok(())
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            seccomp_release(self.ctx)
        };
    }
}

pub fn activate_stage1() -> Result<(), SeccompError> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::open)?;
    ctx.allow_syscall(Syscall::close)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::stat)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::stat64)?;
    ctx.allow_syscall(Syscall::fstat)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::fstat64)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::lstat)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::lstat64)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    ctx.allow_syscall(Syscall::lseek)?; // needed for stage2
    #[cfg(not(target_arch = "arm"))]
    ctx.allow_syscall(Syscall::mmap)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::mmap2)?;
    ctx.allow_syscall(Syscall::mprotect)?;
    ctx.allow_syscall(Syscall::munmap)?;
    ctx.allow_syscall(Syscall::rt_sigprocmask)?;
    ctx.allow_syscall(Syscall::ioctl)?;
    ctx.allow_syscall(Syscall::readv)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::connect)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::send)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::sendmsg)?;
    ctx.allow_syscall(Syscall::recvmsg)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::getsockname)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::clone)?;
    ctx.allow_syscall(Syscall::uname)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::fcntl64)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::getdents)?;
    ctx.allow_syscall(Syscall::chdir)?; // needed for stage2
    ctx.allow_syscall(Syscall::getuid)?; // needed for stage2
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::getuid32)?; // needed for stage2
    ctx.allow_syscall(Syscall::getgid)?; // needed for stage2
    ctx.allow_syscall(Syscall::geteuid)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::geteuid32)?;
    ctx.allow_syscall(Syscall::getegid)?; // needed for stage2
    ctx.allow_syscall(Syscall::setuid)?; // needed for stage2
    ctx.allow_syscall(Syscall::setgid)?; // needed for stage2
    ctx.allow_syscall(Syscall::getgroups)?; // needed for stage2
    ctx.allow_syscall(Syscall::setgroups)?; // needed for stage2
    ctx.allow_syscall(Syscall::getresuid)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::getresuid32)?;
    ctx.allow_syscall(Syscall::getresgid)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::getresgid32)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::prctl)?; // needed for stage2
    ctx.allow_syscall(Syscall::chroot)?; // needed for stage2
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::sched_yield)?;
    ctx.allow_syscall(Syscall::getdents64)?;
    ctx.allow_syscall(Syscall::clock_getres)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::set_robust_list)?;
    ctx.allow_syscall(Syscall::openat)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::newfstatat)?;
    ctx.allow_syscall(Syscall::seccomp)?; // needed for stage2
    ctx.allow_syscall(Syscall::getrandom)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::pipe)?; // used in libpcap
    ctx.allow_syscall(Syscall::wait4)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::gettimeofday)?;

    ctx.load()?;

    info!("stage 1/2 is active");

    Ok(())
}

pub fn activate_stage2() -> Result<(), SeccompError> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    // ctx.allow_syscall(Syscall::open)?;
    ctx.allow_syscall(Syscall::close)?;
    // ctx.allow_syscall(Syscall::stat)?;
    // ctx.allow_syscall(Syscall::fstat)?;
    // ctx.allow_syscall(Syscall::lstat)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    #[cfg(not(target_arch = "arm"))]
    ctx.allow_syscall(Syscall::mmap)?;
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::mmap2)?;
    ctx.allow_syscall(Syscall::mprotect)?;
    ctx.allow_syscall(Syscall::munmap)?;
    ctx.allow_syscall(Syscall::rt_sigprocmask)?;
    // ctx.allow_syscall(Syscall::ioctl)?;
    ctx.allow_syscall(Syscall::readv)?;
    // ctx.allow_syscall(Syscall::socket)?;
    // ctx.allow_syscall(Syscall::connect)?;
    // ctx.allow_syscall(Syscall::sendto)?;
    // ctx.allow_syscall(Syscall::recvfrom)?;
    // ctx.allow_syscall(Syscall::sendmsg)?;
    // ctx.allow_syscall(Syscall::recvmsg)?;
    // ctx.allow_syscall(Syscall::bind)?;
    // ctx.allow_syscall(Syscall::getsockname)?;
    // ctx.allow_syscall(Syscall::setsockopt)?;
    // ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::clone)?;
    // ctx.allow_syscall(Syscall::uname)?;
    // ctx.allow_syscall(Syscall::fcntl)?;
    // ctx.allow_syscall(Syscall::getdents)?;
    // ctx.allow_syscall(Syscall::geteuid)?;
    // ctx.allow_syscall(Syscall::getresuid)?;
    // ctx.allow_syscall(Syscall::getresgid)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::sched_yield)?;
    // ctx.allow_syscall(Syscall::clock_getres)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::set_robust_list)?;
    // ctx.allow_syscall(Syscall::openat)?;
    // ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;

    ctx.load()?;

    info!("stage 2/2 is active");

    Ok(())
}

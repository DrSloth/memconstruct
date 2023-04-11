use core::panic::UnwindSafe;

pub(crate) type UnwindError = ();

/// no_std version of catch_unwind just ignores the unwind
#[inline(always)]
pub(crate) fn catch_unwind<F, R>(f: F) -> Result<R, UnwindError>
where
    F: FnOnce() -> R + UnwindSafe,
{
    Ok(f())
}

#[inline(always)]
pub(crate) fn resume_unwind<T>(_t: T) -> ! {
    panic!("Resuming unwind")
}


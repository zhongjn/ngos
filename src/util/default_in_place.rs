pub trait DefaultInPlace {
    unsafe fn default_in_place(s: *mut Self);
}

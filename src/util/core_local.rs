//use core::ops::Deref;
//
//#[macro_export]
//macro_rules! core_local {
//    () => {};
//
//    // process multiple declarations
//    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr; $($rest:tt)*) => (
//        $crate::__core_local_inner!($(#[$attr])* $vis $name, $t, $init);
//        $crate::core_local!($($rest)*);
//    );
//
//    // handle a single declaration
//    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr) => (
//        $crate::__core_local_inner!($(#[$attr])* $vis $name, $t, $init);
//    );
//}
//
//// this is trivially implemented
//// only suited for single core
//struct CoreLocal<T> {
//    val: T
//}
//
//unsafe impl<T> Sync for CoreLocal<T> {}
//
//impl<T> Deref for CoreLocal<T> {
//    type Target = T;
//
//    fn deref(&self) -> &Self::Target {
//        &self.val
//    }
//}
//
//#[doc(hidden)]
//#[macro_export]
//macro_rules! __core_local_inner {
//    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty, $init:expr) => {
//        $(#[$attr])* $vis $name: $crate::CoreLocal<$t> = $crate::CoreLocal { val: $init }
//    };
//}
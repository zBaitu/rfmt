pub mod outer_mod {
    pub mod inner_mod {
        pub(in crate::outer_mod) mod aa;
        pub(in outer_mod) mod bb;

        pub(crate) mod cc;
        pub(in crate) mod ccc;

        pub(super) mod dd;
        pub(in super) mod ddd;

        pub(self) mod ee;
        pub mod ff;
        crate mod gg;

        mod hh;
    }
}

//! What the macro does is visible in the documentation it writes, which only
//! rustdoc reads, so what a test can hold is that the item it rewrites is
//! still the item it was given.

#[typstdoc::typstdoc]
mod hodge {
    //! Exterior calculus on $Lambda^k$.

    /// The Hodge star $star: Lambda^k -> Lambda^(n-k)$ satisfies
    /// $ alpha and star beta = gamma. $
    ///
    /// Neither `$ cargo doc` nor
    ///
    /// ```
    /// let price = "$5";
    /// ```
    ///
    /// is a fragment.
    pub struct Star;

    pub mod nested {
        /// A nested item is reached too: $integral_Omega dif omega$.
        pub fn deep() -> u8 {
            42
        }
    }
}

#[test]
fn the_item_survives_its_documentation() {
    let _ = hodge::Star;
    assert_eq!(hodge::nested::deep(), 42);
}
